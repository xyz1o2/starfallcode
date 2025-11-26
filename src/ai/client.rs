use reqwest;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct LLMConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub temperature: f32,
    pub max_tokens: u32,
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
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
    index: u32,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct StreamChunkData {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Option<Delta>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

#[derive(Clone)]
pub struct LLMClient {
    client: reqwest::Client,
    config: LLMConfig,
}

impl LLMClient {
    /// 创建新的 LLM 客户端
    pub fn new(config: LLMConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self { client, config }
    }

    /// 构建请求头
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        
        if let Ok(auth_value) = format!("Bearer {}", self.config.api_key).parse() {
            headers.insert("Authorization", auth_value);
        }
        
        if let Ok(content_type) = "application/json".parse() {
            headers.insert("Content-Type", content_type);
        }
        
        headers
    }

    /// 非流式聊天完成
    pub async fn generate_completion(&self, prompt: &str) -> Result<String, String> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
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
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API error ({}): {}", status, error_text));
        }

        let response_data: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        response_data
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| "No choices in response".to_string())
    }

    /// 流式聊天完成 - 支持 SSE 格式
    pub async fn generate_completion_stream(
        &self,
        prompt: &str,
        callback: impl Fn(String) -> bool, // Return true to continue, false to stop
    ) -> Result<(), String> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
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
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API error ({}): {}", status, error_text));
        }

        // 处理流式响应
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| format!("Failed to read chunk: {}", e))?
        {
            let text = String::from_utf8(chunk.to_vec())
                .map_err(|e| format!("Failed to parse UTF-8: {}", e))?;

            // 解析 SSE 格式响应
            for line in text.lines() {
                if line.is_empty() {
                    continue;
                }

                if line.starts_with("data: ") {
                    let data = &line[6..]; // 移除 "data: " 前缀

                    if data == "[DONE]" {
                        return Ok(());
                    }

                    // 尝试解析为 JSON
                    match serde_json::from_str::<StreamChunkData>(data) {
                        Ok(chunk_data) => {
                            for choice in chunk_data.choices {
                                if let Some(delta) = choice.delta {
                                    if let Some(content) = delta.content {
                                        if !callback(content) {
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // 忽略解析失败的行
                            continue;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取当前配置
    pub fn config(&self) -> &LLMConfig {
        &self.config
    }

    /// 更新模型
    pub fn set_model(&mut self, model: String) {
        self.config.model = model;
    }

    /// 更新温度参数
    pub fn set_temperature(&mut self, temperature: f32) {
        self.config.temperature = temperature.clamp(0.0, 2.0);
    }

    /// 更新最大令牌数
    pub fn set_max_tokens(&mut self, max_tokens: u32) {
        self.config.max_tokens = max_tokens;
    }
}