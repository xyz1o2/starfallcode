# 智能聊天显示系统 - 基于 Gemini CLI 最佳实践

## 问题分析

你的聊天显示不够智能，缺少以下特性：
- ❌ 对话上下文理解
- ❌ 思考过程展示
- ❌ 建议提示
- ❌ 对话历史管理
- ❌ 流式响应优化

## ✅ Gemini CLI 的最佳实践

### 1. 思考过程展示（Thinking Mode）

**Gemini 2.0 的做法**：
```
用户: 解释一下 AI 如何工作
AI: 💭 思考中...
    [显示推理过程]
    
最终回答: [清晰的答案]
```

**实现**：
```rust
pub struct ThinkingDisplay {
    thinking_content: String,
    is_visible: bool,
    collapsed: bool,
}

impl ThinkingDisplay {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.is_visible {
            // 显示 "💭 思考中..." 或 "💭 思考完成"
            // 支持展开/折叠
        }
    }
}
```

### 2. 建议提示（Suggestions）

**Gemini 的做法**：
```
┌─────────────────────────────────────┐
│ 💬 建议提示:                         │
│ • 解释这个概念                       │
│ • 给出代码示例                       │
│ • 对比不同方案                       │
└─────────────────────────────────────┘
```

**实现**：
```rust
pub struct SuggestionBar {
    suggestions: Vec<String>,
    selected_index: usize,
}

impl SuggestionBar {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // 显示 3-4 个建议
        // 支持快捷键选择 (1-4)
    }
}
```

### 3. 对话历史管理

**Gemini CLI 的做法**：
```bash
# 保存对话
gemini --save conversation.json "开始讨论"

# 加载对话
gemini --load conversation.json "继续讨论"

# 显示历史
gemini --history
```

**实现**：
```rust
pub struct ConversationManager {
    history: Vec<Message>,
    current_session: String,
    auto_save: bool,
}

impl ConversationManager {
    pub fn save_session(&self, path: &str) -> Result<()> {
        // 保存为 JSON
    }
    
    pub fn load_session(&mut self, path: &str) -> Result<()> {
        // 加载历史
    }
}
```

### 4. 流式响应优化

**Gemini 的做法**：
```
用户: 写一个 Rust 函数
AI: 正在生成...
    fn hello() {
        println!("Hello");
    }
    
    [继续生成...]
```

**实现**：
```rust
pub struct StreamingDisplay {
    buffer: String,
    chunk_count: usize,
    last_update: Instant,
}

impl StreamingDisplay {
    pub fn add_chunk(&mut self, chunk: &str) {
        self.buffer.push_str(chunk);
        self.chunk_count += 1;
        
        // 每 50ms 更新一次 UI（避免闪烁）
        if self.last_update.elapsed() > Duration::from_millis(50) {
            self.render_update();
            self.last_update = Instant::now();
        }
    }
}
```

### 5. 上下文感知的显示

**Gemini 的做法**：
```
用户: 这个代码有问题吗?
     [代码块]

AI: 我看到了 3 个问题:
    1. 缺少错误处理
    2. 性能问题
    3. 内存泄漏
    
    [详细解释]
```

**实现**：
```rust
pub struct ContextAwareDisplay {
    message_type: MessageType,  // code, question, explanation
    code_blocks: Vec<CodeBlock>,
    issues: Vec<Issue>,
}

#[derive(Debug)]
pub enum MessageType {
    Question,
    Code,
    Explanation,
    Error,
    Suggestion,
}

impl ContextAwareDisplay {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        match self.message_type {
            MessageType::Code => self.render_code_block(frame, area),
            MessageType::Explanation => self.render_explanation(frame, area),
            MessageType::Error => self.render_error(frame, area),
            _ => self.render_default(frame, area),
        }
    }
}
```

## 完整的智能聊天系统架构

