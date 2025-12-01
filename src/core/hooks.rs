/// 钩子系统 - 对应 Gemini CLI 的 Hook Triggers
/// 
/// 支持在对话生命周期的关键点执行自定义逻辑
/// 类似于 fireBeforeModelHook, fireAfterModelHook 等

use std::sync::Arc;
use async_trait::async_trait;
use crate::core::conversation_engine::{ConversationContext, ProcessedResponse};

/// 钩子执行结果
pub type HookResult = Result<(), String>;

/// 前置钩子 - 在 LLM 调用前执行
#[async_trait]
pub trait BeforeModelHook: Send + Sync {
    async fn execute(&self, context: &ConversationContext) -> HookResult;
}

/// 后置钩子 - 在 LLM 调用后执行
#[async_trait]
pub trait AfterModelHook: Send + Sync {
    async fn execute(&self, response: &ProcessedResponse) -> HookResult;
}

/// 工具选择前钩子 - 在选择工具前执行
#[async_trait]
pub trait BeforeToolSelectionHook: Send + Sync {
    async fn execute(&self, response: &ProcessedResponse) -> HookResult;
}

/// 工具执行后钩子 - 在工具执行后执行
#[async_trait]
pub trait AfterToolExecutionHook: Send + Sync {
    async fn execute(&self, tool_name: &str, result: &str) -> HookResult;
}

/// 重试钩子 - 在重试前执行
#[async_trait]
pub trait OnRetryHook: Send + Sync {
    async fn execute(&self, attempt: u32, reason: &str) -> HookResult;
}

/// 钩子管理器 - 管理所有钩子
pub struct HookManager {
    before_model_hooks: Vec<Arc<dyn BeforeModelHook>>,
    after_model_hooks: Vec<Arc<dyn AfterModelHook>>,
    before_tool_selection_hooks: Vec<Arc<dyn BeforeToolSelectionHook>>,
    after_tool_execution_hooks: Vec<Arc<dyn AfterToolExecutionHook>>,
    on_retry_hooks: Vec<Arc<dyn OnRetryHook>>,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            before_model_hooks: Vec::new(),
            after_model_hooks: Vec::new(),
            before_tool_selection_hooks: Vec::new(),
            after_tool_execution_hooks: Vec::new(),
            on_retry_hooks: Vec::new(),
        }
    }

    /// 注册前置模型钩子
    pub fn register_before_model_hook(&mut self, hook: Arc<dyn BeforeModelHook>) {
        self.before_model_hooks.push(hook);
    }

    /// 注册后置模型钩子
    pub fn register_after_model_hook(&mut self, hook: Arc<dyn AfterModelHook>) {
        self.after_model_hooks.push(hook);
    }

    /// 注册工具选择前钩子
    pub fn register_before_tool_selection_hook(&mut self, hook: Arc<dyn BeforeToolSelectionHook>) {
        self.before_tool_selection_hooks.push(hook);
    }

    /// 注册工具执行后钩子
    pub fn register_after_tool_execution_hook(&mut self, hook: Arc<dyn AfterToolExecutionHook>) {
        self.after_tool_execution_hooks.push(hook);
    }

    /// 注册重试钩子
    pub fn register_on_retry_hook(&mut self, hook: Arc<dyn OnRetryHook>) {
        self.on_retry_hooks.push(hook);
    }

    /// 执行所有前置模型钩子
    pub async fn fire_before_model_hooks(&self, context: &ConversationContext) -> HookResult {
        for hook in &self.before_model_hooks {
            hook.execute(context).await?;
        }
        Ok(())
    }

    /// 执行所有后置模型钩子
    pub async fn fire_after_model_hooks(&self, response: &ProcessedResponse) -> HookResult {
        for hook in &self.after_model_hooks {
            hook.execute(response).await?;
        }
        Ok(())
    }

    /// 执行所有工具选择前钩子
    pub async fn fire_before_tool_selection_hooks(&self, response: &ProcessedResponse) -> HookResult {
        for hook in &self.before_tool_selection_hooks {
            hook.execute(response).await?;
        }
        Ok(())
    }

    /// 执行所有工具执行后钩子
    pub async fn fire_after_tool_execution_hooks(&self, tool_name: &str, result: &str) -> HookResult {
        for hook in &self.after_tool_execution_hooks {
            hook.execute(tool_name, result).await?;
        }
        Ok(())
    }

    /// 执行所有重试钩子
    pub async fn fire_on_retry_hooks(&self, attempt: u32, reason: &str) -> HookResult {
        for hook in &self.on_retry_hooks {
            hook.execute(attempt, reason).await?;
        }
        Ok(())
    }

    /// 前置钩子别名（用于 chat_orchestrator）
    pub async fn run_pre_hooks(&self, context: &ConversationContext) -> HookResult {
        self.fire_before_model_hooks(context).await
    }

    /// 后置钩子别名（用于 chat_orchestrator）
    pub async fn run_post_hooks(&self, response: &ProcessedResponse) -> HookResult {
        self.fire_after_model_hooks(response).await
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 日志钩子 - 记录对话过程
pub struct LoggingHook;

#[async_trait]
impl BeforeModelHook for LoggingHook {
    async fn execute(&self, context: &ConversationContext) -> HookResult {
        println!("[BEFORE_MODEL] Input: {}", context.user_input);
        Ok(())
    }
}

#[async_trait]
impl AfterModelHook for LoggingHook {
    async fn execute(&self, response: &ProcessedResponse) -> HookResult {
        println!("[AFTER_MODEL] Response length: {}", response.content.len());
        println!("[AFTER_MODEL] Modifications: {}", response.modifications.len());
        Ok(())
    }
}

#[async_trait]
impl OnRetryHook for LoggingHook {
    async fn execute(&self, attempt: u32, reason: &str) -> HookResult {
        println!("[RETRY] Attempt: {}, Reason: {}", attempt, reason);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hook_manager_creation() {
        let manager = HookManager::new();
        assert_eq!(manager.before_model_hooks.len(), 0);
        assert_eq!(manager.after_model_hooks.len(), 0);
    }

    #[tokio::test]
    async fn test_logging_hook() {
        let hook = LoggingHook;
        let context = ConversationContext::new(
            "test input".to_string(),
            crate::core::UserIntent::Chat {
                query: "test".to_string(),
                context_files: vec![],
            },
        );
        
        let result = <LoggingHook as BeforeModelHook>::execute(&hook, &context).await;
        assert!(result.is_ok());
    }
}
