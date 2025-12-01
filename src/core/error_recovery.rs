/// 错误恢复模块 - 对应 Gemini CLI 的错误处理机制
/// 
/// 支持完善的错误恢复：
/// - 错误类型识别
/// - 自动恢复策略
/// - 模型降级
/// - 上下文压缩
/// - 历史管理

use std::collections::HashMap;

/// 可恢复的错误类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecoverableError {
    RateLimitExceeded,      // 速率限制
    TokenLimitExceeded,     // Token 限制
    ModelNotAvailable,      // 模型不可用
    NetworkError,           // 网络错误
    TimeoutError,           // 超时
    InvalidResponse,        // 无效响应
    PartialResponse,        // 部分响应
    ContextTooLarge,        // 上下文过大
    Unknown,                // 未知错误
}

impl RecoverableError {
    /// 从错误字符串识别错误类型
    pub fn from_string(error: &str) -> Self {
        if error.contains("rate limit") || error.contains("429") {
            RecoverableError::RateLimitExceeded
        } else if error.contains("token") || error.contains("context") {
            RecoverableError::TokenLimitExceeded
        } else if error.contains("model") || error.contains("404") {
            RecoverableError::ModelNotAvailable
        } else if error.contains("network") || error.contains("connection") {
            RecoverableError::NetworkError
        } else if error.contains("timeout") {
            RecoverableError::TimeoutError
        } else if error.contains("invalid") {
            RecoverableError::InvalidResponse
        } else if error.contains("partial") {
            RecoverableError::PartialResponse
        } else {
            RecoverableError::Unknown
        }
    }
}

/// 恢复策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStrategy {
    Retry,                  // 重试
    Fallback,               // 降级到备选模型
    ReduceContext,          // 减少上下文
    CompressHistory,        // 压缩历史
    SkipTools,              // 跳过工具调用
    Abort,                  // 中止
}

/// 恢复配置
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub max_recovery_attempts: u32,
    pub retry_delay_ms: u64,
    pub fallback_models: Vec<String>,
    pub context_reduction_factor: f32,
    pub enable_history_compression: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_recovery_attempts: 3,
            retry_delay_ms: 1000,
            fallback_models: vec![
                "gemini-2.0-flash".to_string(),
                "gemini-1.5-pro".to_string(),
            ],
            context_reduction_factor: 0.8,
            enable_history_compression: true,
        }
    }
}

/// 恢复结果
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub strategy_used: RecoveryStrategy,
    pub success: bool,
    pub attempts: u32,
    pub fallback_model: Option<String>,
    pub context_reduced: bool,
    pub history_compressed: bool,
}

/// 错误恢复器
pub struct ErrorRecovery {
    config: RecoveryConfig,
    error_handlers: HashMap<RecoverableError, Vec<RecoveryStrategy>>,
    recovery_history: Vec<RecoveryResult>,
}

impl ErrorRecovery {
    /// 创建新的错误恢复器
    pub fn new(config: RecoveryConfig) -> Self {
        let mut error_handlers = HashMap::new();
        
        // 配置错误处理策略
        error_handlers.insert(
            RecoverableError::RateLimitExceeded,
            vec![RecoveryStrategy::Retry, RecoveryStrategy::ReduceContext],
        );
        error_handlers.insert(
            RecoverableError::TokenLimitExceeded,
            vec![RecoveryStrategy::CompressHistory, RecoveryStrategy::ReduceContext],
        );
        error_handlers.insert(
            RecoverableError::ModelNotAvailable,
            vec![RecoveryStrategy::Fallback, RecoveryStrategy::Retry],
        );
        error_handlers.insert(
            RecoverableError::NetworkError,
            vec![RecoveryStrategy::Retry, RecoveryStrategy::Fallback],
        );
        error_handlers.insert(
            RecoverableError::TimeoutError,
            vec![RecoveryStrategy::Retry, RecoveryStrategy::ReduceContext],
        );
        error_handlers.insert(
            RecoverableError::InvalidResponse,
            vec![RecoveryStrategy::Retry, RecoveryStrategy::SkipTools],
        );
        error_handlers.insert(
            RecoverableError::PartialResponse,
            vec![RecoveryStrategy::Retry, RecoveryStrategy::CompressHistory],
        );
        error_handlers.insert(
            RecoverableError::ContextTooLarge,
            vec![RecoveryStrategy::CompressHistory, RecoveryStrategy::ReduceContext],
        );
        error_handlers.insert(
            RecoverableError::Unknown,
            vec![RecoveryStrategy::Retry, RecoveryStrategy::Abort],
        );
        
        Self {
            config,
            error_handlers,
            recovery_history: Vec::new(),
        }
    }
    
