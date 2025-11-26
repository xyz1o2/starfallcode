/// 高级 LLM 客户端 - 支持多提供商、重试、速率限制等
use crate::ai::client::{LLMClient, LLMConfig as ClientConfig};
use crate::ai::config::{LLMConfig, LLMProvider};
use std::time::Duration;
use tokio::time::sleep;

/// 高级客户端配置
#[derive(Clone)]
pub struct AdvancedClientConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub timeout_secs: u64,
}

impl Default for AdvancedClientConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
            timeout_secs: 300,
        }
    }
}

/// 高级 LLM 客户端
pub struct AdvancedLLMClient {
    client: LLMClient,
    config: LLMConfig,
    advanced_config: AdvancedClientConfig,
}

impl AdvancedLLMClient {
    /// 创建新的高级客户端
    pub fn new(config: LLMConfig) -> Self {
        Self::with_advanced_config(config, AdvancedClientConfig::default())
    }

    /// 使用自定义配置创建高级客户端
    pub fn with_advanced_config(config: LLMConfig, advanced_config: AdvancedClientConfig) -> Self {
        let client_config = ClientConfig {
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        };

        let client = LLMClient::new(client_config);

        Self {
            client,
            config,
            advanced_config,
        }
    }

    /// 带重试的聊天完成
    pub async fn generate_completion_with_retry(&self, prompt: &str) -> Result<String, String> {
        let mut last_error = String::new();

        for attempt in 0..=self.advanced_config.max_retries {
            match self.client.generate_completion(prompt).await {
                Ok(response) => {
                    if attempt > 0 {
                        eprintln!("✓ 重试成功 (尝试 {})", attempt + 1);
                    }
                    return Ok(response);
                }
                Err(e) => {
                    last_error = e.clone();

                    if attempt < self.advanced_config.max_retries {
                        let delay = self.advanced_config.retry_delay_ms * (2_u64.pow(attempt));
                        eprintln!(
                            "⚠ 请求失败 (尝试 {}): {}. 在 {}ms 后重试...",
                            attempt + 1,
                            e,
                            delay
                        );
                        sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(format!(
            "在 {} 次尝试后失败: {}",
            self.advanced_config.max_retries + 1,
            last_error
        ))
    }

    /// 带重试的流式聊天完成
    pub async fn generate_completion_stream_with_retry(
        &self,
        prompt: &str,
        callback: impl Fn(String) -> bool,
    ) -> Result<(), String> {
        let mut last_error = String::new();

        for attempt in 0..=self.advanced_config.max_retries {
            match self.client.generate_completion_stream(prompt, &callback).await {
                Ok(_) => {
                    if attempt > 0 {
                        eprintln!("✓ 流式请求成功 (尝试 {})", attempt + 1);
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = e.clone();

                    if attempt < self.advanced_config.max_retries {
                        let delay = self.advanced_config.retry_delay_ms * (2_u64.pow(attempt));
                        eprintln!(
                            "⚠ 流式请求失败 (尝试 {}): {}. 在 {}ms 后重试...",
                            attempt + 1,
                            e,
                            delay
                        );
                        sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(format!(
            "流式请求在 {} 次尝试后失败: {}",
            self.advanced_config.max_retries + 1,
            last_error
        ))
    }

    /// 获取提供商信息
    pub fn provider(&self) -> &LLMProvider {
        &self.config.provider
    }

    /// 获取模型信息
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// 获取温度设置
    pub fn temperature(&self) -> f32 {
        self.config.temperature
    }

    /// 获取最大令牌数
    pub fn max_tokens(&self) -> u32 {
        self.config.max_tokens
    }

    /// 获取配置摘要
    pub fn config_summary(&self) -> String {
        format!(
            "Provider: {}\nModel: {}\nTemperature: {}\nMax Tokens: {}",
            self.config.provider.to_string(),
            self.config.model,
            self.config.temperature,
            self.config.max_tokens
        )
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.config.api_key.is_empty() && self.config.provider != LLMProvider::Ollama {
            return Err("API key is required for this provider".to_string());
        }

        if self.config.model.is_empty() {
            return Err("Model name is required".to_string());
        }

        if self.config.base_url.is_empty() {
            return Err("Base URL is required".to_string());
        }

        if self.config.temperature < 0.0 || self.config.temperature > 2.0 {
            return Err("Temperature must be between 0.0 and 2.0".to_string());
        }

        if self.config.max_tokens == 0 {
            return Err("Max tokens must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_client_creation() {
        let config = LLMConfig::default_openai("test-key".to_string());
        let client = AdvancedLLMClient::new(config);
        assert_eq!(client.model(), "gpt-3.5-turbo");
    }

    #[test]
    fn test_config_validation() {
        let config = LLMConfig::default_openai("test-key".to_string());
        let client = AdvancedLLMClient::new(config);
        assert!(client.validate().is_ok());
    }

    #[test]
    fn test_invalid_temperature() {
        let mut config = LLMConfig::default_openai("test-key".to_string());
        config.temperature = 3.0; // Invalid
        let client = AdvancedLLMClient::new(config);
        assert!(client.validate().is_err());
    }
}
