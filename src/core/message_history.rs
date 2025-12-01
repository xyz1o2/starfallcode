/// 消息历史管理模块 - 对应 Gemini CLI 的 Turn 和 ConversationHistory
/// 
/// 支持完整的对话历史管理：
/// - 消息存储和检索
/// - 上下文窗口管理
/// - 令牌计数
/// - 历史压缩

use std::collections::VecDeque;
use chrono::{DateTime, Local};
use crate::core::conversation_engine::ConversationContext;

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
        }
    }
}

/// 单条消息
#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub token_count: usize,
}

impl Message {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            timestamp: Local::now(),
            token_count: 0,
        }
    }

    pub fn with_token_count(mut self, count: usize) -> Self {
        self.token_count = count;
        self
    }

    /// 估算 token 数（简单启发式）
    pub fn estimate_tokens(&mut self) {
        // 简单估算：平均每 4 个字符 = 1 token
        self.token_count = (self.content.len() + 3) / 4;
    }
}

/// 对话轮次 - 一个用户消息和对应的助手响应
#[derive(Debug, Clone)]
pub struct Turn {
    pub user_message: Message,
    pub assistant_message: Option<Message>,
    pub context: Option<ConversationContext>,
}

impl Turn {
    pub fn new(user_message: Message) -> Self {
        Self {
            user_message,
            assistant_message: None,
            context: None,
        }
    }

    pub fn with_assistant_response(mut self, response: Message) -> Self {
        self.assistant_message = Some(response);
        self
    }

    pub fn with_context(mut self, context: ConversationContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn total_tokens(&self) -> usize {
        let mut total = self.user_message.token_count;
        if let Some(ref msg) = self.assistant_message {
            total += msg.token_count;
        }
        total
    }
}

/// 消息历史管理器
pub struct MessageHistory {
    messages: VecDeque<Message>,
    turns: VecDeque<Turn>,
    max_messages: usize,
    max_tokens: usize,
    current_tokens: usize,
}

impl MessageHistory {
    pub fn new(max_messages: usize, max_tokens: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            turns: VecDeque::new(),
            max_messages,
            max_tokens,
            current_tokens: 0,
        }
    }

    /// 添加消息
    pub fn add_message(&mut self, mut message: Message) -> Result<(), String> {
        message.estimate_tokens();

        // 检查是否超过 token 限制
        if self.current_tokens + message.token_count > self.max_tokens {
            self.trim_oldest_messages()?;
        }

        self.current_tokens += message.token_count;
        self.messages.push_back(message);

        // 检查是否超过消息数限制
        while self.messages.len() > self.max_messages {
            if let Some(msg) = self.messages.pop_front() {
                self.current_tokens = self.current_tokens.saturating_sub(msg.token_count);
            }
        }

        Ok(())
    }

    /// 添加对话轮次
    pub fn add_turn(&mut self, mut turn: Turn) -> Result<(), String> {
        turn.user_message.estimate_tokens();
        if let Some(ref mut msg) = turn.assistant_message {
            msg.estimate_tokens();
        }

        self.add_message(turn.user_message.clone())?;
        if let Some(msg) = turn.assistant_message.clone() {
            self.add_message(msg)?;
        }

        self.turns.push_back(turn);

        // 保持轮次数量与消息数量同步
        while self.turns.len() > self.max_messages / 2 {
            self.turns.pop_front();
        }

        Ok(())
    }

    /// 添加助手消息（便捷方法）
    pub fn add_assistant_message(&mut self, content: impl Into<String>) -> Result<(), String> {
        let msg = Message::new(MessageRole::Assistant, content);
        self.add_message(msg)
    }

    /// 获取所有消息
    pub fn get_messages(&self) -> Vec<&Message> {
        self.messages.iter().collect()
    }

    /// 获取所有轮次
    pub fn get_turns(&self) -> Vec<&Turn> {
        self.turns.iter().collect()
    }

