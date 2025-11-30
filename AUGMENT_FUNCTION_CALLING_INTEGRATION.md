# The Augment XML 与 Function Calling 集成指南

## 问题分析

你的 `the-augment.xml` 是一个强大的规则提示词系统，但**不能直接放在 system message 中**，因为：

1. **System Message 的职责**：定义 LLM 的基本角色和行为
2. **Function Calling 的要求**：LLM 必须清晰识别和调用工具
3. **冲突点**：复杂的规则会干扰工具调用逻辑，LLM 会优先遵循规则而不是调用工具

## ✅ 最佳实践：分离架构

### 架构图

```
┌─────────────────────────────────────────────────────────┐
│                    LLM Request                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  System Message (简洁 - 20-50 词)                      │
│  ├─ 角色定义                                            │
│  ├─ 工具调用说明                                        │
│  └─ 基本行为准则                                        │
│                                                         │
│  User Message (包含规则 - 可长)                        │
│  ├─ <augment_rules>                                    │
│  │  └─ the-augment.xml 内容                            │
│  ├─ </augment_rules>                                   │
│  ├─ 用户实际请求                                        │
│  └─ 上下文信息                                          │
│                                                         │
│  Tool Definitions (工具定义)                           │
│  ├─ 工具名称、描述、参数                                │
│  └─ 工具级别的规则                                      │
│                                                         │
└─────────────────────────────────────────────────────────┘
         ↓
    LLM 处理
         ↓
┌─────────────────────────────────────────────────────────┐
│                   LLM Response                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  1. 识别工具调用（优先级高）                            │
│  2. 应用规则约束（优先级中）                            │
│  3. 生成回复（优先级低）                                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## 实现方案

### 方案 1：规则注入到 User Message（推荐）

**优点**：
- ✅ 不干扰工具调用
- ✅ 规则作为上下文而不是强制
- ✅ 最符合 OpenAI 最佳实践

**实现**：

```rust
// src/ai/prompt_builder.rs

use std::fs;

pub struct PromptBuilder {
    system_prompt: String,
    augment_rules: Option<String>,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {
            system_prompt: Self::default_system_prompt(),
            augment_rules: None,
        }
    }

    /// 加载 the-augment.xml
    pub fn load_augment_rules(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string("src/prompts/the-augment.xml")?;
        self.augment_rules = Some(content);
        Ok(self)
    }

    /// 简洁的系统提示词
    fn default_system_prompt() -> String {
        r#"You are The Augster, an elite AI programming partner.

You have access to the following tools:
- code_analysis: Analyze code structure and identify issues
- file_operations: Read, write, and manage files
- terminal_commands: Execute system commands
- project_tools: Analyze project structure and dependencies

When the user asks a question, use the appropriate tools to help them.
Always follow the augment rules provided in the user message."#
            .to_string()
    }

    /// 构建消息列表
    pub fn build_messages(&self, user_request: &str) -> Vec<Message> {
        let mut messages = vec![
            Message {
                role: "system".to_string(),
                content: self.system_prompt.clone(),
            },
        ];

        // 构建用户消息：规则 + 请求
        let user_content = if let Some(rules) = &self.augment_rules {
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
        };

        messages.push(Message {
            role: "user".to_string(),
            content: user_content,
        });

        messages
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}
```

### 方案 2：工具级别的规则应用

**优点**：
- ✅ 工具执行时应用规则
- ✅ 规则与工具紧密结合
- ✅ 更精细的控制

**实现**：

```rust
// src/tools/tool_with_rules.rs

use crate::tools::Tool;
use std::fs;

pub struct RuleAwareTool {
    inner_tool: Box<dyn Tool>,
    augment_rules: String,
}

impl RuleAwareTool {
    pub fn new(tool: Box<dyn Tool>) -> Result<Self, Box<dyn std::error::Error>> {
        let augment_rules = fs::read_to_string("src/prompts/the-augment.xml")?;
        Ok(Self {
            inner_tool: tool,
            augment_rules,
        })
    }

    /// 在执行工具前应用规则检查
    pub fn validate_before_execution(&self, arguments: &serde_json::Value) -> Result<(), String> {
        // 提取规则中的验证逻辑
        // 例如：检查 Security_Awareness, Impact_Awareness 等
        
        // 示例：检查是否涉及安全敏感操作
        if self.is_security_sensitive(arguments) {
            // 应用安全规则
            self.apply_security_rules(arguments)?;
        }

        Ok(())
    }

    fn is_security_sensitive(&self, _args: &serde_json::Value) -> bool {
        // 根据规则检查是否涉及安全敏感操作
        true
    }

    fn apply_security_rules(&self, _args: &serde_json::Value) -> Result<(), String> {
        // 应用 Security_Awareness 规则
        Ok(())
    }
}
```

### 方案 3：对话历史中的规则确认

**优点**：
- ✅ 一次性加载规则
- ✅ 减少每次请求的 token 消耗
- ✅ 适合长对话

**实现**：

```rust
// 在对话开始时
let mut messages = vec![
    Message {
        role: "system".to_string(),
        content: "You are The Augster, an elite AI programming partner.".to_string(),
    },
];