    /// 获取错误的恢复策略
    pub fn get_recovery_strategies(&self, error: &RecoverableError) -> Vec<RecoveryStrategy> {
        self.error_handlers
            .get(error)
            .cloned()
            .unwrap_or_else(|| vec![RecoveryStrategy::Abort])
    }
    
    /// 处理错误并返回恢复策略
    pub async fn handle_error(
        &self,
        error: RecoverableError,
    ) -> Result<RecoveryStrategy, String> {
        let strategies = self.get_recovery_strategies(&error);
        
        if strategies.is_empty() {
            return Err("No recovery strategy available".to_string());
        }
        
        // 返回第一个可用的策略
        Ok(strategies[0].clone())
    }
    
    /// 检查是否应该重试
    pub fn should_retry(&self, attempts: u32) -> bool {
        attempts < self.config.max_recovery_attempts
    }
    
    /// 获取重试延迟（毫秒）
    pub fn get_retry_delay(&self, attempt: u32) -> u64 {
        // 指数退避：delay * 2^attempt
        self.config.retry_delay_ms * (2_u64.pow(attempt))
    }
    
    /// 获取备选模型
    pub fn get_fallback_model(&self, current_model: &str) -> Option<String> {
        self.config
            .fallback_models
            .iter()
            .find(|m| m != &current_model)
            .cloned()
    }
    
    /// 计算上下文缩减量
    pub fn calculate_context_reduction(&self, current_size: usize) -> usize {
        (current_size as f32 * self.config.context_reduction_factor) as usize
    }
    
    /// 记录恢复结果
    pub fn record_recovery(&mut self, result: RecoveryResult) {
        self.recovery_history.push(result);
    }
    
    /// 获取恢复历史
    pub fn get_recovery_history(&self) -> &[RecoveryResult] {
        &self.recovery_history
    }
    
    /// 清空恢复历史
    pub fn clear_history(&mut self) {
        self.recovery_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_identification() {
        assert_eq!(
            RecoverableError::from_string("rate limit exceeded"),
            RecoverableError::RateLimitExceeded
        );
        assert_eq!(
            RecoverableError::from_string("token limit"),
            RecoverableError::TokenLimitExceeded
        );
        assert_eq!(
            RecoverableError::from_string("model not found"),
            RecoverableError::ModelNotAvailable
        );
    }

    #[test]
    fn test_recovery_strategies() {
        let recovery = ErrorRecovery::new(RecoveryConfig::default());
        
        let strategies = recovery.get_recovery_strategies(&RecoverableError::RateLimitExceeded);
        assert!(!strategies.is_empty());
        assert_eq!(strategies[0], RecoveryStrategy::Retry);
    }

    #[test]
    fn test_retry_delay() {
        let recovery = ErrorRecovery::new(RecoveryConfig::default());
        
        let delay_0 = recovery.get_retry_delay(0);
        let delay_1 = recovery.get_retry_delay(1);
        let delay_2 = recovery.get_retry_delay(2);
        
        assert!(delay_1 > delay_0);
        assert!(delay_2 > delay_1);
    }

    #[test]
    fn test_context_reduction() {
        let recovery = ErrorRecovery::new(RecoveryConfig::default());
        
        let original_size = 1000;
        let reduced_size = recovery.calculate_context_reduction(original_size);
        
        assert!(reduced_size < original_size);
        assert_eq!(reduced_size, 800); // 0.8 * 1000
    }

    #[test]
    fn test_fallback_model() {
        let recovery = ErrorRecovery::new(RecoveryConfig::default());
        
        let fallback = recovery.get_fallback_model("gemini-2.5-pro");
        assert!(fallback.is_some());
        assert_ne!(fallback.unwrap(), "gemini-2.5-pro");
    }

    #[tokio::test]
    async fn test_handle_error() {
        let recovery = ErrorRecovery::new(RecoveryConfig::default());
        
        let strategy = recovery
            .handle_error(RecoverableError::RateLimitExceeded)
            .await;
        assert!(strategy.is_ok());
    }

    #[test]
    fn test_recovery_history() {
        let mut recovery = ErrorRecovery::new(RecoveryConfig::default());
        
        let result = RecoveryResult {
            strategy_used: RecoveryStrategy::Retry,
            success: true,
            attempts: 1,
            fallback_model: None,
            context_reduced: false,
            history_compressed: false,
        };
        
        recovery.record_recovery(result);
        assert_eq!(recovery.get_recovery_history().len(), 1);
        
        recovery.clear_history();
        assert_eq!(recovery.get_recovery_history().len(), 0);
    }
}
