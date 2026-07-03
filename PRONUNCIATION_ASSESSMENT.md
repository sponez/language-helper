# Pronunciation assessment

Status: implemented for desktop Learn mini-tests and Test sessions.

## Provider and privacy

The first implementation uses Azure Speech Pronunciation Assessment through
the short-audio REST API. Azure settings belong to a local user and are shared
by all of that user's language profiles.

The frontend records a maximum of 10 seconds and converts it to mono 16-bit PCM
WAV at 16 kHz. The recording is held in memory, sent to Azure through the Rust
backend, and is never written to SQLite or the filesystem.

Azure is an external cloud service. Audio leaves the device, internet access is
required, and the subscription key is stored locally in `language-helper.db`
beside the executable, following the same policy as the existing AI API key.

## Session flow

Pronunciation checking and its required accuracy are selected for each Learn or
Test session. The default threshold is 75.

Before the written answer:

1. The user records the displayed word or phrase.
2. Azure returns a normalized assessment report.
3. `AccuracyScore` is compared with the session threshold.
4. A passing attempt opens the written answer.
5. The first failed attempt allows one retry.
6. The second failed attempt immediately fails the card. Test applies `-2`;
   a Learn mini-test marks its set for repetition.

`NoMatch`, initial silence, and noise consume an assessed attempt with score 0.
Network, provider, invalid-response, microphone, and encoding failures are
technical failures and do not consume pronunciation attempts. After two
consecutive technical failures, the user must disable pronunciation checking
for the rest of the current session. A valid Azure response resets that
technical-failure counter.

The session repository stores normalized reports and pass/fail decisions, but
not raw audio.

## Reference text

The card language follows its direction:

- Straight uses the profile target language.
- Reverse uses the profile source language.

Reference text sent to Azure:

- English: `card.word`.
- Russian: `card.word` with stress marks removed.
- Japanese: the first canonical kana reading, falling back to `card.word`.

`readings` remains a human-readable pronunciation aid:

- English: General American IPA enclosed in `/slashes/`.
- Russian: Cyrillic spelling with lexical stress marked.
- Japanese: canonical kana reading without romaji or pitch-accent notation.

## Japanese pitch accent

Japanese pitch accent is not evaluated separately. Azure's general
`AccuracyScore` determines pass/fail.

A future pitch-accent implementation requires trusted accent metadata,
speaker-normalized pitch extraction, mora alignment, and separate calibration.
It must initially be reported independently and must not affect pass/fail until
validated against real recordings.
