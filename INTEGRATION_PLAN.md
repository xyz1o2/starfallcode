# ConversationEngine 集成计划

## 优先级 1：核心集成（必须）

### 任务 1.1：在 App 中集成 ConversationEngine

**文件**: `src/app.rs`

**步骤**:
1. 导入 ConversationEngine
```rust
use crate::core::ConversationEngine;
```

2. 在 App 结构体中添加字段
```rust
pub struct App {
    // ... 现有字段
    pub conversation_engine: ConversationEngine,
}
```

3. 在 `App::new()` 中初始化
```rust
impl App {
    pub fn new() -> Self {
        Self {
            // ... 现有初始化
            conversation_engine: ConversationEngine::new(),
        }
    }
}
```

**预期结果**: App 现在拥有一个 ConversationEngine 实例

---

### 任务 1.2：连接 UI 事件处理

**文件**: `src/events/handler.rs`

**步骤**:
1. 在 Enter 键处理中调用 ConversationEngine
```rust
KeyCode::Enter => {
    if !app.input_text.is_empty() {
        // 处理用户输入
        let context = app.conversation_engine.process_input(
            app.input_text.clone()
        );
        
        // 保存到聊天历史
        app.chat_history.push(context);
        
        // 清空输入
        app.input_text.clear();
        app.input_cursor = 0;
    }
    AppAction::SubmitChat
}
```

2. 在意图识别后处理不同的意图类型
```rust
match &context.intent {
    UserIntent::FileMention { paths, query } => {
        // 加载文件内容
        // 注入到 AI 提示
    }
    UserIntent::Command { name, args } => {
        // 执行命令
    }
    UserIntent::Chat { query, .. } => {
        // 发送给 LLM
    }
    // ... 其他意图类型
}
```

**预期结果**: 用户输入被正确识别和分类

---

### 任务 1.3：集成 LLM 客户端

**文件**: `src/app.rs`

**步骤**:
1. 添加 LLM 客户端字段
```rust
pub struct App {
    // ... 现有字段
    pub llm_client: LLMClient,
}
```

2. 在处理 Chat 意图时调用 LLM
```rust
UserIntent::Chat { query, .. } => {
    // 使用 PromptBuilder 构建消息
    let messages = app.prompt_builder.build_messages(&query);
    
    // 调用 LLM
    let response = app.llm_client.chat(
        messages,
        app.tool_registry.list_definitions()
    ).await?;
    
    // 处理响应
    let processed = app.conversation_engine.process_response(&response);
}
```

**预期结果**: LLM 能够接收用户请求并返回响应

---

### 任务 1.4：集成 SmartChatDisplay

**文件**: `src/ui/mod.rs`

**步骤**:
1. 在 App 中添加 SmartChatDisplay
```rust
pub struct App {
    // ... 现有字段
    pub smart_display: SmartChatDisplay,
}
```

2. 在渲染时使用 SmartChatDisplay
```rust
fn render_chat_area(&self, f: &mut Frame, area: Rect) {
    // 获取消息
    let messages = self.chat_history.get_messages();
    
    // 添加到 SmartChatDisplay
    for msg in messages {
        let smart_msg = SmartMessage::new(
            msg.role,
            msg.content
        ).with_type(msg.message_type);
        
        self.smart_display.add_message(smart_msg);
    }
    
    // 渲染
    self.smart_display.render_card_style(f, area, &self.theme);
}
```

3. 显示建议提示
```rust
if let Some(suggestions) = &processed_response.suggestions {
    self.smart_display.generate_suggestions(suggestions.clone());
}
```

**预期结果**: 聊天历史以现代化风格显示，包含建议提示

---

## 优先级 2：功能完善（必须）

### 任务 2.1：完善意图识别逻辑

**文件**: `src/core/conversation_engine.rs`

**改进方向**:
1. 增强 @mention 检测
```rust
// 支持更多格式
// @src/main.rs
// @./config.json
// @../docs/README.md
```

2. 改进命令识别
```rust
// 支持参数解析
// /modify-file src/main.rs <content>
// /create-file path/to/file.rs <content>
```

