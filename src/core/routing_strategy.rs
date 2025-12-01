use std::sync::Arc;

use async_trait::async_trait;

use crate::core::conversation_engine::ConversationContext;
use crate::core::RetryConfig;

/// 模型路由决策结果
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub model: String,
    pub reason: String,
}

impl RoutingDecision {
    pub fn new(model: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            reason: reason.into(),
        }
    }
}

/// 路由策略接口，参考 Gemini CLI 的 `RoutingStrategy`
#[async_trait]
pub trait RoutingStrategy: Send + Sync {
    fn name(&self) -> &str;

    async fn route(
        &self,
        context: &ConversationContext,
        retry: &RetryConfig,
    ) -> Result<Option<RoutingDecision>, String>;
}

/// 组合路由器，顺序尝试多种策略
#[derive(Default)]
pub struct CompositeRouter {
    strategies: Vec<Arc<dyn RoutingStrategy>>,
    default_model: String,
}

impl CompositeRouter {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
            default_model: "gemini-2.5-pro".to_string(),
        }
    }

    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    pub fn register_strategy(&mut self, strategy: Arc<dyn RoutingStrategy>) {
        self.strategies.push(strategy);
    }

    pub async fn route(
        &self,
        context: &ConversationContext,
        retry: &RetryConfig,
    ) -> Result<RoutingDecision, String> {
        for strategy in &self.strategies {
            if let Some(decision) = strategy.route(context, retry).await? {
                return Ok(decision);
            }
        }

        Ok(RoutingDecision::new(
            &self.default_model,
            "default routing strategy",
        ))
    }
}

/// 默认策略：始终返回默认模型
pub struct DefaultRoutingStrategy;

#[async_trait]
impl RoutingStrategy for DefaultRoutingStrategy {
    fn name(&self) -> &str {
        "default"
    }

    async fn route(
        &self,
        _context: &ConversationContext,
        _retry: &RetryConfig,
    ) -> Result<Option<RoutingDecision>, String> {
        Ok(None)
    }
}
