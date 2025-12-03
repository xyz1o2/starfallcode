use crate::types::{GrokMessage, GrokTool, GrokToolCall, GrokToolCallFunction};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use futures::Stream;

#[derive(Debug)]
pub struct GrokClient {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub is_openai_compatible: bool,
    http_client: reqwest::Client,
    pub default_max_tokens: u32,
}

impl Clone for GrokClient {
    fn clone(&self) -> Self {
        Self {
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            is_openai_compatible: self.is_openai_compatible,
            http_client: reqwest::Client::new(), // Create a new client since reqwest::Client doesn't implement Clone
            default_max_tokens: self.default_max_tokens,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GrokResponse {
    pub choices: Vec<GrokChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GrokChoice {
    pub message: GrokMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_parameters: Option<SearchParameters>,
}

impl GrokClient {
    pub fn new(api_key: &str, model: Option<String>, base_url: Option<String>, is_openai_compatible: Option<bool>) -> Self {
        let default_max_tokens = std::env::var("GROK_MAX_TOKENS")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(1536);

        // Create HTTP client with timeout
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            api_key: api_key.to_string(),
            base_url: base_url.unwrap_or_else(|| "https://api.x.ai/v1".to_string()),
            model: model.unwrap_or_else(|| "grok-code-fast-1".to_string()),
            is_openai_compatible: is_openai_compatible.unwrap_or(false),
            http_client,
            default_max_tokens,
        }
    }

    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    pub fn get_current_model(&self) -> &str {
        &self.model
    }

    pub async fn chat(
        &self,
        messages: Vec<GrokMessage>,
        tools: Option<Vec<GrokTool>>,
        model: Option<String>,
        search_options: Option<SearchOptions>,
    ) -> Result<GrokResponse, Box<dyn std::error::Error>> {
        // Check if we have a valid API key
        if self.api_key == "API_KEY_NOT_SET" {
            return Err("No API key set. Please configure your API key.".into());
        }

        let request_payload = self.create_request_payload(
            model.as_deref().unwrap_or(&self.model),
            messages,
            tools,
            search_options,
        );

        // Retry logic with exponential backoff
        let mut retries = 0;
        let max_retries = 3;
        
        loop {
            match self
                .http_client
                .post(format!("{}/chat/completions", self.base_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request_payload)
                .send()
                .await
            {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        
                        // Retry on server errors (5xx) and rate limits (429)
                        if (status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS) && retries < max_retries {
                            retries += 1;
                            let wait_time = std::time::Duration::from_secs(2_u64.pow(retries as u32));
                            eprintln!("⚠️  API error ({}). Retrying in {:?}...", status, wait_time);
                            tokio::time::sleep(wait_time).await;
                            continue;
                        }
                        
                        return Err(format!("Grok API error ({}): {}", status, error_text).into());
                    }

                    let response: GrokResponse = response.json().await?;
                    return Ok(response);
                }
                Err(e) => {
                    // Retry on timeout and connection errors
                    if (e.is_timeout() || e.is_connect()) && retries < max_retries {
                        retries += 1;
                        let wait_time = std::time::Duration::from_secs(2_u64.pow(retries as u32));
                        eprintln!("⚠️  Connection error: {}. Retrying in {:?}...", e, wait_time);
                        tokio::time::sleep(wait_time).await;
                        continue;
                    }
                    
                    return Err(Box::new(e));
                }
            }
        }
    }

    pub async fn chat_stream(
        &self,
        messages: Vec<GrokMessage>,
        tools: Option<Vec<GrokTool>>,
        model: Option<String>,
        search_options: Option<SearchOptions>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value, Box<dyn std::error::Error + Send>>> + Send>>, Box<dyn std::error::Error + Send>> {
        // For now, return an error as streaming implementation requires external dependencies
        // like reqwest-eventsource which may have compatibility issues in this context
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Streaming not fully implemented yet")))
    }

    pub async fn search(
        &self,
        query: &str,
        search_parameters: Option<SearchParameters>,
    ) -> Result<GrokResponse, Box<dyn std::error::Error>> {
        // Check if we have a valid API key
        if self.api_key == "API_KEY_NOT_SET" {
            return Err("No API key set. Please configure your API key.".into());
        }

        let search_message = GrokMessage {
            role: "user".to_string(),
            content: Some(query.to_string()),
            tool_calls: None,
            tool_call_id: None,
        };

        self.chat(
            vec![search_message],
            None,
            None,
            Some(SearchOptions {
                search_parameters,
            }),
        )
        .await
    }

    fn create_request_payload(
        &self,
        model: &str,
        messages: Vec<GrokMessage>,
        tools: Option<Vec<GrokTool>>,
        search_options: Option<SearchOptions>,
    ) -> serde_json::Value {
        let mut payload = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": self.default_max_tokens,
        });

        if let Some(tool_list) = tools {
            if !tool_list.is_empty() {
                payload["tools"] = serde_json::to_value(tool_list).unwrap();
                payload["tool_choice"] = serde_json::Value::String("auto".to_string());
            }
        }

        if !self.is_openai_compatible {
            // Add Grok-specific parameters
            if let Some(search_opts) = search_options {
                if let Some(search_params) = search_opts.search_parameters {
                    payload["search_parameters"] = serde_json::to_value(search_params).unwrap();
                }
            }
        }

        payload
    }
}