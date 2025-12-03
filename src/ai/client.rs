use crate::ai::config::LLMConfig;
use crate::tools::ToolDefinition;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinitionForLLM>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
struct ToolDefinitionForLLM {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionDefinition,
}

#[derive(Debug, Serialize, Clone)]
struct FunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
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

#[derive(Debug, Deserialize)]
struct NonStreamingResponse {
    choices: Vec<ResponseChoice>,
}

#[derive(Debug, Deserialize)]
struct ResponseChoice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    role: String,
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

    /// 生成非流式响应（支持工具调用）
    pub async fn generate_completion(
        &self,
        messages: Vec<ChatMessage>,
        model_override: Option<String>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let has_tools = tools.is_some();

        // 转换工具定义格式
        let tools_for_llm = tools.map(|defs| {
            defs.iter()
                .map(|def| ToolDefinitionForLLM {
                    tool_type: "function".to_string(),
                    function: FunctionDefinition {
                        name: def.name.clone(),
                        description: def.description.clone(),
                        parameters: self.convert_parameters(&def.parameters),
                    },
                })
                .collect()
        });

        let request_body = ChatCompletionRequest {
            model: model_override.unwrap_or_else(|| self.config.model.clone()),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: false,
            tools: tools_for_llm,
            tool_choice: if has_tools {
                Some("auto".to_string())
            } else {
                None
            },
        };

        let response = self
            .client
            .post(&self.config.base_url)
            .json(&request_body)
            .send()
            .await?;

        let response_text = response.text().await?;
        println!("LLM Response: {}", response_text);

        // 解析响应
        if let Ok(parsed) = serde_json::from_str::<NonStreamingResponse>(&response_text) {
            if let Some(choice) = parsed.choices.get(0) {
                return Ok(choice.message.content.clone().unwrap_or_default());
            }
        }

        Ok(response_text)
    }

    /// 生成流式响应
    pub async fn generate_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
        model_override: Option<String>,
        mut callback: impl FnMut(String) -> bool + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request_body = ChatCompletionRequest {
            model: model_override.unwrap_or_else(|| self.config.model.clone()),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: true,
            tools: None,
            tool_choice: None,
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

    /// 转换工具参数到 JSON Schema 格式
    fn convert_parameters(&self, parameters: &[crate::tools::ToolParameter]) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required_params = Vec::new();

        for param in parameters {
            let param_type = match param.param_type.as_str() {
                "string" => "string",
                "number" => "number",
                "boolean" => "boolean",
                "array" => "array",
                "object" => "object",
                _ => "string",
            };

            let param_schema = serde_json::json!({
                "type": param_type,
                "description": param.description,
            });

            properties.insert(param.name.clone(), param_schema);

            if param.required {
                required_params.push(param.name.clone());
            }
        }

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required_params,
        })
    }
}
