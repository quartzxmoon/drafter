// Speech Recognition Service
// Voice-to-text functionality for legal dictation

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn, error};
use reqwest::Client;
use tokio::fs;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeechRecognitionConfig {
    pub provider: SpeechProvider,
    pub language: String,
    pub sample_rate: u32,
    pub encoding: AudioEncoding,
    pub enable_automatic_punctuation: bool,
    pub enable_word_time_offsets: bool,
    pub enable_speaker_diarization: bool,
    pub diarization_speaker_count: Option<u32>,
    pub profanity_filter: bool,
    pub speech_contexts: Vec<SpeechContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SpeechProvider {
    GoogleCloud,
    AzureCognitive,
    AmazonTranscribe,
    OpenAIWhisper,
    AssemblyAI,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AudioEncoding {
    Linear16,
    Flac,
    Mulaw,
    Amr,
    AmrWb,
    OggOpus,
    SpeeXWithHeaderByte,
    Mp3,
    Wav,
    WebmOpus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeechContext {
    pub phrases: Vec<String>,
    pub boost: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub transcript: String,
    pub confidence: f32,
    pub words: Vec<WordInfo>,
    pub speaker_tags: Vec<SpeakerTag>,
    pub language_code: String,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordInfo {
    pub word: String,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub confidence: f32,
    pub speaker_tag: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeakerTag {
    pub speaker_id: u32,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegalDictationSettings {
    pub auto_capitalize_legal_terms: bool,
    pub legal_vocabulary: Vec<String>,
    pub citation_formatting: bool,
    pub paragraph_detection: bool,
    pub punctuation_commands: bool,
    pub custom_commands: HashMap<String, String>,
}

pub struct SpeechRecognitionService {
    config: SpeechRecognitionConfig,
    legal_settings: LegalDictationSettings,
    client: Client,
    api_keys: HashMap<String, String>,
}

impl SpeechRecognitionService {
    pub fn new(config: SpeechRecognitionConfig, legal_settings: LegalDictationSettings) -> Self {
        Self {
            config,
            legal_settings,
            client: Client::new(),
            api_keys: HashMap::new(),
        }
    }

    pub fn set_api_key(&mut self, provider: &str, api_key: String) {
        self.api_keys.insert(provider.to_string(), api_key);
    }

    pub async fn transcribe_audio_file(&self, audio_path: &Path) -> Result<TranscriptionResult> {
        let audio_data = fs::read(audio_path).await?;
        self.transcribe_audio_data(&audio_data).await
    }

    pub async fn transcribe_audio_data(&self, audio_data: &[u8]) -> Result<TranscriptionResult> {
        match self.config.provider {
            SpeechProvider::GoogleCloud => self.transcribe_google_cloud(audio_data).await,
            SpeechProvider::AzureCognitive => self.transcribe_azure(audio_data).await,
            SpeechProvider::OpenAIWhisper => self.transcribe_openai_whisper(audio_data).await,
            SpeechProvider::AssemblyAI => self.transcribe_assemblyai(audio_data).await,
            _ => Err(anyhow!("Provider not implemented")),
        }
    }

    async fn transcribe_google_cloud(&self, audio_data: &[u8]) -> Result<TranscriptionResult> {
        let api_key = self.api_keys.get("google_cloud")
            .ok_or_else(|| anyhow!("Google Cloud API key not set"))?;

        let audio_content = general_purpose::STANDARD.encode(audio_data);

        let request_body = serde_json::json!({
            "config": {
                "encoding": self.get_google_encoding(),
                "sampleRateHertz": self.config.sample_rate,
                "languageCode": self.config.language,
                "enableAutomaticPunctuation": self.config.enable_automatic_punctuation,
                "enableWordTimeOffsets": self.config.enable_word_time_offsets,
                "enableSpeakerDiarization": self.config.enable_speaker_diarization,
                "diarizationSpeakerCount": self.config.diarization_speaker_count,
                "profanityFilter": self.config.profanity_filter,
                "speechContexts": self.config.speech_contexts.iter().map(|ctx| {
                    serde_json::json!({
                        "phrases": ctx.phrases,
                        "boost": ctx.boost
                    })
                }).collect::<Vec<_>>()
            },
            "audio": {
                "content": audio_content
            }
        });

        let response = self.client
            .post(&format!("https://speech.googleapis.com/v1/speech:recognize?key={}", api_key))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google Cloud Speech API error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;
        self.parse_google_response(response_json)
    }

    async fn transcribe_openai_whisper(&self, audio_data: &[u8]) -> Result<TranscriptionResult> {
        let api_key = self.api_keys.get("openai")
            .ok_or_else(|| anyhow!("OpenAI API key not set"))?;

        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(audio_data.to_vec())
                .file_name("audio.wav")
                .mime_str("audio/wav")?)
            .text("model", "whisper-1")
            .text("language", self.config.language.clone())
            .text("response_format", "verbose_json")
            .text("timestamp_granularities[]", "word");

        let response = self.client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI Whisper API error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;
        self.parse_whisper_response(response_json)
    }

    async fn transcribe_assemblyai(&self, audio_data: &[u8]) -> Result<TranscriptionResult> {
        let api_key = self.api_keys.get("assemblyai")
            .ok_or_else(|| anyhow!("AssemblyAI API key not set"))?;

        // First, upload the audio file
        let upload_response = self.client
            .post("https://api.assemblyai.com/v2/upload")
            .header("authorization", api_key)
            .body(audio_data.to_vec())
            .send()
            .await?;

        let upload_result: serde_json::Value = upload_response.json().await?;
        let upload_url = upload_result["upload_url"].as_str()
            .ok_or_else(|| anyhow!("Failed to get upload URL"))?;

        // Then, request transcription
        let transcription_request = serde_json::json!({
            "audio_url": upload_url,
            "language_code": self.config.language,
            "punctuate": self.config.enable_automatic_punctuation,
            "format_text": true,
            "speaker_labels": self.config.enable_speaker_diarization,
            "speakers_expected": self.config.diarization_speaker_count,
            "word_boost": self.legal_settings.legal_vocabulary,
            "boost_param": "high"
        });

        let transcription_response = self.client
            .post("https://api.assemblyai.com/v2/transcript")
            .header("authorization", api_key)
            .json(&transcription_request)
            .send()
            .await?;

        let transcription_result: serde_json::Value = transcription_response.json().await?;
        let transcript_id = transcription_result["id"].as_str()
            .ok_or_else(|| anyhow!("Failed to get transcript ID"))?;

        // Poll for completion
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let status_response = self.client
                .get(&format!("https://api.assemblyai.com/v2/transcript/{}", transcript_id))
                .header("authorization", api_key)
                .send()
                .await?;

            let status_result: serde_json::Value = status_response.json().await?;
            let status = status_result["status"].as_str().unwrap_or("");

            match status {
                "completed" => return self.parse_assemblyai_response(status_result),
                "error" => return Err(anyhow!("Transcription failed: {}", 
                    status_result["error"].as_str().unwrap_or("Unknown error"))),
                _ => continue, // Still processing
            }
        }
    }

    async fn transcribe_azure(&self, audio_data: &[u8]) -> Result<TranscriptionResult> {
        // Azure Cognitive Services implementation
        let api_key = self.api_keys.get("azure")
            .ok_or_else(|| anyhow!("Azure API key not set"))?;
        
        let region = self.api_keys.get("azure_region")
            .ok_or_else(|| anyhow!("Azure region not set"))?;

        let response = self.client
            .post(&format!("https://{}.stt.speech.microsoft.com/speech/recognition/conversation/cognitiveservices/v1", region))
            .header("Ocp-Apim-Subscription-Key", api_key)
            .header("Content-Type", "audio/wav")
            .query(&[
                ("language", self.config.language.as_str()),
                ("format", "detailed"),
                ("profanity", if self.config.profanity_filter { "masked" } else { "raw" }),
            ])
            .body(audio_data.to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Azure Speech API error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;
        self.parse_azure_response(response_json)
    }

    fn parse_google_response(&self, response: serde_json::Value) -> Result<TranscriptionResult> {
        let results = response["results"].as_array()
            .ok_or_else(|| anyhow!("Invalid Google response format"))?;

        if results.is_empty() {
            return Ok(TranscriptionResult {
                transcript: String::new(),
                confidence: 0.0,
                words: vec![],
                speaker_tags: vec![],
                language_code: self.config.language.clone(),
                processing_time_ms: 0,
            });
        }

        let best_result = &results[0];
        let alternatives = best_result["alternatives"].as_array()
            .ok_or_else(|| anyhow!("No alternatives in response"))?;

        let best_alternative = &alternatives[0];
        let transcript = best_alternative["transcript"].as_str()
            .ok_or_else(|| anyhow!("No transcript in response"))?;

        let confidence = best_alternative["confidence"].as_f64().unwrap_or(0.0) as f32;

        let words = if let Some(words_array) = best_alternative["words"].as_array() {
            words_array.iter().map(|word| {
                WordInfo {
                    word: word["word"].as_str().unwrap_or("").to_string(),
                    start_time_ms: self.parse_duration(word["startTime"].as_str().unwrap_or("0s")),
                    end_time_ms: self.parse_duration(word["endTime"].as_str().unwrap_or("0s")),
                    confidence: word["confidence"].as_f64().unwrap_or(0.0) as f32,
                    speaker_tag: word["speakerTag"].as_u64().map(|t| t as u32),
                }
            }).collect()
        } else {
            vec![]
        };

        let processed_transcript = self.apply_legal_formatting(transcript);

        Ok(TranscriptionResult {
            transcript: processed_transcript,
            confidence,
            words,
            speaker_tags: vec![], // Would need to parse speaker diarization results
            language_code: self.config.language.clone(),
            processing_time_ms: 0,
        })
    }

    fn parse_whisper_response(&self, response: serde_json::Value) -> Result<TranscriptionResult> {
        let transcript = response["text"].as_str()
            .ok_or_else(|| anyhow!("No transcript in Whisper response"))?;

        let words = if let Some(words_array) = response["words"].as_array() {
            words_array.iter().map(|word| {
                WordInfo {
                    word: word["word"].as_str().unwrap_or("").to_string(),
                    start_time_ms: (word["start"].as_f64().unwrap_or(0.0) * 1000.0) as u64,
                    end_time_ms: (word["end"].as_f64().unwrap_or(0.0) * 1000.0) as u64,
                    confidence: 1.0, // Whisper doesn't provide word-level confidence
                    speaker_tag: None,
                }
            }).collect()
        } else {
            vec![]
        };

        let processed_transcript = self.apply_legal_formatting(transcript);

        Ok(TranscriptionResult {
            transcript: processed_transcript,
            confidence: 1.0, // Whisper doesn't provide overall confidence
            words,
            speaker_tags: vec![],
            language_code: response["language"].as_str().unwrap_or(&self.config.language).to_string(),
            processing_time_ms: 0,
        })
    }

    fn parse_assemblyai_response(&self, response: serde_json::Value) -> Result<TranscriptionResult> {
        let transcript = response["text"].as_str()
            .ok_or_else(|| anyhow!("No transcript in AssemblyAI response"))?;

        let confidence = response["confidence"].as_f64().unwrap_or(0.0) as f32;

        let words = if let Some(words_array) = response["words"].as_array() {
            words_array.iter().map(|word| {
                WordInfo {
                    word: word["text"].as_str().unwrap_or("").to_string(),
                    start_time_ms: word["start"].as_u64().unwrap_or(0),
                    end_time_ms: word["end"].as_u64().unwrap_or(0),
                    confidence: word["confidence"].as_f64().unwrap_or(0.0) as f32,
                    speaker_tag: word["speaker"].as_str().and_then(|s| s.parse().ok()),
                }
            }).collect()
        } else {
            vec![]
        };

        let processed_transcript = self.apply_legal_formatting(transcript);

        Ok(TranscriptionResult {
            transcript: processed_transcript,
            confidence,
            words,
            speaker_tags: vec![], // Would parse from utterances if available
            language_code: self.config.language.clone(),
            processing_time_ms: 0,
        })
    }

    fn parse_azure_response(&self, response: serde_json::Value) -> Result<TranscriptionResult> {
        let display_text = response["DisplayText"].as_str()
            .ok_or_else(|| anyhow!("No DisplayText in Azure response"))?;

        let confidence = response["Confidence"].as_f64().unwrap_or(0.0) as f32;

        let processed_transcript = self.apply_legal_formatting(display_text);

        Ok(TranscriptionResult {
            transcript: processed_transcript,
            confidence,
            words: vec![], // Azure detailed response would have word-level info
            speaker_tags: vec![],
            language_code: self.config.language.clone(),
            processing_time_ms: 0,
        })
    }

    fn apply_legal_formatting(&self, transcript: &str) -> String {
        let mut formatted = transcript.to_string();

        // Apply legal vocabulary capitalization
        if self.legal_settings.auto_capitalize_legal_terms {
            for term in &self.legal_settings.legal_vocabulary {
                let pattern = regex::Regex::new(&format!(r"\b{}\b", regex::escape(term))).unwrap();
                formatted = pattern.replace_all(&formatted, term).to_string();
            }
        }

        // Apply custom commands
        for (command, replacement) in &self.legal_settings.custom_commands {
            formatted = formatted.replace(command, replacement);
        }

        // Apply punctuation commands if enabled
        if self.legal_settings.punctuation_commands {
            formatted = formatted
                .replace(" period", ".")
                .replace(" comma", ",")
                .replace(" question mark", "?")
                .replace(" exclamation point", "!")
                .replace(" semicolon", ";")
                .replace(" colon", ":")
                .replace(" new paragraph", "\n\n")
                .replace(" new line", "\n");
        }

        formatted
    }

    fn get_google_encoding(&self) -> &str {
        match self.config.encoding {
            AudioEncoding::Linear16 => "LINEAR16",
            AudioEncoding::Flac => "FLAC",
            AudioEncoding::Mulaw => "MULAW",
            AudioEncoding::Mp3 => "MP3",
            AudioEncoding::Wav => "LINEAR16", // WAV is typically LINEAR16
            AudioEncoding::WebmOpus => "WEBM_OPUS",
            _ => "LINEAR16",
        }
    }

    fn parse_duration(&self, duration_str: &str) -> u64 {
        // Parse Google's duration format (e.g., "1.234s")
        if let Some(seconds_str) = duration_str.strip_suffix('s') {
            if let Ok(seconds) = seconds_str.parse::<f64>() {
                return (seconds * 1000.0) as u64;
            }
        }
        0
    }

    pub fn get_default_legal_vocabulary() -> Vec<String> {
        vec![
            "plaintiff".to_string(),
            "defendant".to_string(),
            "appellant".to_string(),
            "appellee".to_string(),
            "petitioner".to_string(),
            "respondent".to_string(),
            "deposition".to_string(),
            "interrogatories".to_string(),
            "subpoena".to_string(),
            "affidavit".to_string(),
            "motion".to_string(),
            "brief".to_string(),
            "complaint".to_string(),
            "answer".to_string(),
            "counterclaim".to_string(),
            "cross-claim".to_string(),
            "summary judgment".to_string(),
            "voir dire".to_string(),
            "habeas corpus".to_string(),
            "certiorari".to_string(),
            "mandamus".to_string(),
            "injunction".to_string(),
            "restraining order".to_string(),
            "discovery".to_string(),
            "jurisdiction".to_string(),
            "venue".to_string(),
            "statute of limitations".to_string(),
            "res judicata".to_string(),
            "collateral estoppel".to_string(),
            "prima facie".to_string(),
            "burden of proof".to_string(),
            "preponderance of evidence".to_string(),
            "beyond reasonable doubt".to_string(),
            "Federal Rules of Civil Procedure".to_string(),
            "Federal Rules of Evidence".to_string(),
        ]
    }
}

impl Default for SpeechRecognitionConfig {
    fn default() -> Self {
        Self {
            provider: SpeechProvider::OpenAIWhisper,
            language: "en-US".to_string(),
            sample_rate: 16000,
            encoding: AudioEncoding::Wav,
            enable_automatic_punctuation: true,
            enable_word_time_offsets: true,
            enable_speaker_diarization: false,
            diarization_speaker_count: None,
            profanity_filter: false,
            speech_contexts: vec![],
        }
    }
}

impl Default for LegalDictationSettings {
    fn default() -> Self {
        Self {
            auto_capitalize_legal_terms: true,
            legal_vocabulary: SpeechRecognitionService::get_default_legal_vocabulary(),
            citation_formatting: true,
            paragraph_detection: true,
            punctuation_commands: true,
            custom_commands: HashMap::new(),
        }
    }
}