```
┌─────────────────────────────────────────────────────┐
│                  Smart Chat Display                 │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌─ 消息头 ──────────────────────────────────────┐ │
│  │ 👤 User [14:30:45] / 🤖 AI [14:30:46]        │ │
│  └──────────────────────────────────────────────┘ │
│                                                     │
│  ┌─ 思考过程 ────────────────────────────────────┐ │
│  │ 💭 思考中... (可折叠)                         │ │
│  │ [推理过程]                                    │ │
│  └──────────────────────────────────────────────┘ │
│                                                     │
│  ┌─ 主要内容 ────────────────────────────────────┐ │
│  │ • 代码块（高亮）                              │ │
│  │ • 列表项（格式化）                            │ │
│  │ • 链接（可点击）                              │ │
│  │ • 表格（对齐）                                │ │
│  └──────────────────────────────────────────────┘ │
│                                                     │
│  ┌─ 建议提示 ────────────────────────────────────┐ │
│  │ 💡 建议: [1] 解释 [2] 示例 [3] 对比          │ │
│  └──────────────────────────────────────────────┘ │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## 实现步骤

### 步骤 1：创建智能显示模块

```rust
// src/ui/smart_chat_display.rs

pub struct SmartChatDisplay {
    messages: Vec<SmartMessage>,
    thinking_display: Option<ThinkingDisplay>,
    suggestion_bar: Option<SuggestionBar>,
    streaming_display: Option<StreamingDisplay>,
}

pub struct SmartMessage {
    role: MessageRole,
    content: String,
    message_type: MessageType,
    timestamp: DateTime<Local>,
    metadata: MessageMetadata,
}

pub struct MessageMetadata {
    has_code: bool,
    code_blocks: Vec<CodeBlock>,
    has_issues: bool,
    issues: Vec<Issue>,
    suggested_actions: Vec<String>,
}
```

### 步骤 2：实现思考过程显示

```rust
impl SmartChatDisplay {
    pub fn show_thinking(&mut self, thinking: String) {
        self.thinking_display = Some(ThinkingDisplay {
            thinking_content: thinking,
            is_visible: true,
            collapsed: false,
        });
    }
    
    pub fn toggle_thinking(&mut self) {
        if let Some(ref mut thinking) = self.thinking_display {
            thinking.collapsed = !thinking.collapsed;
        }
    }
}
```

### 步骤 3：实现建议提示

```rust
impl SmartChatDisplay {
    pub fn generate_suggestions(&mut self, message: &str) {
        let suggestions = vec![
            "解释这个概念".to_string(),
            "给出代码示例".to_string(),
            "对比不同方案".to_string(),
            "提供最佳实践".to_string(),
        ];
        
        self.suggestion_bar = Some(SuggestionBar {
            suggestions,
            selected_index: 0,
        });
    }
}
```

### 步骤 4：优化流式响应

```rust
impl SmartChatDisplay {
    pub fn add_streaming_chunk(&mut self, chunk: &str) {
        if self.streaming_display.is_none() {
            self.streaming_display = Some(StreamingDisplay::new());
        }
        
        if let Some(ref mut display) = self.streaming_display {
            display.add_chunk(chunk);
        }
    }
    
    pub fn finalize_streaming(&mut self) {
        if let Some(display) = self.streaming_display.take() {
            // 将流式内容转换为最终消息
            let final_message = SmartMessage {
                content: display.buffer,
                ..Default::default()
            };
            self.messages.push(final_message);
        }
    }
}
```

## 快捷键设计

| 快捷键 | 功能 |
|--------|------|
| `T` | 切换思考过程显示 |
| `1-4` | 选择建议提示 |
| `↑/↓` | 浏览对话历史 |
| `Ctrl+S` | 保存对话 |
| `Ctrl+L` | 加载对话 |
| `Ctrl+C` | 复制消息 |
| `Ctrl+H` | 显示历史 |

## 性能优化

### 1. 消息缓存
```rust
pub struct MessageCache {
    rendered_messages: HashMap<usize, Vec<Line>>,
    dirty_flags: Vec<bool>,
}
```

### 2. 增量渲染
```rust
pub fn render_incremental(&self, frame: &mut Frame, area: Rect) {
    // 只重新渲染改变的消息
    for (idx, message) in self.messages.iter().enumerate() {
        if self.dirty_flags[idx] {
            self.render_message(frame, area, message);
        }
    }
}
```

### 3. 流式响应优化
```rust
pub fn should_update_ui(&self) -> bool {
    // 每 50ms 更新一次，避免过度渲染
    self.last_update.elapsed() > Duration::from_millis(50)
}
```

## 参考资源

- Gemini CLI 官方文档
- Gradio Chatbot 实现
- LangChain Chat 管理
- Llama Index Chat 接口

## 下一步

1. 实现 `SmartChatDisplay` 模块
2. 集成思考过程显示
3. 添加建议提示系统
4. 优化流式响应
5. 实现对话历史管理