// 第一条消息：加载规则
messages.push(Message {
    role: "user".to_string(),
    content: format!(
        "Please acknowledge that you understand these augment rules:\n\n{}",
        augment_rules
    ),
});

// LLM 确认
messages.push(Message {
    role: "assistant".to_string(),
    content: "I understand and will follow the augment rules throughout our conversation.".to_string(),
});

// 后续消息：只包含用户请求
messages.push(Message {
    role: "user".to_string(),
    content: user_request.to_string(),
});
```

## 集成到你的项目

### 步骤 1：创建 PromptBuilder 模块

```bash
# 创建新文件
touch src/ai/prompt_builder.rs
```

### 步骤 2：更新 src/ai/mod.rs

```rust
pub mod prompt_builder;

pub use prompt_builder::{PromptBuilder, Message};
```

### 步骤 3：在 LLM 客户端中使用

```rust
// src/ai/client.rs

use crate::ai::PromptBuilder;

pub async fn chat_with_augment(
    &self,
    user_request: &str,
    tools: Vec<ToolDefinition>,
) -> Result<String, Box<dyn std::error::Error>> {
    // 构建提示词
    let builder = PromptBuilder::new()
        .load_augment_rules()?;
    
    let messages = builder.build_messages(user_request);

    // 调用 LLM
    let response = self.chat_completion(messages, tools).await?;

    Ok(response)
}
```

### 步骤 4：在应用中使用

```rust
// src/app.rs

pub async fn handle_user_message(&mut self, message: String) -> Result<(), Box<dyn std::error::Error>> {
    // 使用带规则的 LLM 调用
    let response = self.llm_client
        .chat_with_augment(&message, self.get_available_tools())
        .await?;

    self.add_assistant_message(response);
    Ok(())
}
```

## 性能优化

### Token 消耗优化

```rust
// 方案 1：缓存规则
lazy_static::lazy_static! {
    static ref AUGMENT_RULES: String = {
        std::fs::read_to_string("src/prompts/the-augment.xml")
            .unwrap_or_default()
    };
}

// 方案 2：压缩规则
pub fn compress_augment_rules(rules: &str) -> String {
    rules
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("<!--"))
        .collect::<Vec<_>>()
        .join("\n")
}

// 方案 3：分阶段加载规则
pub enum RuleLevel {
    Minimal,      // 只加载核心规则
    Standard,     // 标准规则
    Complete,     // 完整规则
}

pub fn load_rules_by_level(level: RuleLevel) -> String {
    match level {
        RuleLevel::Minimal => extract_core_rules(),
        RuleLevel::Standard => extract_standard_rules(),
        RuleLevel::Complete => std::fs::read_to_string("src/prompts/the-augment.xml").unwrap(),
    }
}
```

## 验证检查清单

- [ ] 系统提示词简洁（<100 词）
- [ ] 规则注入到用户消息
- [ ] 工具调用正常工作
- [ ] 规则被正确应用
- [ ] Token 消耗在可接受范围内
- [ ] 测试多个 LLM 提供商（OpenAI, Claude, Gemini）
- [ ] 验证工具调用优先级高于规则

## 测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder_with_rules() {
        let builder = PromptBuilder::new()
            .load_augment_rules()
            .unwrap();

        let messages = builder.build_messages("What's wrong with this code?");

        // 验证系统消息简洁
        assert!(messages[0].content.len() < 500);

        // 验证用户消息包含规则
        assert!(messages[1].content.contains("<augment_rules>"));

        // 验证用户请求在规则之后
        assert!(messages[1].content.contains("What's wrong with this code?"));
    }

    #[test]
    fn test_tool_calling_with_rules() {
        // 测试工具调用是否正常工作
        // 即使有规则也应该能调用工具
    }
}
```

## 常见问题

### Q: 规则会不会被忽略？
A: 不会。规则在用户消息中，LLM 会看到并应用。但工具调用的优先级更高。

### Q: 如何确保规则被应用？
A: 
1. 在规则中明确说明工具调用时的行为
2. 在工具执行时验证规则
3. 在 LLM 响应后检查是否遵循了规则

### Q: Token 消耗会不会很大？
A: 是的，the-augment.xml 很长。解决方案：
1. 使用缓存
2. 只在需要时加载完整规则
3. 使用分阶段规则加载

### Q: 如何处理多轮对话？
A: 
- 第一轮：加载完整规则
- 后续轮：只在用户消息中提醒关键规则
- 或者在对话开始时确认规则，后续轮次不重复

## 参考资源

- [OpenAI Function Calling 最佳实践](https://platform.openai.com/docs/guides/function-calling)
- [Claude Tool Use 指南](https://docs.anthropic.com/claude/reference/tool-use)
- [LangChain Prompt 管理](https://python.langchain.com/docs/modules/model_io/prompts/)
- [Exa 研究：LLM 提示词工程](https://exa.ai)

## 总结

✅ **系统消息**：简洁，定义角色和工具
✅ **用户消息**：包含规则和请求
✅ **工具定义**：清晰的工具说明
✅ **执行时**：应用规则验证

这样既能保证工具调用正常工作，又能确保规则被正确应用。
