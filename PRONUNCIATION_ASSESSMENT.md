# Pronunciation assessment

Status: implemented for desktop Learn mini-tests and Test sessions.

## Provider and privacy

The first implementation uses Azure Speech Pronunciation Assessment through
the short-audio REST API. Azure settings belong to a local user and are shared
by all of that user's language profiles.

The frontend records a maximum of 10 seconds and converts it to mono 16-bit PCM
WAV at 16 kHz. The recording is held in memory, sent to Azure through the Rust
backend, and is never written to SQLite, the filesystem, or application logs.

Azure is an external cloud service. Audio leaves the device, internet access is
required, and the subscription key is stored locally in `language-helper.db`,
following the same policy as the existing AI API key. Windows keeps the
database beside the executable; macOS and Linux use Tauri's
application-local-data directory.

## Session flow

Pronunciation checking and its required strict score are selected for each
Learn or Test session. The default threshold is 75. The last selections are
stored separately for each language profile and mode.

Assessment applies only to straight cards. A reverse-only session disables the
option, while an `Any` session checks straight cards and automatically skips
reverse cards. Skipped cards do not send audio to Azure and do not consume
provider quota.

Before the written answer:

1. The user records the displayed word or phrase.
2. Azure returns word- and phoneme-level assessment evidence.
3. The application calculates a strict score from completeness and the weakest
   20% of phonemes in every word. Azure's overall `AccuracyScore` is ignored.
4. The strict score is compared with the session threshold. Azure word errors
   and clear English phoneme substitutions are hard failures regardless of the
   score.
5. A passing attempt opens the written answer.
6. The first failed attempt allows one retry.
7. The second failed attempt immediately fails the card. Test applies `-2`;
   a Learn mini-test marks its set for repetition.

`NoMatch`, initial silence, and noise consume an assessed attempt with score 0.
Network, provider, invalid-response, microphone, and encoding failures are
technical failures and do not consume pronunciation attempts. After two
consecutive technical failures, the user must disable pronunciation checking
for the rest of the current session. A valid Azure response resets that
technical-failure counter.

The session repository stores normalized reports and pass/fail decisions, but
not raw audio.

Assessment requests use phoneme granularity. English additionally requests IPA
phonemes, five likely spoken-phoneme candidates, and prosody assessment.
`PronScore`, fluency, and prosody remain diagnostic because they did not
reliably separate the calibration recordings.

The strict scoring policy is versioned. Its current rules are:

- Calculate each word's phonetic score as the integer average of its weakest
  20% of phonemes, using at least one phoneme.
- Use the lower of completeness and the weakest word phonetic score as the
  session-facing strict score.
- Fail on every Azure word `ErrorType` other than `None`.
- For English, fail when the strongest competing IPA candidate exceeds the
  expected phoneme by more than five points, or the expected candidate is
  absent.

## Reference text

Only straight cards are assessed, using the profile target language. Reference
text sent to Azure:

- English: `card.word`.
- Russian: `card.word` with stress marks removed.
- Japanese: the first canonical kana reading, falling back to `card.word`.

`readings` remains a human-readable pronunciation aid, but it is intentionally
hidden while the user is recording an assessed pronunciation:

- English: General American IPA enclosed in `/slashes/`.
- Russian: Cyrillic spelling with lexical stress marked.
- Japanese: canonical kana reading without romaji or pitch-accent notation.

## Japanese pitch accent

Japanese pitch accent is not evaluated separately. General segment scores
affect the strict score, but they cannot be interpreted as pitch-accent
assessment.

A future pitch-accent implementation requires trusted accent metadata,
speaker-normalized pitch extraction, mora alignment, and separate calibration.
It must initially be reported independently and must not affect pass/fail until
validated against real recordings.

The calibration fixture contains 21 normalized reports: seven natural and
fourteen intentionally degraded. Strict scoring version 3 accepts all seven
natural recordings and rejects thirteen degraded recordings. Azure still
accepts `がこう` in place of `がっこう`; the returned evidence does not contain a
reliable signal from which that missing geminate can be recovered.
