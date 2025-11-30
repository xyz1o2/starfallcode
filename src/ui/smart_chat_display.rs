/// 智能聊天显示系统 - 基于 Gemini CLI 最佳实践
/// 
/// 特性：
/// - 思考过程展示（可折叠）
/// - 建议提示系统
/// - 流式响应优化
/// - 上下文感知的消息类型
/// - 对话历史管理

use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Question,
    Code,
    Explanation,
    Error,
    Suggestion,
    Thinking,
    Default,
}

/// 代码块
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: String,
    pub content: String,
    pub line_count: usize,
}

/// 问题/建议
#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// 消息元数据
#[derive(Debug, Clone, Default)]
pub struct MessageMetadata {
    pub has_code: bool,
    pub code_blocks: Vec<CodeBlock>,
    pub has_issues: bool,
    pub issues: Vec<Issue>,
    pub suggested_actions: Vec<String>,
}

/// 智能消息
#[derive(Debug, Clone)]
pub struct SmartMessage {
    pub role: MessageRole,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Local>,
    pub metadata: MessageMetadata,
}

impl SmartMessage {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            message_type: MessageType::Default,
            timestamp: Local::now(),
            metadata: MessageMetadata::default(),
        }
    }

    pub fn with_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }

    pub fn with_metadata(mut self, metadata: MessageMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// 思考过程显示
#[derive(Debug, Clone)]
pub struct ThinkingDisplay {
    pub thinking_content: String,
    pub is_visible: bool,
    pub collapsed: bool,
    pub created_at: Instant,
}

impl ThinkingDisplay {
    pub fn new(thinking: String) -> Self {
        Self {
            thinking_content: thinking,
            is_visible: true,
            collapsed: false,
            created_at: Instant::now(),
        }
    }

    pub fn toggle(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn get_display_text(&self) -> String {
        if self.collapsed {
            format!("💭 思考中... ({:.1}s)", self.created_at.elapsed().as_secs_f32())
        } else {
            format!("💭 思考过程:\n{}", self.thinking_content)
        }
    }
}

/// 建议提示栏
#[derive(Debug, Clone)]
pub struct SuggestionBar {
    pub suggestions: Vec<String>,
    pub selected_index: usize,
    pub visible: bool,
}

impl SuggestionBar {
    pub fn new(suggestions: Vec<String>) -> Self {
        Self {
            suggestions,
            selected_index: 0,
            visible: true,
        }
    }

    pub fn select_next(&mut self) {
        if self.selected_index < self.suggestions.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn get_selected(&self) -> Option<&str> {
        self.suggestions.get(self.selected_index).map(|s| s.as_str())
    }

    pub fn get_display_text(&self) -> String {
        let suggestions = self.suggestions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if i == self.selected_index {
                    format!("[{}] ✓ {}", i + 1, s)
                } else {
                    format!("[{}] {}", i + 1, s)
                }
            })
            .collect::<Vec<_>>()
            .join(" | ");

        format!("💡 建议: {}", suggestions)
    }
}

/// 流式响应显示
#[derive(Debug, Clone)]
pub struct StreamingDisplay {
    pub buffer: String,
    pub chunk_count: usize,
    pub last_update: Instant,
    pub is_complete: bool,
}

impl StreamingDisplay {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            chunk_count: 0,
            last_update: Instant::now(),
            is_complete: false,
        }
    }

    pub fn add_chunk(&mut self, chunk: &str) {
        self.buffer.push_str(chunk);
        self.chunk_count += 1;
        self.last_update = Instant::now();
    }

    pub fn should_update_ui(&self) -> bool {
        // 每 50ms 更新一次，避免过度渲染
        self.last_update.elapsed() > Duration::from_millis(50)
    }

    pub fn finalize(&mut self) {
        self.is_complete = true;
    }

    pub fn get_progress_text(&self) -> String {
        if self.is_complete {
            format!("✓ 完成 ({} 块)", self.chunk_count)
        } else {
            format!("⏳ 生成中... ({} 块)", self.chunk_count)
        }
    }
}

impl Default for StreamingDisplay {
    fn default() -> Self {
        Self::new()
    }
}

/// 智能聊天显示系统
pub struct SmartChatDisplay {
    pub messages: Vec<SmartMessage>,
    pub thinking_display: Option<ThinkingDisplay>,
    pub suggestion_bar: Option<SuggestionBar>,
    pub streaming_display: Option<StreamingDisplay>,
    pub scroll_offset: usize,
    pub message_cache: HashMap<usize, String>,
    pub dirty_flags: Vec<bool>,
}

