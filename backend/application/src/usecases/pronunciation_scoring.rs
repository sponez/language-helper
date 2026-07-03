use crate::ports::{
    input::study_session::models::{PronunciationAssessmentIssue, PronunciationAssessmentReport},
    output::pronunciation_assessor::models::{
        PronunciationAssessmentReport as ProviderReport, PronunciationPhonemeAssessment,
        PronunciationWordAssessment,
    },
};

pub const PRONUNCIATION_SCORING_VERSION: u8 = 3;
const PHONEME_TAIL_PERCENT: usize = 20;
const ENGLISH_SUBSTITUTION_MARGIN: u8 = 5;

pub fn score_pronunciation(
    locale: &str,
    threshold: u8,
    report: ProviderReport,
) -> PronunciationAssessmentReport {
    let weakest_phoneme_score = report
        .words
        .iter()
        .flat_map(|word| word.phonemes.iter())
        .map(|phoneme| phoneme.accuracy_score)
        .min();
    let weakest_word_score = report.words.iter().map(word_score).min();

    let strict_score =
        weakest_word_score.map_or(0, |word_score| report.completeness_score.min(word_score));

    let mut issues = word_issues(&report.words);
    if locale == "en-US" {
        issues.extend(english_substitution_issues(&report.words));
    }
    let passed = strict_score >= threshold && issues.is_empty();

    PronunciationAssessmentReport {
        strict_score,
        weakest_phoneme_score,
        weakest_word_score,
        pronunciation_score: report.pronunciation_score,
        fluency_score: report.fluency_score,
        completeness_score: Some(report.completeness_score),
        prosody_score: report.prosody_score,
        recognized_text: report.recognized_text,
        issues,
        scoring_version: PRONUNCIATION_SCORING_VERSION,
        passed,
    }
}

fn word_score(word: &PronunciationWordAssessment) -> u8 {
    let Some(phoneme_score) = lower_tail_score(&word.phonemes) else {
        return word.accuracy_score;
    };
    word.accuracy_score.min(phoneme_score)
}

fn lower_tail_score(phonemes: &[PronunciationPhonemeAssessment]) -> Option<u8> {
    if phonemes.is_empty() {
        return None;
    }
    let mut scores = phonemes
        .iter()
        .map(|phoneme| phoneme.accuracy_score)
        .collect::<Vec<_>>();
    scores.sort_unstable();
    let count = (scores.len() * PHONEME_TAIL_PERCENT).div_ceil(100).max(1);
    Some(
        (scores[..count]
            .iter()
            .map(|score| u32::from(*score))
            .sum::<u32>()
            / count as u32) as u8,
    )
}

fn word_issues(words: &[PronunciationWordAssessment]) -> Vec<PronunciationAssessmentIssue> {
    words
        .iter()
        .filter_map(|word| {
            let error_type = word.error_type.as_deref()?.trim();
            (!error_type.eq_ignore_ascii_case("none")).then(|| {
                PronunciationAssessmentIssue::WordError {
                    word: word.word.clone(),
                    error_type: error_type.to_string(),
                }
            })
        })
        .collect()
}

