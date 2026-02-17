use futures_util::StreamExt;
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
        let request = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: system.to_string(),
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama returned status: {}", response.status()));
        }

        let mut full_text = String::new();
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            // Process complete NDJSON lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

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
                            let _ = channel.send(StoryToken {
                                token: String::new(),
                                done: true,
                            });
                        }
                    }
                    Err(_) => {
                        // Skip malformed JSON lines
                        continue;
                    }
                }
            }
        }

        Ok(full_text)
    }

    /// Non-streaming generation for summaries
    pub async fn generate(&self, model: &str, system: &str, prompt: &str) -> Result<String, String> {
        let request = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: system.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
        }

        let result: OllamaResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(result.response)
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
