use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
pub struct LLMConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct LLMRequest {
    model: String,
    messages: Vec<LLMMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct LLMMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LLMResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: LLMMessage,
    index: u32,
    finish_reason: Option<String>,
}

pub struct LLMClient {
    client: reqwest::Client,
    config: LLMConfig,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn generate_completion(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = LLMRequest {
            model: self.config.model.clone(),
            messages: vec![LLMMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: false,
        };

        let response = self
            .client
            .post(&self.config.base_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let llm_response: LLMResponse = response.json().await?;
            if let Some(choice) = llm_response.choices.first() {
                Ok(choice.message.content.clone())
            } else {
                Err("No choices in response".into())
            }
        } else {
            Err(format!("Request failed: {}", response.status()).into())
        }
    }

    pub async fn generate_completion_stream(
        &self,
        prompt: &str,
        callback: impl Fn(String) -> bool, // Return true to continue, false to stop
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = LLMRequest {
            model: self.config.model.clone(),
            messages: vec![LLMMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: true,
        };

        let mut response = self
            .client
            .post(&self.config.base_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            while let Some(chunk) = response.chunk().await? {
                let text = String::from_utf8(chunk.to_vec())?;
                
                // Parse the SSE response format
                for line in text.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..]; // Remove "data: " prefix
                        if data == "[DONE]" {
                            break;
                        }
                        
                        // Try to parse as JSON response
                        if let Ok(llm_response) = serde_json::from_str::<LLMResponse>(data) {
                            if let Some(choice) = llm_response.choices.first() {
                                if !callback(choice.message.content.clone()) {
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        } else {
            Err(format!("Request failed: {}", response.status()).into())
        }
    }
}