    /// 获取最后 N 条消息
    pub fn get_last_n_messages(&self, n: usize) -> Vec<&Message> {
        self.messages
            .iter()
            .rev()
            .take(n)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// 获取最后一条消息
    pub fn get_last_message(&self) -> Option<&Message> {
        self.messages.back()
    }

    /// 清空历史
    pub fn clear(&mut self) {
        self.messages.clear();
        self.turns.clear();
        self.current_tokens = 0;
    }

    /// 获取当前 token 数
    pub fn get_current_tokens(&self) -> usize {
        self.current_tokens
    }

    /// 获取剩余 token 数
    pub fn get_remaining_tokens(&self) -> usize {
        self.max_tokens.saturating_sub(self.current_tokens)
    }

    /// 获取消息数
    pub fn get_message_count(&self) -> usize {
        self.messages.len()
    }

    /// 获取轮次数
    pub fn get_turn_count(&self) -> usize {
        self.turns.len()
    }

    /// 修剪最旧的消息
    fn trim_oldest_messages(&mut self) -> Result<(), String> {
        if let Some(msg) = self.messages.pop_front() {
            self.current_tokens = self.current_tokens.saturating_sub(msg.token_count);
            Ok(())
        } else {
            Err("No messages to trim".to_string())
        }
    }

    /// 压缩历史 - 移除旧消息直到满足 token 限制
    pub fn compress(&mut self, target_tokens: usize) -> Result<(), String> {
        while self.current_tokens > target_tokens && !self.messages.is_empty() {
            self.trim_oldest_messages()?;
        }
        Ok(())
    }

    /// 导出为字符串（用于调试）
    pub fn to_string_debug(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("=== Message History ({}  messages, {} tokens) ===\n", 
            self.messages.len(), self.current_tokens));

        for (i, msg) in self.messages.iter().enumerate() {
            result.push_str(&format!(
                "[{}] {}: {} ({} tokens)\n",
                i, msg.role, msg.content, msg.token_count
            ));
        }

        result
    }
}

impl Default for MessageHistory {
    fn default() -> Self {
        Self::new(100, 10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(MessageRole::User, "Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_token_estimation() {
        let mut msg = Message::new(MessageRole::User, "Hello world");
        msg.estimate_tokens();
        assert!(msg.token_count > 0);
    }

    #[test]
    fn test_turn_creation() {
        let user_msg = Message::new(MessageRole::User, "What is Rust?");
        let assistant_msg = Message::new(MessageRole::Assistant, "Rust is a systems programming language");
        
        let turn = Turn::new(user_msg)
            .with_assistant_response(assistant_msg);
        
        assert!(turn.assistant_message.is_some());
    }

    #[test]
    fn test_message_history() {
        let mut history = MessageHistory::new(10, 1000);
        
        let msg1 = Message::new(MessageRole::User, "Hello");
        history.add_message(msg1).unwrap();
        
        assert_eq!(history.get_message_count(), 1);
        assert!(history.get_last_message().is_some());
    }

    #[test]
    fn test_message_history_token_limit() {
        let mut history = MessageHistory::new(10, 50);
        
        let msg1 = Message::new(MessageRole::User, "This is a very long message that will consume many tokens");
        history.add_message(msg1).unwrap();
        
        let msg2 = Message::new(MessageRole::User, "Another message");
        history.add_message(msg2).unwrap();
        
        assert!(history.get_current_tokens() <= 50);
    }

    #[test]
    fn test_message_history_compression() {
        let mut history = MessageHistory::new(100, 1000);
        
        for i in 0..20 {
            let msg = Message::new(MessageRole::User, format!("Message {}", i));
            history.add_message(msg).unwrap();
        }
        
        history.compress(500).unwrap();
        assert!(history.get_current_tokens() <= 500);
    }

    #[test]
    fn test_get_last_n_messages() {
        let mut history = MessageHistory::new(10, 1000);
        
        for i in 0..5 {
            let msg = Message::new(MessageRole::User, format!("Message {}", i));
            history.add_message(msg).unwrap();
        }
        
        let last_3 = history.get_last_n_messages(3);
        assert_eq!(last_3.len(), 3);
    }
}