impl SmartChatDisplay {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            thinking_display: None,
            suggestion_bar: None,
            streaming_display: None,
            scroll_offset: 0,
            message_cache: HashMap::new(),
            dirty_flags: Vec::new(),
        }
    }

    /// 添加消息
    pub fn add_message(&mut self, message: SmartMessage) {
        self.messages.push(message);
        self.dirty_flags.push(true);
        self.message_cache.remove(&(self.messages.len() - 1));
    }

    /// 显示思考过程
    pub fn show_thinking(&mut self, thinking: String) {
        self.thinking_display = Some(ThinkingDisplay::new(thinking));
    }

    /// 切换思考过程显示
    pub fn toggle_thinking(&mut self) {
        if let Some(ref mut thinking) = self.thinking_display {
            thinking.toggle();
        }
    }

    /// 隐藏思考过程
    pub fn hide_thinking(&mut self) {
        self.thinking_display = None;
    }

    /// 生成建议
    pub fn generate_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestion_bar = Some(SuggestionBar::new(suggestions));
    }

    /// 隐藏建议
    pub fn hide_suggestions(&mut self) {
        self.suggestion_bar = None;
    }

    /// 添加流式响应块
    pub fn add_streaming_chunk(&mut self, chunk: &str) {
        if self.streaming_display.is_none() {
            self.streaming_display = Some(StreamingDisplay::new());
        }

        if let Some(ref mut display) = self.streaming_display {
            display.add_chunk(chunk);
        }
    }

    /// 完成流式响应
    pub fn finalize_streaming(&mut self) -> Option<String> {
        if let Some(mut display) = self.streaming_display.take() {
            display.finalize();
            Some(display.buffer)
        } else {
            None
        }
    }

    /// 获取最后一条消息
    pub fn get_last_message(&self) -> Option<&SmartMessage> {
        self.messages.last()
    }

    /// 获取消息数量
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// 清空所有消息
    pub fn clear(&mut self) {
        self.messages.clear();
        self.dirty_flags.clear();
        self.message_cache.clear();
        self.thinking_display = None;
        self.suggestion_bar = None;
        self.streaming_display = None;
    }

    /// 滚动到底部
    pub fn scroll_to_bottom(&mut self) {
        if self.messages.len() > 0 {
            self.scroll_offset = self.messages.len() - 1;
        }
    }

    /// 向上滚动
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// 向下滚动
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
    }

    /// 获取显示的消息范围
    pub fn get_visible_messages(&self, max_lines: usize) -> Vec<&SmartMessage> {
        let start = self.scroll_offset;
        let end = (start + max_lines).min(self.messages.len());
        self.messages[start..end].iter().collect()
    }

    /// 标记消息为脏（需要重新渲染）
    pub fn mark_dirty(&mut self, index: usize) {
        if index < self.dirty_flags.len() {
            self.dirty_flags[index] = true;
            self.message_cache.remove(&index);
        }
    }

    /// 标记所有消息为脏
    pub fn mark_all_dirty(&mut self) {
        for flag in &mut self.dirty_flags {
            *flag = true;
        }
        self.message_cache.clear();
    }

    /// 检查消息是否需要重新渲染
    pub fn is_dirty(&self, index: usize) -> bool {
        self.dirty_flags.get(index).copied().unwrap_or(false)
    }

    /// 获取缓存的渲染内容
    pub fn get_cached_render(&self, index: usize) -> Option<&str> {
        self.message_cache.get(&index).map(|s| s.as_str())
    }

    /// 缓存渲染内容
    pub fn cache_render(&mut self, index: usize, rendered: String) {
        self.message_cache.insert(index, rendered);
        if index < self.dirty_flags.len() {
            self.dirty_flags[index] = false;
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> ChatStats {
        let total_messages = self.messages.len();
        let user_messages = self.messages.iter().filter(|m| m.role == MessageRole::User).count();
        let assistant_messages = self.messages.iter().filter(|m| m.role == MessageRole::Assistant).count();
        let total_chars: usize = self.messages.iter().map(|m| m.content.len()).sum();

        ChatStats {
            total_messages,
            user_messages,
            assistant_messages,
            total_chars,
        }
    }
}

impl Default for SmartChatDisplay {
    fn default() -> Self {
        Self::new()
    }
}

/// 聊天统计
#[derive(Debug, Clone)]
pub struct ChatStats {
    pub total_messages: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub total_chars: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_message_creation() {
        let msg = SmartMessage::new(MessageRole::User, "Hello".to_string());
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.message_type, MessageType::Default);
    }

    #[test]
    fn test_thinking_display() {
        let mut thinking = ThinkingDisplay::new("分析问题...".to_string());
        assert!(!thinking.collapsed);
        
        thinking.toggle();
        assert!(thinking.collapsed);
        
        let text = thinking.get_display_text();
        assert!(text.contains("💭"));
    }

    #[test]
    fn test_suggestion_bar() {
        let suggestions = vec![
            "解释".to_string(),
            "示例".to_string(),
            "对比".to_string(),
        ];
        let mut bar = SuggestionBar::new(suggestions);
        
        assert_eq!(bar.selected_index, 0);
        bar.select_next();
        assert_eq!(bar.selected_index, 1);
        bar.select_previous();
        assert_eq!(bar.selected_index, 0);
    }

    #[test]
    fn test_streaming_display() {
        let mut streaming = StreamingDisplay::new();
        assert_eq!(streaming.chunk_count, 0);
        
        streaming.add_chunk("Hello ");
        streaming.add_chunk("World");
        
        assert_eq!(streaming.chunk_count, 2);
        assert_eq!(streaming.buffer, "Hello World");
    }

    #[test]
    fn test_smart_chat_display() {
        let mut display = SmartChatDisplay::new();
        
        let msg = SmartMessage::new(MessageRole::User, "Hi".to_string());
        display.add_message(msg);
        
        assert_eq!(display.message_count(), 1);
        assert!(display.get_last_message().is_some());
    }

    #[test]
    fn test_chat_stats() {
        let mut display = SmartChatDisplay::new();
        
        display.add_message(SmartMessage::new(MessageRole::User, "Q".to_string()));
        display.add_message(SmartMessage::new(MessageRole::Assistant, "A".to_string()));
        
        let stats = display.get_stats();
        assert_eq!(stats.total_messages, 2);
        assert_eq!(stats.user_messages, 1);
        assert_eq!(stats.assistant_messages, 1);
    }
}
