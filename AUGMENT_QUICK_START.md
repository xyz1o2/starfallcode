# The Augment XML 快速集成指南

## 问题回顾

你的 `the-augment.xml` 是一个强大的规则系统，但**不能直接放在 system message 中**，否则会干扰 LLM 的 function calling 功能。

## ✅ 解决方案（3 步）

### 步骤 1：使用 PromptBuilder

```rust
use crate::ai::PromptBuilder;

// 创建构建器并加载规则
let builder = PromptBuilder::new()
    .load_augment_rules()?;

// 构建消息
let messages = builder.build_messages("用户请求内容");
```

### 步骤 2：发送给 LLM（带工具定义）

```rust
// messages 包含：
// [0] System Message (简洁)
// [1] User Message (包含规则 + 请求)

let response = llm_client.chat_with_tools(
    messages,
    tools_definitions
).await?;
```

### 步骤 3：验证工作

- ✅ 工具调用正常工作
- ✅ 规则被应用
- ✅ LLM 同时满足两者

## 代码示例

### 完整示例

```rust
// src/app.rs

use crate::ai::PromptBuilder;

pub async fn handle_user_message(&mut self, message: String) -> Result<()> {
    // 1. 构建提示词
    let builder = PromptBuilder::new()
        .load_augment_rules()?;
    
    let messages = builder.build_messages(&message);

    // 2. 获取可用工具
    let tools = self.get_available_tools();

    // 3. 调用 LLM
    let response = self.llm_client
        .chat_with_tools(messages, tools)
        .await?;

    // 4. 处理响应
    self.add_assistant_message(response);
    Ok(())
}
```

### 带规则确认的示例

```rust
// 如果想在对话开始时确认规则
let builder = PromptBuilder::new()
    .load_augment_rules()?;

let messages = builder.build_messages_with_confirmation(
    "用户的第一个请求"
);

// messages 会包含：
// [0] System Message
// [1] User: "请确认你理解这些规则..."
// [2] Assistant: "我理解并将遵循规则"
// [3] User: "用户的第一个请求"
```

### 压缩规则以节省 Token

```rust
use crate::ai::RulesCompressor;

let rules = std::fs::read_to_string("src/prompts/the-augment.xml")?;
let compressed = RulesCompressor::compress(&rules);

// 获取压缩率
let ratio = RulesCompressor::compression_ratio(&rules, &compressed);
println!("压缩率: {:.1}%", ratio);
```

## 架构对比

### ❌ 错误做法（会干扰工具调用）

```
System Message:
  - 角色定义
  - 规则提示词（the-augment.xml）  ← 问题！
  - 工具说明

User Message:
  - 用户请求
```

### ✅ 正确做法（工具调用正常）

```
System Message:
  - 角色定义
  - 工具说明

User Message:
  - <augment_rules>
      the-augment.xml 内容
    </augment_rules>
  - 用户请求
```

## 关键点

| 方面 | 说明 |
|------|------|
| **System Message** | 简洁（<100 词），只定义角色和工具 |
| **User Message** | 包含规则和用户请求 |
| **工具定义** | 清晰的工具说明 |
| **优先级** | 工具调用 > 规则应用 > 文本生成 |

## 性能优化

### Token 消耗

- 完整规则：~4000 tokens
- 压缩后：~2000 tokens（50% 节省）
- 核心规则：~800 tokens

### 建议

1. **首次对话**：使用完整规则
2. **长对话**：使用规则确认方式
3. **Token 紧张**：使用压缩规则

## 测试检查清单

- [ ] 加载 the-augment.xml 成功
- [ ] PromptBuilder 构建消息成功
- [ ] 消息格式正确（System + User）
- [ ] 工具调用正常工作
- [ ] 规则被应用
- [ ] Token 消耗在预期范围内
- [ ] 多个 LLM 提供商都支持

## 常见问题

**Q: 为什么不能把规则放在 system message？**
A: System message 用于定义 LLM 的基本行为。复杂的规则会干扰 LLM 的工具调用逻辑，导致 LLM 优先遵循规则而不是调用工具。

**Q: 规则会不会被忽略？**
A: 不会。规则在用户消息中，LLM 会看到并应用。但工具调用的优先级更高。

**Q: 如何确保规则被应用？**
A: 
1. 在规则中明确说明工具调用时的行为
2. 在工具执行时验证规则
3. 在 LLM 响应后检查是否遵循了规则

**Q: 长对话中规则会不会被遗忘？**
A: 可能会。解决方案：
- 在对话开始时确认规则
- 在关键时刻重复提醒规则
- 使用规则确认方式

## 下一步

1. 集成 PromptBuilder 到你的应用
2. 测试工具调用是否正常
3. 验证规则是否被应用
4. 优化 token 消耗

## 参考资源

- `AUGMENT_FUNCTION_CALLING_INTEGRATION.md` - 完整指南
- `src/ai/prompt_builder.rs` - 实现代码
- OpenAI Function Calling 最佳实践
- Anthropic Tool Use 指南
