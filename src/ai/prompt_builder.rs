/// Prompt Builder - 分离系统提示词和规则提示词
/// 
/// 这个模块解决了一个关键问题：
/// - the-augment.xml 规则提示词不能放在 system message 中
/// - 否则会干扰 LLM 的 function calling 功能
/// 
/// 解决方案：
/// - System Message：简洁的角色定义和工具说明
/// - User Message：包含规则和用户请求

use std::fs;
use std::path::Path;

/// 消息结构
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// 提示词构建器
pub struct PromptBuilder {
    system_prompt: String,
    augment_rules: Option<String>,
    include_rules_in_user_message: bool,
}

impl PromptBuilder {
    /// 创建新的提示词构建器
    pub fn new() -> Self {
        Self {
            system_prompt: Self::default_system_prompt(),
            augment_rules: None,
            include_rules_in_user_message: true,
        }
    }

    /// 加载 the-augment.xml 规则
    pub fn load_augment_rules(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let path = "src/prompts/the-augment.xml";
        if Path::new(path).exists() {
            let content = fs::read_to_string(path)?;
            self.augment_rules = Some(content);
        }
        Ok(self)
    }

    /// 从字符串加载规则
    pub fn with_rules(mut self, rules: String) -> Self {
        self.augment_rules = Some(rules);
        self
    }

    /// 设置是否在用户消息中包含规则
    pub fn include_rules_in_user_message(mut self, include: bool) -> Self {
        self.include_rules_in_user_message = include;
        self
    }

    /// 设置自定义系统提示词
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// 简洁的默认系统提示词
    /// 
    /// 注意：这里故意简洁，不包含复杂的规则
    /// 规则会通过用户消息注入
    fn default_system_prompt() -> String {
        r#"You are The Augster, an elite AI programming partner.

Your role:
- Provide expert coding assistance
- Use tools proactively to solve problems
- Follow the augment rules provided in the conversation

You have access to tools for:
- Code analysis and modification
- File operations
- Terminal commands
- Project structure analysis

When the user provides augment rules, follow them throughout the conversation.
Always prioritize tool usage when appropriate."#
            .to_string()
    }

    /// 构建消息列表（不包含规则确认）
    pub fn build_messages(&self, user_request: &str) -> Vec<Message> {
        let mut messages = vec![Message::system(self.system_prompt.clone())];

        // 构建用户消息
        let user_content = if self.include_rules_in_user_message {
            if let Some(rules) = &self.augment_rules {
                format!(
                    r#"<augment_rules>
{}
</augment_rules>

User Request:
{}"#,
                    rules, user_request
                )
            } else {
                user_request.to_string()
            }
        } else {
            user_request.to_string()
        };

        messages.push(Message::user(user_content));
        messages
    }

    /// 构建消息列表（包含规则确认）
    /// 
    /// 这种方式适合长对话：
    /// 1. 第一条消息：加载规则
    /// 2. LLM 确认规则
    /// 3. 后续消息：只包含用户请求
    pub fn build_messages_with_confirmation(&self, user_request: &str) -> Vec<Message> {
        let mut messages = vec![Message::system(self.system_prompt.clone())];

        // 如果有规则，先让 LLM 确认
        if let Some(rules) = &self.augment_rules {
            messages.push(Message::user(format!(
                r#"Please acknowledge that you understand these augment rules and will follow them:

<augment_rules>
{}
</augment_rules>

Respond with: "I understand and will follow the augment rules.""#,
                rules
            )));

            // 模拟 LLM 的确认响应
            messages.push(Message::assistant(
                "I understand and will follow the augment rules.".to_string(),
            ));
        }

        // 用户的实际请求
        messages.push(Message::user(user_request.to_string()));

        messages
    }

    /// 获取规则内容
    pub fn get_rules(&self) -> Option<&str> {
        self.augment_rules.as_deref()
    }

    /// 获取系统提示词
    pub fn get_system_prompt(&self) -> &str {
        &self.system_prompt
    }

    /// 获取规则的统计信息
    pub fn get_rules_stats(&self) -> Option<RulesStats> {
        self.augment_rules.as_ref().map(|rules| RulesStats {
            total_chars: rules.len(),
            total_lines: rules.lines().count(),
            estimated_tokens: (rules.len() / 4) as u32, // 粗略估计
        })
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 规则统计信息
#[derive(Debug, Clone)]
pub struct RulesStats {
    pub total_chars: usize,
    pub total_lines: usize,
    pub estimated_tokens: u32,
}

/// 规则压缩器 - 减少 token 消耗
pub struct RulesCompressor;

impl RulesCompressor {
    /// 移除注释和空行
    pub fn compress(rules: &str) -> String {
        rules
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with("<!--")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 提取核心规则（最小化版本）
    pub fn extract_core_rules(rules: &str) -> String {
        // 提取 CorePrinciples 和 CoreIdentity
        let mut result = String::new();

        for line in rules.lines() {
            if line.contains("CoreIdentity") 
                || line.contains("CorePrinciples")
                || line.contains("Trait")
                || line.contains("PrimaryFunction")
                || line.contains("CoreMandate") {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }

    /// 获取压缩率
    pub fn compression_ratio(original: &str, compressed: &str) -> f32 {
        if original.is_empty() {
            return 0.0;
        }
        (compressed.len() as f32 / original.len() as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::system("test");
        assert_eq!(msg.role, "system");
        assert_eq!(msg.content, "test");
    }

    #[test]
    fn test_prompt_builder_default() {
        let builder = PromptBuilder::new();
        let messages = builder.build_messages("Hello");

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[1].role, "user");
        assert!(messages[1].content.contains("Hello"));
    }

    #[test]
    fn test_prompt_builder_with_rules() {
        let rules = "<rules>Test Rule</rules>";
        let builder = PromptBuilder::new().with_rules(rules.to_string());
        let messages = builder.build_messages("Hello");

        assert_eq!(messages.len(), 2);
        assert!(messages[1].content.contains("<augment_rules>"));
        assert!(messages[1].content.contains("Test Rule"));
    }

    #[test]
    fn test_prompt_builder_with_confirmation() {
        let rules = "<rules>Test Rule</rules>";
        let builder = PromptBuilder::new().with_rules(rules.to_string());
        let messages = builder.build_messages_with_confirmation("Hello");

        // 应该有：系统消息 + 规则确认 + LLM确认 + 用户请求
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[2].role, "assistant");
        assert_eq!(messages[3].role, "user");
    }

    #[test]
    fn test_rules_compressor() {
        let original = r#"
<!-- Comment -->
<rule>
    <item>Test</item>
</rule>

<empty>

</empty>
"#;
        let compressed = RulesCompressor::compress(original);
        
        // 应该移除注释和空行
        assert!(!compressed.contains("<!--"));
        assert!(compressed.contains("<rule>"));
    }

    #[test]
    fn test_rules_stats() {
        let rules = "Test rules content";
        let builder = PromptBuilder::new().with_rules(rules.to_string());
        let stats = builder.get_rules_stats();

        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.total_chars, 18);
        assert_eq!(stats.total_lines, 1);
    }
}
