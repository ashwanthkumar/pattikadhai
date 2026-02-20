use futures_util::StreamExt;
use log;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub system: String,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStreamChunk {
    pub response: Option<String>,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoryToken {
    pub token: String,
    pub done: bool,
}

pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn generate_streaming(
        &self,
        model: &str,
        system: &str,
        prompt: &str,
        channel: &Channel<StoryToken>,
    ) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);
        log::info!(
            "Ollama streaming request: model={}, prompt_len={}, system_len={}",
            model,
            prompt.len(),
            system.len()
        );

        let request = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: system.to_string(),
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("Ollama connection failed: {}", e);
                format!("Failed to connect to Ollama: {}", e)
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            log::error!("Ollama returned status {}: {}", status, body);
            return Err(format!(
                "Ollama returned status {}: {}",
                status,
                body.chars().take(500).collect::<String>()
            ));
        }

        log::debug!("Ollama streaming response started (status {})", status);

        let mut full_text = String::new();
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut chunk_count: u32 = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| {
                log::error!("Ollama stream error after {} chunks: {}", chunk_count, e);
                format!("Stream error after {} chunks: {}", chunk_count, e)
            })?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            // Process complete NDJSON lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                chunk_count += 1;
                match serde_json::from_str::<OllamaStreamChunk>(&line) {
                    Ok(parsed) => {
                        if let Some(ref token) = parsed.response {
                            full_text.push_str(token);
                            let _ = channel.send(StoryToken {
                                token: token.clone(),
                                done: false,
                            });
                        }
                        if parsed.done {
                            log::info!(
                                "Ollama streaming complete: {} chunks, {} chars generated",
                                chunk_count,
                                full_text.len()
                            );
                            let _ = channel.send(StoryToken {
                                token: String::new(),
                                done: true,
                            });
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "Ollama malformed JSON chunk #{}: {} (line: {})",
                            chunk_count,
                            e,
                            line.chars().take(200).collect::<String>()
                        );
                        continue;
                    }
                }
            }
        }

        if full_text.is_empty() {
            log::warn!("Ollama streaming produced empty response after {} chunks", chunk_count);
        }

        Ok(full_text)
    }

    /// Streaming generation that collects text internally (no channel).
    /// Used for summaries and other internal calls.
    pub async fn generate(&self, model: &str, system: &str, prompt: &str) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);
        log::info!(
            "Ollama generate request: model={}, prompt_len={}, system_len={}",
            model,
            prompt.len(),
            system.len()
        );

        let request = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: system.to_string(),
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("Ollama connection failed: {}", e);
                format!("Failed to connect to Ollama: {}", e)
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            log::error!("Ollama returned status {}: {}", status, body);
            return Err(format!(
                "Ollama returned status {}: {}",
                status,
                body.chars().take(500).collect::<String>()
            ));
        }

        let mut full_text = String::new();
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut chunk_count: u32 = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| {
                log::error!("Ollama stream error after {} chunks: {}", chunk_count, e);
                format!("Stream error after {} chunks: {}", chunk_count, e)
            })?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                chunk_count += 1;
                match serde_json::from_str::<OllamaStreamChunk>(&line) {
                    Ok(parsed) => {
                        if let Some(ref token) = parsed.response {
                            full_text.push_str(token);
                        }
                        if parsed.done {
                            log::info!(
                                "Ollama generate complete: {} chunks, {} chars",
                                chunk_count,
                                full_text.len()
                            );
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "Ollama malformed JSON chunk #{}: {} (line: {})",
                            chunk_count,
                            e,
                            line.chars().take(200).collect::<String>()
                        );
                    }
                }
            }
        }

        if full_text.is_empty() {
            log::warn!("Ollama generate produced empty response after {} chunks", chunk_count);
        }

        Ok(full_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ollama_stream_chunk() {
        let json = r#"{"response":"Hello","done":false}"#;
        let chunk: OllamaStreamChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.response.unwrap(), "Hello");
        assert!(!chunk.done);
    }

    #[test]
    fn parse_ollama_done_chunk() {
        let json = r#"{"response":"","done":true}"#;
        let chunk: OllamaStreamChunk = serde_json::from_str(json).unwrap();
        assert!(chunk.done);
    }

    #[test]
    fn parse_malformed_json_gracefully() {
        let json = r#"{"broken"#;
        let result = serde_json::from_str::<OllamaStreamChunk>(json);
        assert!(result.is_err());
    }

    #[test]
    fn parse_chunk_with_missing_response() {
        let json = r#"{"done":true}"#;
        let chunk: OllamaStreamChunk = serde_json::from_str(json).unwrap();
        assert!(chunk.response.is_none());
        assert!(chunk.done);
    }
}
