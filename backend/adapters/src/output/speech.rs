use std::time::Duration;

use application::ports::output::{
    SpeechSynthesizer,
    speech_synthesizer::models::{
        SpeechSynthesisError, SpeechSynthesisIdentity, SpeechSynthesisRequest,
        SpeechSynthesisResult,
    },
};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::Response;
use serde::{Deserialize, Serialize};

const GEMINI_MODEL: &str = "gemini-3.1-flash-tts-preview";
const GEMINI_VOICE: &str = "Iapetus";
const OPENAI_MODEL: &str = "gpt-4o-mini-tts";
const OPENAI_VOICE: &str = "marin";
const MAX_AUDIO_BYTES: usize = 5 * 1024 * 1024;
const MAX_JSON_BYTES: usize = 7 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_ATTEMPTS: usize = 3;

#[derive(Clone)]
pub struct AiSpeechSynthesizer {
    client: reqwest::Client,
    gemini_endpoint: String,
    openai_endpoint: String,
}

impl Default for AiSpeechSynthesizer {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            gemini_endpoint: format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_MODEL}:generateContent"
            ),
            openai_endpoint: "https://api.openai.com/v1/audio/speech".to_string(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest<'a> {
    contents: [GeminiContent<'a>; 1],
    generation_config: GeminiGenerationConfig<'a>,
}

#[derive(Serialize)]
struct GeminiContent<'a> {
    parts: [GeminiPart<'a>; 1],
}

#[derive(Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig<'a> {
    response_modalities: [&'static str; 1],
    speech_config: GeminiSpeechConfig<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiSpeechConfig<'a> {
    voice_config: GeminiVoiceConfig,
    language_code: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiVoiceConfig {
    prebuilt_voice_config: GeminiPrebuiltVoiceConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiPrebuiltVoiceConfig {
    voice_name: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponsePart {
    inline_data: Option<GeminiInlineData>,
}

#[derive(Deserialize)]
struct GeminiInlineData {
    data: String,
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'static str,
    voice: &'static str,
    input: &'a str,
    instructions: &'a str,
    response_format: &'static str,
}

impl AiSpeechSynthesizer {
    async fn bounded_body(
        mut response: Response,
        limit: usize,
    ) -> Result<Vec<u8>, SpeechSynthesisError> {
        if response
            .content_length()
            .is_some_and(|length| length > limit as u64)
        {
            return Err(SpeechSynthesisError::InvalidResponse);
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| SpeechSynthesisError::Provider(error.to_string()))?
        {
            if body.len() + chunk.len() > limit {
                return Err(SpeechSynthesisError::InvalidResponse);
            }
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }

    async fn response_error(response: Response) -> SpeechSynthesisError {
        let status = response.status();
        let body = Self::bounded_body(response, 1024).await.unwrap_or_default();
        let detail = String::from_utf8_lossy(&body);
        SpeechSynthesisError::Provider(format!(
            "provider returned HTTP {status}: {}",
            detail.trim()
        ))
    }

    async fn retry_delay(attempt: usize) {
        tokio::time::sleep(Duration::from_millis(150 * (attempt as u64 + 1))).await;
    }

    fn pcm_to_wav(pcm: &[u8]) -> Result<Vec<u8>, SpeechSynthesisError> {
        if pcm.is_empty() || pcm.len() > MAX_AUDIO_BYTES - 44 || pcm.len() % 2 != 0 {
            return Err(SpeechSynthesisError::InvalidResponse);
        }
        let data_size =
            u32::try_from(pcm.len()).map_err(|_| SpeechSynthesisError::InvalidResponse)?;
        let riff_size = data_size
            .checked_add(36)
            .ok_or(SpeechSynthesisError::InvalidResponse)?;
        let mut wav = Vec::with_capacity(44 + pcm.len());
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&riff_size.to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&24_000_u32.to_le_bytes());
        wav.extend_from_slice(&48_000_u32.to_le_bytes());
        wav.extend_from_slice(&2_u16.to_le_bytes());
        wav.extend_from_slice(&16_u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        wav.extend_from_slice(pcm);
        Ok(wav)
    }

    async fn synthesize_gemini(
        &self,
        request: &SpeechSynthesisRequest,
    ) -> Result<SpeechSynthesisResult, SpeechSynthesisError> {
        let body = GeminiRequest {
            contents: [GeminiContent {
                parts: [GeminiPart {
                    text: &request.instructions,
                }],
            }],
            generation_config: GeminiGenerationConfig {
                response_modalities: ["AUDIO"],
                speech_config: GeminiSpeechConfig {
                    voice_config: GeminiVoiceConfig {
                        prebuilt_voice_config: GeminiPrebuiltVoiceConfig {
                            voice_name: GEMINI_VOICE,
                        },
                    },
                    language_code: &request.language,
                },
            },
        };

        for attempt in 0..MAX_ATTEMPTS {
            let response = self
                .client
                .post(&self.gemini_endpoint)
                .header("x-goog-api-key", &request.api_key)
                .timeout(REQUEST_TIMEOUT)
                .json(&body)
                .send()
                .await
                .map_err(|error| SpeechSynthesisError::Provider(error.to_string()))?;
            if response.status().is_server_error() && attempt + 1 < MAX_ATTEMPTS {
                Self::retry_delay(attempt).await;
                continue;
            }
            if !response.status().is_success() {
                return Err(Self::response_error(response).await);
            }
            let response: GeminiResponse =
                serde_json::from_slice(&Self::bounded_body(response, MAX_JSON_BYTES).await?)
                    .map_err(|_| SpeechSynthesisError::InvalidResponse)?;
            let encoded = response
                .candidates
                .first()
                .and_then(|candidate| {
                    candidate
                        .content
                        .parts
                        .iter()
                        .find_map(|part| part.inline_data.as_ref())
                })
                .map(|inline| inline.data.as_str())
                .ok_or(SpeechSynthesisError::InvalidResponse)?;
            let pcm = STANDARD
                .decode(encoded)
                .map_err(|_| SpeechSynthesisError::InvalidResponse)?;
            return Ok(SpeechSynthesisResult {
                media_type: "audio/wav".to_string(),
                bytes: Self::pcm_to_wav(&pcm)?,
            });
        }
        Err(SpeechSynthesisError::Provider(
            "speech provider retry limit reached".to_string(),
        ))
    }

    async fn synthesize_openai(
        &self,
        request: &SpeechSynthesisRequest,
    ) -> Result<SpeechSynthesisResult, SpeechSynthesisError> {
        let body = OpenAiRequest {
            model: OPENAI_MODEL,
            voice: OPENAI_VOICE,
            input: &request.transcript,
            instructions: &request.instructions,
            response_format: "wav",
        };
        for attempt in 0..MAX_ATTEMPTS {
            let response = self
                .client
                .post(&self.openai_endpoint)
                .bearer_auth(&request.api_key)
                .timeout(REQUEST_TIMEOUT)
                .json(&body)
                .send()
                .await
                .map_err(|error| SpeechSynthesisError::Provider(error.to_string()))?;
            if response.status().is_server_error() && attempt + 1 < MAX_ATTEMPTS {
                Self::retry_delay(attempt).await;
                continue;
            }
            if !response.status().is_success() {
                return Err(Self::response_error(response).await);
            }
            let bytes = Self::bounded_body(response, MAX_AUDIO_BYTES).await?;
            if !Self::looks_like_wav(&bytes) {
                return Err(SpeechSynthesisError::InvalidResponse);
            }
            return Ok(SpeechSynthesisResult {
                media_type: "audio/wav".to_string(),
                bytes,
            });
        }
        Err(SpeechSynthesisError::Provider(
            "speech provider retry limit reached".to_string(),
        ))
    }

    fn looks_like_wav(bytes: &[u8]) -> bool {
        bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
    }
}

#[async_trait]
impl SpeechSynthesizer for AiSpeechSynthesizer {
    fn identity(&self, provider: &str) -> Result<SpeechSynthesisIdentity, SpeechSynthesisError> {
        match provider {
            "gemini" => Ok(SpeechSynthesisIdentity {
                model: GEMINI_MODEL.to_string(),
                voice: GEMINI_VOICE.to_string(),
            }),
            "openai" => Ok(SpeechSynthesisIdentity {
                model: OPENAI_MODEL.to_string(),
                voice: OPENAI_VOICE.to_string(),
            }),
            _ => Err(SpeechSynthesisError::UnsupportedProvider),
        }
    }

    async fn synthesize(
        &self,
        request: SpeechSynthesisRequest,
    ) -> Result<SpeechSynthesisResult, SpeechSynthesisError> {
        match request.provider.as_str() {
            "gemini" => self.synthesize_gemini(&request).await,
            "openai" => self.synthesize_openai(&request).await,
            _ => Err(SpeechSynthesisError::UnsupportedProvider),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc::{self, Receiver},
        thread,
    };

    use super::*;

    fn http_response(status: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(body);
        response
    }

    fn serve(responses: Vec<Vec<u8>>) -> (String, Receiver<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = Vec::new();
                let mut buffer = [0_u8; 4096];
                let mut expected_length = None;
                loop {
                    let read = stream.read(&mut buffer).unwrap();
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    if expected_length.is_none()
                        && let Some(header_end) =
                            request.windows(4).position(|window| window == b"\r\n\r\n")
                    {
                        let headers = String::from_utf8_lossy(&request[..header_end]);
                        let content_length = headers
                            .lines()
                            .find_map(|line| {
                                let (name, value) = line.split_once(':')?;
                                name.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().unwrap())
                            })
                            .unwrap_or(0);
                        expected_length = Some(header_end + 4 + content_length);
                    }
                    if expected_length.is_some_and(|length| request.len() >= length) {
                        break;
                    }
                }
                sender.send(request).unwrap();
                stream.write_all(&response).unwrap();
            }
        });
        (endpoint, receiver)
    }

    fn request(provider: &str) -> SpeechSynthesisRequest {
        SpeechSynthesisRequest {
            provider: provider.to_string(),
            api_key: "test-key".to_string(),
            language: "ja-JP".to_string(),
            transcript: "橋".to_string(),
            instructions: "Speak 橋 once.".to_string(),
        }
    }

    #[test]
    fn exposes_fixed_provider_models_and_voices() {
        let synthesizer = AiSpeechSynthesizer::default();
        assert_eq!(
            synthesizer.identity("gemini").unwrap(),
            SpeechSynthesisIdentity {
                model: GEMINI_MODEL.to_string(),
                voice: GEMINI_VOICE.to_string(),
            }
        );
        assert_eq!(
            synthesizer.identity("openai").unwrap(),
            SpeechSynthesisIdentity {
                model: OPENAI_MODEL.to_string(),
                voice: OPENAI_VOICE.to_string(),
            }
        );
        assert_eq!(
            synthesizer.identity("unknown"),
            Err(SpeechSynthesisError::UnsupportedProvider)
        );
    }

    #[test]
    fn wraps_gemini_pcm_in_a_valid_wav_header() {
        let wav = AiSpeechSynthesizer::pcm_to_wav(&[0, 0, 1, 0]).unwrap();
        assert!(AiSpeechSynthesizer::looks_like_wav(&wav));
        assert_eq!(&wav[40..44], &4_u32.to_le_bytes());
        assert_eq!(&wav[44..], &[0, 0, 1, 0]);
    }

    #[test]
    fn rejects_empty_or_misaligned_pcm() {
        assert_eq!(
            AiSpeechSynthesizer::pcm_to_wav(&[]),
            Err(SpeechSynthesisError::InvalidResponse)
        );
        assert_eq!(
            AiSpeechSynthesizer::pcm_to_wav(&[0]),
            Err(SpeechSynthesisError::InvalidResponse)
        );
        assert_eq!(
            AiSpeechSynthesizer::pcm_to_wav(&vec![0; MAX_AUDIO_BYTES]),
            Err(SpeechSynthesisError::InvalidResponse)
        );
    }

    #[tokio::test]
    async fn sends_gemini_speech_configuration_and_wraps_pcm() {
        let body =
            br#"{"candidates":[{"content":{"parts":[{"inlineData":{"data":"AAABAA=="}}]}}]}"#;
        let (endpoint, requests) = serve(vec![http_response("200 OK", "application/json", body)]);
        let synthesizer = AiSpeechSynthesizer {
            client: reqwest::Client::new(),
            gemini_endpoint: endpoint,
            openai_endpoint: String::new(),
        };

        let result = synthesizer.synthesize(request("gemini")).await.unwrap();
        assert!(AiSpeechSynthesizer::looks_like_wav(&result.bytes));
        let request = String::from_utf8_lossy(&requests.recv().unwrap()).to_string();
        assert!(request.contains("x-goog-api-key: test-key"));
        assert!(request.contains(r#""responseModalities":["AUDIO"]"#));
        assert!(request.contains(r#""voiceName":"Iapetus""#));
        assert!(request.contains(r#""languageCode":"ja-JP""#));
    }

    #[tokio::test]
    async fn sends_openai_wav_request_and_retries_server_errors() {
        let wav = AiSpeechSynthesizer::pcm_to_wav(&[0, 0, 1, 0]).unwrap();
        let (endpoint, requests) = serve(vec![
            http_response("503 Service Unavailable", "text/plain", b"temporary"),
            http_response("200 OK", "audio/wav", &wav),
        ]);
        let synthesizer = AiSpeechSynthesizer {
            client: reqwest::Client::new(),
            gemini_endpoint: String::new(),
            openai_endpoint: endpoint,
        };

        assert_eq!(
            synthesizer
                .synthesize(request("openai"))
                .await
                .unwrap()
                .bytes,
            wav
        );
        let first = String::from_utf8_lossy(&requests.recv().unwrap()).to_string();
        let second = String::from_utf8_lossy(&requests.recv().unwrap()).to_string();
        for request in [first, second] {
            assert!(request.contains("authorization: Bearer test-key"));
            assert!(request.contains(r#""model":"gpt-4o-mini-tts""#));
            assert!(request.contains(r#""voice":"marin""#));
            assert!(request.contains(r#""response_format":"wav""#));
        }
    }
}