3. 增强关键词检测
```rust
// 更精确的关键词匹配
// 支持多语言关键词
// 支持上下文感知
```

**预期结果**: 意图识别准确率 > 95%

---

### 任务 2.2：实现上下文加载

**文件**: `src/core/conversation_engine.rs`

**步骤**:
1. 实现文件加载
```rust
impl ContextManager {
    pub async fn load_files(&self, paths: &[String]) -> Result<Vec<FileContent>> {
        let mut files = Vec::new();
        for path in paths {
            let content = std::fs::read_to_string(path)?;
            files.push(FileContent {
                path: path.clone(),
                content,
                language: detect_language(path),
                line_count: content.lines().count(),
            });
        }
        Ok(files)
    }
}
```

2. 实现规则注入
```rust
pub async fn build(&self, input: &str, intent: &UserIntent) -> ConversationContext {
    let mut context = ConversationContext::new(input.to_string(), intent.clone());
    
    // 加载文件
    if let UserIntent::FileMention { paths, .. } = intent {
        context = context.with_files(self.load_files(paths).await?);
    }
    
    // 加载规则
    let rules = load_augment_rules()?;
    context = context.with_rules(rules);
    
    Ok(context)
}
```

**预期结果**: 上下文完整，包含文件内容和规则

---

### 任务 2.3：完善响应处理

**文件**: `src/core/conversation_engine.rs`

**步骤**:
1. 改进代码修改检测
```rust
fn extract_modifications(response: &str) -> Vec<CodeModification> {
    let mut modifications = Vec::new();
    
    // 检测 create file
    // 检测 modify
    // 检测 delete
    // 检测代码块
    
    modifications
}
```

2. 改进建议提取
```rust
fn extract_suggestions(response: &str) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    // 检测"建议"关键词
    // 检测"最佳实践"
    // 检测"示例"
    // 检测"参考"
    
    suggestions
}
```

**预期结果**: 能够准确提取修改指令和建议

---

### 任务 2.4：添加错误处理

**文件**: `src/core/conversation_engine.rs`

**步骤**:
1. 定义错误类型
```rust
#[derive(Debug)]
pub enum ConversationError {
    FileNotFound(String),
    InvalidIntent,
    ProcessingError(String),
    LLMError(String),
}
```

2. 添加错误处理
```rust
impl ConversationEngine {
    pub fn process_input(&mut self, input: String) -> Result<ConversationContext, ConversationError> {
        if input.is_empty() {
            return Err(ConversationError::InvalidIntent);
        }
        
        let intent = IntentRecognizer::recognize(&input);
        let context = ContextManager::build(&input, &intent)
            .map_err(|e| ConversationError::ProcessingError(e.to_string()))?;
        
        Ok(context)
    }
}
```

**预期结果**: 完善的错误处理和用户反馈

---

## 实现顺序

```
1. 任务 1.1 - App 集成 (1-2 小时)
   ↓
2. 任务 1.2 - 事件处理 (1-2 小时)
   ↓
3. 任务 1.3 - LLM 客户端 (2-3 小时)
   ↓
4. 任务 1.4 - SmartChatDisplay (1-2 小时)
   ↓
5. 任务 2.1 - 完善意图识别 (2-3 小时)
   ↓
6. 任务 2.2 - 上下文加载 (2-3 小时)
   ↓
7. 任务 2.3 - 响应处理 (2-3 小时)
   ↓
8. 任务 2.4 - 错误处理 (1-2 小时)
```

**总预计时间**: 12-20 小时

---

## 验证清单

- [ ] 编译成功，无错误
- [ ] 用户输入被正确识别
- [ ] LLM 能够接收请求
- [ ] 聊天历史正确显示
- [ ] 建议提示正常工作
- [ ] 错误处理完善
- [ ] 单元测试通过
- [ ] 集成测试通过

---

## 相关文件

- `src/app.rs` - 主应用结构
- `src/events/handler.rs` - 事件处理
- `src/core/conversation_engine.rs` - 对话引擎
- `src/ui/mod.rs` - UI 渲染
- `src/ai/client.rs` - LLM 客户端
