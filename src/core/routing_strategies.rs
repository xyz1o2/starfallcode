/// 具体的路由策略实现 - 对应 Gemini CLI 的各种 RoutingStrategy
/// 
/// 支持多种模型选择策略：
/// - FallbackStrategy: 主模型失败时降级到备选模型
/// - ModelSelectionStrategy: 根据任务类型选择最合适的模型
/// - CostOptimizationStrategy: 根据成本选择模型
/// - PerformanceStrategy: 根据性能选择模型

use async_trait::async_trait;
use crate::core::{RoutingStrategy, RoutingDecision, RetryConfig};
use crate::core::conversation_engine::{ConversationContext, UserIntent};

/// 降级策略 - 主模型失败时尝试备选模型
pub struct FallbackStrategy {
    primary_model: String,
    fallback_models: Vec<String>,
}

impl FallbackStrategy {
    pub fn new(primary: impl Into<String>, fallbacks: Vec<String>) -> Self {
        Self {
            primary_model: primary.into(),
            fallback_models: fallbacks,
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            primary_model: "gemini-2.5-pro".to_string(),
            fallback_models: vec![
                "gemini-2.0-flash".to_string(),
                "gemini-1.5-pro".to_string(),
            ],
        }
    }
}

#[async_trait]
impl RoutingStrategy for FallbackStrategy {
    fn name(&self) -> &str {
        "fallback"
    }

    async fn route(
        &self,
        _context: &ConversationContext,
        _retry: &RetryConfig,
    ) -> Result<Option<RoutingDecision>, String> {
        Ok(Some(RoutingDecision::new(
            &self.primary_model,
            "primary model from fallback strategy",
        )))
    }
}

/// 模型选择策略 - 根据任务类型选择最合适的模型
pub struct ModelSelectionStrategy {
    code_model: String,
    chat_model: String,
    analysis_model: String,
}

impl ModelSelectionStrategy {
    pub fn new(code: impl Into<String>, chat: impl Into<String>, analysis: impl Into<String>) -> Self {
        Self {
            code_model: code.into(),
            chat_model: chat.into(),
            analysis_model: analysis.into(),
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            code_model: "gemini-2.5-pro".to_string(),
            chat_model: "gemini-2.0-flash".to_string(),
            analysis_model: "gemini-1.5-pro".to_string(),
        }
    }

    fn select_model_for_intent(intent: &UserIntent) -> &'static str {
        match intent {
            UserIntent::CodeReview { .. } => "code",
            UserIntent::CodeGeneration { .. } => "code",
            UserIntent::Debug { .. } => "code",
            UserIntent::Chat { .. } => "chat",
            UserIntent::FileMention { .. } => "analysis",
            UserIntent::Command { .. } => "chat",
        }
    }
}

#[async_trait]
impl RoutingStrategy for ModelSelectionStrategy {
    fn name(&self) -> &str {
        "model_selection"
    }

    async fn route(
        &self,
        context: &ConversationContext,
        _retry: &RetryConfig,
    ) -> Result<Option<RoutingDecision>, String> {
        let intent_type = Self::select_model_for_intent(&context.intent);
        
        let (model, reason) = match intent_type {
            "code" => (&self.code_model, "code-related task"),
            "chat" => (&self.chat_model, "general chat"),
            "analysis" => (&self.analysis_model, "analysis task"),
            _ => (&self.chat_model, "default"),
        };

        Ok(Some(RoutingDecision::new(model, reason)))
    }
}

/// 成本优化策略 - 根据成本选择模型
pub struct CostOptimizationStrategy {
    cheap_model: String,
    expensive_model: String,
    cost_threshold: f32,
}

impl CostOptimizationStrategy {
    pub fn new(cheap: impl Into<String>, expensive: impl Into<String>, threshold: f32) -> Self {
        Self {
            cheap_model: cheap.into(),
            expensive_model: expensive.into(),
            cost_threshold: threshold,
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            cheap_model: "gemini-2.0-flash".to_string(),
            expensive_model: "gemini-2.5-pro".to_string(),
            cost_threshold: 0.5,
        }
    }
}

#[async_trait]
impl RoutingStrategy for CostOptimizationStrategy {
    fn name(&self) -> &str {
        "cost_optimization"
    }

    async fn route(
        &self,
        context: &ConversationContext,
        _retry: &RetryConfig,
    ) -> Result<Option<RoutingDecision>, String> {
        // 简单的启发式：长输入使用便宜模型
        let input_length = context.user_input.len() as f32;
        let threshold = self.cost_threshold * 1000.0;

        let (model, reason) = if input_length > threshold {
            (&self.cheap_model, "long input, using cost-optimized model")
        } else {
            (&self.expensive_model, "short input, using high-quality model")
        };

        Ok(Some(RoutingDecision::new(model, reason)))
    }
}

/// 性能策略 - 根据性能选择模型
pub struct PerformanceStrategy {
    fast_model: String,
    quality_model: String,
}

impl PerformanceStrategy {
    pub fn new(fast: impl Into<String>, quality: impl Into<String>) -> Self {
        Self {
            fast_model: fast.into(),
            quality_model: quality.into(),
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            fast_model: "gemini-2.0-flash".to_string(),
            quality_model: "gemini-2.5-pro".to_string(),
        }
    }
}

#[async_trait]
impl RoutingStrategy for PerformanceStrategy {
    fn name(&self) -> &str {
        "performance"
    }

    async fn route(
        &self,
        context: &ConversationContext,
        _retry: &RetryConfig,
    ) -> Result<Option<RoutingDecision>, String> {
        // 简单的启发式：命令使用快速模型，其他使用质量模型
        let (model, reason) = match &context.intent {
            UserIntent::Command { .. } => (&self.fast_model, "command execution, using fast model"),
            _ => (&self.quality_model, "general task, using quality model"),
        };

        Ok(Some(RoutingDecision::new(model, reason)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fallback_strategy() {
        let strategy = FallbackStrategy::with_defaults();
        let context = ConversationContext::new(
            "test".to_string(),
            UserIntent::Chat {
                query: "test".to_string(),
                context_files: vec![],
            },
        );
        let retry = RetryConfig::default();

        let result = strategy.route(&context, &retry).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_model_selection_strategy() {
        let strategy = ModelSelectionStrategy::with_defaults();
        let context = ConversationContext::new(
            "review this code".to_string(),
            UserIntent::CodeReview {
                files: vec![],
                focus: "performance".to_string(),
            },
        );
        let retry = RetryConfig::default();

        let result = strategy.route(&context, &retry).await;
        assert!(result.is_ok());
        let decision = result.unwrap().unwrap();
        assert_eq!(decision.model, "gemini-2.5-pro");
    }

    #[tokio::test]
    async fn test_cost_optimization_strategy() {
        let strategy = CostOptimizationStrategy::with_defaults();
        let context = ConversationContext::new(
            "short".to_string(),
            UserIntent::Chat {
                query: "short".to_string(),
                context_files: vec![],
            },
        );
        let retry = RetryConfig::default();

        let result = strategy.route(&context, &retry).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_performance_strategy() {
        let strategy = PerformanceStrategy::with_defaults();
        let context = ConversationContext::new(
            "/help".to_string(),
            UserIntent::Command {
                name: "help".to_string(),
                args: vec![],
            },
        );
        let retry = RetryConfig::default();

        let result = strategy.route(&context, &retry).await;
        assert!(result.is_ok());
        let decision = result.unwrap().unwrap();
        assert_eq!(decision.model, "gemini-2.0-flash");
    }
}
