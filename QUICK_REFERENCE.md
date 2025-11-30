# 快速参考指南

## 核心模块位置

| 模块 | 文件 | 主要类型 |
|------|------|--------|
| **ConversationEngine** | `src/core/conversation_engine.rs` | `ConversationEngine`, `UserIntent`, `ConversationContext` |
| **SmartChatDisplay** | `src/ui/smart_chat_display.rs` | `SmartChatDisplay`, `SmartMessage`, `MessageType` |
| **PromptBuilder** | `src/ai/prompt_builder.rs` | `PromptBuilder`, `Message`, `RulesCompressor` |
| **ToolRegistry** | `src/tools/tool_registry.rs` | `ToolRegistry`, `ToolCall`, `ToolResult` |
| **CodeModificationDetector** | `src/ai/code_modification.rs` | `AICodeModificationDetector`, `CodeDiff` |
| **FileSearchEngine** | `src/ui/file_search.rs` | `FileSearchEngine`, `SearchResult` |

## 关键 API

### ConversationEngine

```rust
// 创建引擎
let mut engine = ConversationEngine::new();

// 处理用户输入
let context = engine.process_input("@src/main.rs What's wrong?".to_string());

// 处理 LLM 响应
let processed = engine.process_response("Here's the issue...");

// 获取历史
let history = engine.get_history();
```

### SmartChatDisplay

```rust
// 创建显示
let mut display = SmartChatDisplay::new();

// 添加消息
display.add_message(SmartMessage::new(
    MessageRole::User,
    "Hello".to_string()
));

// 显示思考过程
display.show_thinking("Analyzing...".to_string());

// 生成建议
display.generate_suggestions(vec![
    "解释".to_string(),
    "示例".to_string(),
]);

// 渲染
display.render_card_style(frame, area, &theme);
```

### PromptBuilder

```rust
// 创建构建器
let builder = PromptBuilder::new()
    .load_augment_rules()?;

// 构建消息
let messages = builder.build_messages("用户请求");

// 压缩规则
let compressed = RulesCompressor::compress(&rules);
```

## 常见任务

### 任务：识别用户意图

```rust
let intent = IntentRecognizer::recognize("@src/main.rs 这个文件有什么问题");
// 返回: UserIntent::FileMention { paths: ["src/main.rs"], query: "这个文件有什么问题" }
```

### 任务：加载文件内容

```rust
let context = ContextManager::build("@src/main.rs", &intent);
// context.files 包含文件内容
```

### 任务：调用 LLM

```rust
let response = llm_client.chat(
    messages,
    tool_registry.list_definitions()
).await?;
```

### 任务：检测代码修改

```rust
let modifications = engine.process_response(&response);
// modifications.modifications 包含检测到的修改
```

### 任务：显示聊天

```rust
for msg in &messages {
    let smart_msg = SmartMessage::new(msg.role, msg.content.clone());
    display.add_message(smart_msg);
}
display.render_card_style(f, area, &theme);
```

## 编译和运行

```bash
# 检查编译
cargo check

# 构建
cargo build

# 运行
cargo run

# 测试
cargo test
```

## 文件结构

```
src/
├── app.rs                    # 主应用
├── main.rs                   # 入口点
├── events/
│   └── handler.rs           # 事件处理
├── core/
│   ├── mod.rs
│   └── conversation_engine.rs  # 对话引擎 ⭐
├── ui/
│   ├── mod.rs
│   ├── smart_chat_display.rs   # 智能显示 ⭐
│   └── file_search.rs
├── ai/
│   ├── mod.rs
│   ├── client.rs            # LLM 客户端
│   ├── prompt_builder.rs    # 规则构建 ⭐
│   └── code_modification.rs
└── tools/
    ├── mod.rs
    ├── tool.rs
    └── tool_registry.rs     # 工具系统 ⭐
```

## 关键数据流

```
用户输入
  ↓
ConversationEngine.process_input()
  ↓
IntentRecognizer.recognize()
  ↓
ContextManager.build()
  ↓
LLMClient.chat()
  ↓
ConversationEngine.process_response()
  ↓
SmartChatDisplay.add_message()
  ↓
UI 渲染
```

## 调试技巧

### 打印意图
```rust
println!("Intent: {:?}", context.intent);
```

### 打印消息
```rust
println!("Messages: {:?}", messages);
```

### 打印响应
```rust
println!("Response: {}", response);
```

### 查看修改
```rust
println!("Modifications: {:?}", processed.modifications);
```

## 常见错误

| 错误 | 原因 | 解决 |
|------|------|------|
| `borrow of moved value` | 值被移动后再使用 | 使用引用或 clone |
| `trait bound not satisfied` | 类型不满足 trait | 检查类型实现 |
| `cannot borrow as mutable` | 可变借用冲突 | 限制借用作用域 |
| `expected struct, found enum` | 类型不匹配 | 检查模式匹配 |

## 性能优化

- 使用缓存避免重复计算
- 异步处理 LLM 请求
- 只渲染可见的 UI 元素
- 使用流式响应处理大文本

## 下一步

1. 完成优先级 1 集成 (12-20 小时)
2. 完成优先级 2 功能完善 (8-12 小时)
3. 完成优先级 3 增强功能 (可选)
4. 性能优化和测试
5. 部署和发布

---

**详见**: `INTEGRATION_PLAN.md` 获取完整的实现计划