fn english_substitution_issues(
    words: &[PronunciationWordAssessment],
) -> Vec<PronunciationAssessmentIssue> {
    words
        .iter()
        .flat_map(|word| {
            word.phonemes.iter().filter_map(|phoneme| {
                let expected = phoneme.phoneme.as_deref()?.trim();
                if expected.is_empty() || phoneme.candidates.is_empty() {
                    return None;
                }
                let expected_score = phoneme
                    .candidates
                    .iter()
                    .find(|candidate| candidate.phoneme == expected)
                    .map(|candidate| candidate.score);
                let competitor = phoneme
                    .candidates
                    .iter()
                    .filter(|candidate| candidate.phoneme != expected)
                    .max_by_key(|candidate| candidate.score)?;
                let is_substitution = expected_score.is_none_or(|expected_score| {
                    competitor.score > expected_score.saturating_add(ENGLISH_SUBSTITUTION_MARGIN)
                });
                is_substitution.then(|| PronunciationAssessmentIssue::PhonemeSubstitution {
                    word: word.word.clone(),
                    expected: expected.to_string(),
                    detected: competitor.phoneme.clone(),
                })
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::ports::output::pronunciation_assessor::models::{
        PronunciationPhonemeAssessment, PronunciationPhonemeCandidate, PronunciationWordAssessment,
    };

    fn phoneme(name: &str, accuracy: u8) -> PronunciationPhonemeAssessment {
        PronunciationPhonemeAssessment {
            phoneme: (!name.is_empty()).then(|| name.to_string()),
            accuracy_score: accuracy,
            candidates: Vec::new(),
        }
    }

    fn report(
        completeness: u8,
        word_accuracy: u8,
        phonemes: Vec<PronunciationPhonemeAssessment>,
    ) -> ProviderReport {
        ProviderReport {
            pronunciation_score: None,
            fluency_score: Some(100),
            completeness_score: completeness,
            prosody_score: None,
            recognized_text: None,
            words: vec![PronunciationWordAssessment {
                word: "word".to_string(),
                accuracy_score: word_accuracy,
                error_type: Some("None".to_string()),
                phonemes,
            }],
        }
    }

    #[test]
    fn averages_the_weakest_twenty_percent_of_each_word() {
        let scored = score_pronunciation(
            "en-US",
            75,
            report(
                100,
                98,
                vec![
                    phoneme("f", 91),
                    phoneme("oʊ", 100),
                    phoneme("t", 100),
                    phoneme("ə", 100),
                    phoneme("g", 100),
                    phoneme("ɹ", 100),
                    phoneme("æ", 100),
                    phoneme("f", 100),
                ],
            ),
        );

        assert_eq!(scored.weakest_phoneme_score, Some(91));
        assert_eq!(scored.weakest_word_score, Some(95));
        assert_eq!(scored.strict_score, 95);
        assert!(scored.passed);
    }

    #[test]
    fn an_explicit_english_substitution_is_a_hard_failure() {
        let mut substituted = phoneme("θ", 92);
        substituted.candidates = vec![
            PronunciationPhonemeCandidate {
                phoneme: "t".to_string(),
                score: 100,
            },
            PronunciationPhonemeCandidate {
                phoneme: "θ".to_string(),
                score: 92,
            },
        ];
        let scored = score_pronunciation(
            "en-US",
            75,
            report(
                100,
                96,
                vec![
                    substituted,
                    phoneme("ɪ", 89),
                    phoneme("ŋ", 100),
                    phoneme("k", 95),
                ],
            ),
        );

        assert_eq!(scored.strict_score, 89);
        assert!(!scored.passed);
        assert_eq!(
            scored.issues,
            vec![PronunciationAssessmentIssue::PhonemeSubstitution {
                word: "word".to_string(),
                expected: "θ".to_string(),
                detected: "t".to_string(),
            }]
        );
    }

    #[test]
    fn any_word_error_is_a_hard_failure() {
        let mut provider = report(100, 98, vec![phoneme("", 98)]);
        provider.words[0].error_type = Some("UnexpectedFutureError".to_string());
        let scored = score_pronunciation("ru-RU", 75, provider);

        assert!(!scored.passed);
        assert!(matches!(
            scored.issues.as_slice(),
            [PronunciationAssessmentIssue::WordError { error_type, .. }]
                if error_type == "UnexpectedFutureError"
        ));
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CalibrationCase {
        quality: String,
        locale: String,
        word: String,
        completeness: u8,
        word_accuracy: u8,
        error_type: Option<String>,
        phonemes: Vec<CalibrationPhoneme>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CalibrationPhoneme {
        phoneme: Option<String>,
        accuracy: u8,
        expected_score: Option<u8>,
        competitor: Option<String>,
        competitor_score: Option<u8>,
    }

    #[test]
    fn calibration_fixture_accepts_all_good_and_rejects_thirteen_degraded_samples() {
        let cases: Vec<CalibrationCase> = serde_json::from_str(include_str!(
            "../../tests/fixtures/pronunciation_calibration.json"
        ))
        .unwrap();
        let mut good_passes = 0;
        let mut degraded_failures = 0;
        let mut degraded_passes = Vec::new();

        for case in cases {
            let phonemes = case
                .phonemes
                .into_iter()
                .map(|phoneme| {
                    let mut candidates = Vec::new();
                    if let (Some(expected), Some(score)) =
                        (phoneme.phoneme.as_ref(), phoneme.expected_score)
                    {
                        candidates.push(PronunciationPhonemeCandidate {
                            phoneme: expected.clone(),
                            score,
                        });
                    }
                    if let (Some(competitor), Some(score)) =
                        (phoneme.competitor, phoneme.competitor_score)
                    {
                        candidates.push(PronunciationPhonemeCandidate {
                            phoneme: competitor,
                            score,
                        });
                    }
                    PronunciationPhonemeAssessment {
                        phoneme: phoneme.phoneme,
                        accuracy_score: phoneme.accuracy,
                        candidates,
                    }
                })
                .collect();
            let scored = score_pronunciation(
                &case.locale,
                75,
                ProviderReport {
                    pronunciation_score: None,
                    fluency_score: Some(100),
                    completeness_score: case.completeness,
                    prosody_score: None,
                    recognized_text: Some(case.word.clone()),
                    words: vec![PronunciationWordAssessment {
                        word: case.word.clone(),
                        accuracy_score: case.word_accuracy,
                        error_type: case.error_type.or_else(|| Some("None".to_string())),
                        phonemes,
                    }],
                },
            );
            if case.quality == "good" {
                good_passes += usize::from(scored.passed);
            } else if scored.passed {
                degraded_passes.push((case.locale, case.word, case.quality));
            } else {
                degraded_failures += 1;
            }
        }

        assert_eq!(good_passes, 7);
        assert_eq!(degraded_failures, 13);
        assert_eq!(
            degraded_passes,
            vec![(
                "ja-JP".to_string(),
                "がっこう".to_string(),
                "worse".to_string()
            )]
        );
    }
}
