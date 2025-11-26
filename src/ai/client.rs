use crate::ai::config::LLMConfig;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct LLMClient {
    client: reqwest::Client,
    config: LLMConfig,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct StreamChunkData {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Option<Delta>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        let mut headers = HeaderMap::new();
        if !config.api_key.is_empty() {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", config.api_key)).unwrap(),
            );
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap();

        Self { client, config }
    }

    pub async fn generate_completion_stream(
        &self,
        prompt: &str,
        mut callback: impl FnMut(String) -> bool + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request_body = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: true,
        };

        let mut stream = self
            .client
            .post(&self.config.base_url)
            .json(&request_body)
            .send()
            .await?
            .bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            let chunk_str = String::from_utf8(chunk.to_vec())?;

            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        return Ok(());
                    }

                    if let Ok(stream_chunk) = serde_json::from_str::<StreamChunkData>(data) {
                        if let Some(choice) = stream_chunk.choices.get(0) {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    if !callback(content.clone()) {
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}