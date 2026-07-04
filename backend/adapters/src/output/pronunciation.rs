use std::time::Duration;

use application::ports::output::{
    PronunciationAssessor,
    pronunciation_assessor::models::{
        PronunciationAssessmentError, PronunciationAssessmentReport,
        PronunciationAssessmentRequest, PronunciationPhonemeAssessment,
        PronunciationPhonemeCandidate, PronunciationWordAssessment,
    },
};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::Response;
use serde::{Deserialize, Serialize};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_PCM_BYTES: usize = 16_000 * 2 * 10;

#[derive(Clone, Default)]
pub struct AzurePronunciationAssessor {
    client: reqwest::Client,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct AssessmentHeader<'a> {
    reference_text: &'a str,
    grading_system: &'static str,
    granularity: &'static str,
    dimension: &'static str,
    enable_miscue: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_prosody_assessment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phoneme_alphabet: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n_best_phoneme_count: Option<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AzureResponse {
    recognition_status: String,
    display_text: Option<String>,
    #[serde(default)]
    n_best: Vec<AzureBest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AzureBest {
    pron_score: Option<f64>,
    fluency_score: Option<f64>,
    completeness_score: Option<f64>,
    prosody_score: Option<f64>,
    display: Option<String>,
    #[serde(default)]
    words: Vec<AzureWord>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AzureWord {
    word: Option<String>,
    accuracy_score: Option<f64>,
    error_type: Option<String>,
    #[serde(default)]
    phonemes: Vec<AzurePhoneme>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AzurePhoneme {
    phoneme: Option<String>,
    accuracy_score: Option<f64>,
    #[serde(default)]
    n_best_phonemes: Vec<AzurePhonemeCandidate>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AzurePhonemeCandidate {
    phoneme: Option<String>,
    score: Option<f64>,
}

impl AzurePronunciationAssessor {
    fn validate_wav(audio: &[u8]) -> Result<(), PronunciationAssessmentError> {
        if audio.len() < 44
            || &audio[..4] != b"RIFF"
            || &audio[8..12] != b"WAVE"
            || &audio[12..16] != b"fmt "
        {
            return Err(PronunciationAssessmentError::InvalidAudio);
        }
        let format_size = u32::from_le_bytes(audio[16..20].try_into().unwrap()) as usize;
        if format_size < 16 || audio.len() < 20 + format_size {
            return Err(PronunciationAssessmentError::InvalidAudio);
        }
        let audio_format = u16::from_le_bytes(audio[20..22].try_into().unwrap());
        let channels = u16::from_le_bytes(audio[22..24].try_into().unwrap());
        let sample_rate = u32::from_le_bytes(audio[24..28].try_into().unwrap());
        let bits_per_sample = u16::from_le_bytes(audio[34..36].try_into().unwrap());
        if audio_format != 1 || channels != 1 || sample_rate != 16_000 || bits_per_sample != 16 {
            return Err(PronunciationAssessmentError::InvalidAudio);
        }
        let mut offset = 20 + format_size;
        while offset + 8 <= audio.len() {
            let chunk_size =
                u32::from_le_bytes(audio[offset + 4..offset + 8].try_into().unwrap()) as usize;
            let data_start = offset + 8;
            let data_end = data_start
                .checked_add(chunk_size)
                .ok_or(PronunciationAssessmentError::InvalidAudio)?;
            if data_end > audio.len() {
                return Err(PronunciationAssessmentError::InvalidAudio);
            }
            if &audio[offset..offset + 4] == b"data" {
                if chunk_size == 0 || chunk_size > MAX_PCM_BYTES || chunk_size % 2 != 0 {
                    return Err(PronunciationAssessmentError::InvalidAudio);
                }
                return Ok(());
            }
            offset = data_end + (chunk_size % 2);
        }
        Err(PronunciationAssessmentError::InvalidAudio)
    }

    async fn bounded_body(mut response: Response) -> Result<Vec<u8>, PronunciationAssessmentError> {
        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
        {
            return Err(PronunciationAssessmentError::InvalidResponse);
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| {
            PronunciationAssessmentError::Provider(
                "failed to read the Azure Speech response".to_string(),
            )
        })? {
            if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
                return Err(PronunciationAssessmentError::InvalidResponse);
            }
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }

    fn score(value: Option<f64>) -> Option<u8> {
        value
            .filter(|value| value.is_finite())
            .map(|value| value.round().clamp(0.0, 100.0) as u8)
    }
}

#[async_trait]
impl PronunciationAssessor for AzurePronunciationAssessor {
    async fn assess(
        &self,
        request: PronunciationAssessmentRequest,
    ) -> Result<PronunciationAssessmentReport, PronunciationAssessmentError> {
        Self::validate_wav(&request.audio)?;
        let is_english = request.locale == "en-US";
        let configuration = AssessmentHeader {
            reference_text: &request.reference_text,
            grading_system: "HundredMark",
            granularity: "Phoneme",
            dimension: "Comprehensive",
            enable_miscue: true,
            enable_prosody_assessment: is_english.then_some(true),
            phoneme_alphabet: is_english.then_some("IPA"),
            n_best_phoneme_count: is_english.then_some(5),
        };
        let header = STANDARD.encode(
            serde_json::to_vec(&configuration)
                .map_err(|_| PronunciationAssessmentError::InvalidResponse)?,
        );
        let endpoint = format!(
            "{}/stt/speech/recognition/conversation/cognitiveservices/v1",
            request.endpoint.trim_end_matches('/')
        );
        let response = self
            .client
            .post(endpoint)
            .query(&[
                ("language", request.locale.as_str()),
                ("format", "detailed"),
            ])
            .header("Ocp-Apim-Subscription-Key", request.subscription_key)
            .header("Pronunciation-Assessment", header)
            .header("Accept", "application/json")
            .header(
                "Content-Type",
                "audio/wav; codecs=audio/pcm; samplerate=16000",
            )
            .timeout(REQUEST_TIMEOUT)
            .body(request.audio)
            .send()
            .await
            .map_err(|_| {
                PronunciationAssessmentError::Provider("could not reach Azure Speech".to_string())
            })?;
        if !response.status().is_success() {
            return Err(PronunciationAssessmentError::Provider(format!(
                "Azure Speech returned HTTP {}",
                response.status()
            )));
        }
        let response: serde_json::Value =
            serde_json::from_slice(&Self::bounded_body(response).await?)
                .map_err(|_| PronunciationAssessmentError::InvalidResponse)?;
        let response: AzureResponse = serde_json::from_value(response)
            .map_err(|_| PronunciationAssessmentError::InvalidResponse)?;
        if matches!(
            response.recognition_status.as_str(),
            "NoMatch" | "InitialSilenceTimeout" | "BabbleTimeout"
        ) {
            return Ok(PronunciationAssessmentReport {
                pronunciation_score: Some(0),
                fluency_score: None,
                completeness_score: 0,
                prosody_score: None,
                recognized_text: response.display_text,
                words: Vec::new(),
            });
        }
        if response.recognition_status != "Success" {
            return Err(PronunciationAssessmentError::Provider(
                "Azure Speech could not assess the recording".to_string(),
            ));
        }
        let best = response
            .n_best
            .into_iter()
            .next()
            .ok_or(PronunciationAssessmentError::InvalidResponse)?;
        let words = best
            .words
            .into_iter()
            .map(|word| {
                let phonemes = word
                    .phonemes
                    .into_iter()
                    .map(|phoneme| {
                        let candidates = phoneme
                            .n_best_phonemes
                            .into_iter()
                            .filter_map(|candidate| {
                                Some(PronunciationPhonemeCandidate {
                                    phoneme: candidate.phoneme?,
                                    score: Self::score(candidate.score)?,
                                })
                            })
                            .collect();
                        Ok(PronunciationPhonemeAssessment {
                            phoneme: phoneme.phoneme.filter(|value| !value.trim().is_empty()),
                            accuracy_score: Self::score(phoneme.accuracy_score)
                                .ok_or(PronunciationAssessmentError::InvalidResponse)?,
                            candidates,
                        })
                    })
                    .collect::<Result<Vec<_>, PronunciationAssessmentError>>()?;
                Ok(PronunciationWordAssessment {
                    word: word.word.unwrap_or_default(),
                    accuracy_score: Self::score(word.accuracy_score)
                        .ok_or(PronunciationAssessmentError::InvalidResponse)?,
                    error_type: word.error_type,
                    phonemes,
                })
            })
            .collect::<Result<Vec<_>, PronunciationAssessmentError>>()?;
        if words.is_empty() || words.iter().any(|word| word.phonemes.is_empty()) {
            return Err(PronunciationAssessmentError::InvalidResponse);
        }
        Ok(PronunciationAssessmentReport {
            pronunciation_score: Self::score(best.pron_score),
            fluency_score: Self::score(best.fluency_score),
            completeness_score: Self::score(best.completeness_score)
                .ok_or(PronunciationAssessmentError::InvalidResponse)?,
            prosody_score: Self::score(best.prosody_score),
            recognized_text: best.display.or(response.display_text),
            words,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc,
        thread,
    };

    use super::*;

    fn wav(data_bytes: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(44 + data_bytes);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36_u32 + data_bytes as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&16_000_u32.to_le_bytes());
        bytes.extend_from_slice(&32_000_u32.to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&16_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(data_bytes as u32).to_le_bytes());
        bytes.resize(44 + data_bytes, 0);
        bytes
    }

    #[test]
    fn accepts_expected_wav_and_rejects_long_audio() {
        assert_eq!(
            AzurePronunciationAssessor::validate_wav(&wav(32_000)),
            Ok(())
        );
        assert_eq!(
            AzurePronunciationAssessor::validate_wav(&wav(MAX_PCM_BYTES + 2)),
            Err(PronunciationAssessmentError::InvalidAudio)
        );
    }

    #[test]
    fn clamps_and_rounds_provider_scores() {
        assert_eq!(AzurePronunciationAssessor::score(Some(74.6)), Some(75));
        assert_eq!(AzurePronunciationAssessor::score(Some(120.0)), Some(100));
        assert_eq!(AzurePronunciationAssessor::score(Some(f64::NAN)), None);
    }

    #[tokio::test]
    async fn sends_azure_headers_and_parses_the_detailed_score() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let read = stream.read(&mut buffer).unwrap();
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") && request.len() >= 44 {
                    let headers = String::from_utf8_lossy(&request);
                    let content_length = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    let header_end = request
                        .windows(4)
                        .position(|window| window == b"\r\n\r\n")
                        .unwrap()
                        + 4;
                    if request.len() >= header_end + content_length {
                        break;
                    }
                }
            }
            sender.send(request).unwrap();
            let body = br#"{"RecognitionStatus":"Success","DisplayText":"hello","NBest":[{"AccuracyScore":87.4,"PronScore":84.2,"FluencyScore":91.0,"CompletenessScore":100.0,"ProsodyScore":72.0,"Display":"hello","Words":[{"Word":"hello","AccuracyScore":87.4,"ErrorType":"None","Phonemes":[{"Phoneme":"h","AccuracyScore":76.0,"NBestPhonemes":[{"Phoneme":"h","Score":90.0},{"Phoneme":"f","Score":30.0}]}]}]}]}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(body).unwrap();
        });
        let result = AzurePronunciationAssessor::default()
            .assess(PronunciationAssessmentRequest {
                endpoint,
                subscription_key: "test-key".to_string(),
                locale: "en-US".to_string(),
                reference_text: "hello".to_string(),
                audio: wav(32_000),
            })
            .await
            .unwrap();
        assert_eq!(result.pronunciation_score, Some(84));
        assert_eq!(result.fluency_score, Some(91));
        assert_eq!(result.completeness_score, 100);
        assert_eq!(result.prosody_score, Some(72));
        assert_eq!(result.words.len(), 1);
        assert_eq!(result.words[0].phonemes[0].phoneme.as_deref(), Some("h"));
        assert_eq!(result.words[0].phonemes[0].candidates[0].score, 90);
        let request = String::from_utf8_lossy(&receiver.recv().unwrap()).to_string();
        assert!(request.contains("language=en-US&format=detailed"));
        assert!(request.contains("ocp-apim-subscription-key: test-key"));
        assert!(request.contains("pronunciation-assessment:"));
    }
}
