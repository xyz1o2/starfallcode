use crate::core::message::{Message, Role};
use serde::{Deserialize, Serialize};

/// 上下文优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// 最大上下文令牌数
    pub max_tokens: usize,
    /// 保留的输出令牌数
    pub reserve_output_tokens: usize,
    /// 最少保留的消息数
    pub min_messages_to_keep: usize,
    /// 是否启用摘要
    pub enable_summarization: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4000,
            reserve_output_tokens: 1000,
            min_messages_to_keep: 5,
            enable_summarization: true,
        }
    }
}

/// 优化后的上下文
#[derive(Debug, Clone)]
pub struct OptimizedContext {
    pub messages: Vec<Message>,
    pub token_usage: TokenUsage,
    pub was_truncated: bool,
}

/// 令牌使用统计
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub system_tokens: usize,
    pub messages_tokens: usize,
    pub total_tokens: usize,
}

/// 上下文窗口优化器
pub struct ContextWindowOptimizer {
    config: ContextConfig,
}

impl ContextWindowOptimizer {
    pub fn new(config: ContextConfig) -> Self {
        Self { config }
    }

    /// 优化消息上下文以适应令牌限制
    pub fn optimize_context(&self, messages: Vec<Message>) -> OptimizedContext {
        let available_tokens = self.config.max_tokens - self.config.reserve_output_tokens;
        let mut optimized_messages = Vec::new();
        let mut token_count = 0;
        let mut was_truncated = false;

        // 1. 保留系统消息
        let system_messages: Vec<_> = messages
            .iter()
            .filter(|m| m.role == Role::System)
            .cloned()
            .collect();

        for msg in &system_messages {
            let tokens = self.estimate_tokens(&msg.content);
            token_count += tokens;
            optimized_messages.push(msg.clone());
        }

        // 2. 反向遍历消息，优先保留最近的
        let non_system: Vec<_> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .collect();

        let mut recent_messages = Vec::new();
        for msg in non_system.iter().rev() {
            let msg_tokens = self.estimate_tokens(&msg.content);

            if token_count + msg_tokens > available_tokens {
                was_truncated = true;
                break;
            }

            recent_messages.insert(0, (*msg).clone());
            token_count += msg_tokens;
        }

        // 3. 如果被截断且启用摘要，添加摘要消息
        if was_truncated && self.config.enable_summarization {
            let truncated_count = non_system.len() - recent_messages.len();
            if truncated_count > 0 {
                let summary = self.create_summary_message(truncated_count, &non_system);
                optimized_messages.push(summary);
            }
        }

        optimized_messages.extend(recent_messages);

        OptimizedContext {
            messages: optimized_messages,
            token_usage: TokenUsage {
                system_tokens: system_messages.iter().map(|m| self.estimate_tokens(&m.content)).sum(),
                messages_tokens: token_count,
                total_tokens: token_count,
            },
            was_truncated,
        }
    }

    /// 估算文本的令牌数（简单实现）
    /// 实际应用中应使用 tiktoken 或类似库
    fn estimate_tokens(&self, text: &str) -> usize {
        // 粗略估计：平均每个单词 1.3 个令牌
        let word_count = text.split_whitespace().count();
        (word_count as f64 * 1.3).ceil() as usize
    }

    /// 创建摘要消息
    fn create_summary_message(&self, truncated_count: usize, _messages: &[&Message]) -> Message {
        let summary = format!(
            "[Previous {} messages summarized: Contains {} user and assistant exchanges covering various topics]",
            truncated_count,
            truncated_count / 2
        );

        Message {
            role: Role::System,
            content: summary,
        }
    }

    /// 获取消息统计信息
    pub fn get_stats(&self, messages: &[Message]) -> TokenUsage {
        let system_tokens: usize = messages
            .iter()
            .filter(|m| m.role == Role::System)
            .map(|m| self.estimate_tokens(&m.content))
            .sum();

        let messages_tokens: usize = messages
            .iter()
            .map(|m| self.estimate_tokens(&m.content))
            .sum();

        TokenUsage {
            system_tokens,
            messages_tokens,
            total_tokens: messages_tokens,
        }
    }

    /// 检查是否需要优化
    pub fn needs_optimization(&self, messages: &[Message]) -> bool {
        let total_tokens: usize = messages
            .iter()
            .map(|m| self.estimate_tokens(&m.content))
            .sum();

        total_tokens > self.config.max_tokens - self.config.reserve_output_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_optimization() {
        let config = ContextConfig {
            max_tokens: 1000,
            reserve_output_tokens: 200,
            min_messages_to_keep: 2,
            enable_summarization: true,
        };

        let optimizer = ContextWindowOptimizer::new(config);

        let messages = vec![
            Message {
                role: Role::User,
                content: "Hello".to_string(),
            },
            Message {
                role: Role::Assistant,
                content: "Hi there".to_string(),
            },
        ];

        let optimized = optimizer.optimize_context(messages);
        assert!(!optimized.was_truncated);
        assert_eq!(optimized.messages.len(), 2);
    }
}
