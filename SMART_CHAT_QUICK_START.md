# 智能聊天显示系统 - 快速开始

## 3 步集成

### 步骤 1：在 App 中使用 SmartChatDisplay

```rust
use crate::ui::SmartChatDisplay;

pub struct App {
    pub smart_display: SmartChatDisplay,
    // ... 其他字段
}

impl App {
    pub fn new() -> Self {
        Self {
            smart_display: SmartChatDisplay::new(),
            // ...
        }
    }
}
```

### 步骤 2：添加消息

```rust
use crate::ui::{SmartMessage, MessageRole, MessageType};

// 添加用户消息
let user_msg = SmartMessage::new(
    MessageRole::User,
    "解释一下 Rust 的所有权".to_string()
).with_type(MessageType::Question);

app.smart_display.add_message(user_msg);

// 添加 AI 回复
let ai_msg = SmartMessage::new(
    MessageRole::Assistant,
    "Rust 的所有权是...".to_string()
).with_type(MessageType::Explanation);

app.smart_display.add_message(ai_msg);
```

### 步骤 3：处理流式响应

```rust
// 开始流式响应
pub async fn stream_response(&mut self, prompt: &str) {
    // 显示思考过程
    self.smart_display.show_thinking("分析问题中...".to_string());
    
    // 流式接收数据
    while let Some(chunk) = stream.next().await {
        self.smart_display.add_streaming_chunk(&chunk);
    }
    
    // 完成流式响应
    if let Some(content) = self.smart_display.finalize_streaming() {
        let msg = SmartMessage::new(
            MessageRole::Assistant,
            content
        );
        self.smart_display.add_message(msg);
    }
    
    // 隐藏思考过程
    self.smart_display.hide_thinking();
    
    // 生成建议
    self.smart_display.generate_suggestions(vec![
        "解释这个概念".to_string(),
        "给出代码示例".to_string(),
        "对比不同方案".to_string(),
    ]);
}
```

## 核心功能

### 思考过程显示

```rust
// 显示思考
display.show_thinking("分析中...".to_string());

// 切换显示/隐藏
display.toggle_thinking();

// 隐藏思考
display.hide_thinking();
```

### 建议提示

```rust
// 生成建议
display.generate_suggestions(vec![
    "解释".to_string(),
    "示例".to_string(),
    "对比".to_string(),
]);

// 隐藏建议
display.hide_suggestions();
```

### 流式响应

```rust
// 添加块
display.add_streaming_chunk("Hello ");
display.add_streaming_chunk("World");

// 完成
let content = display.finalize_streaming();
```

### 消息管理

```rust
// 添加消息
display.add_message(message);

// 获取最后一条
let last = display.get_last_message();

// 消息数量
let count = display.message_count();

// 清空
display.clear();
```

### 滚动

```rust
// 滚动到底部
display.scroll_to_bottom();

// 向上滚动
display.scroll_up(3);

// 向下滚动
display.scroll_down(2);
```

## 消息类型

```rust
pub enum MessageType {
    Question,      // 用户问题
    Code,          // 代码块
    Explanation,   // 解释
    Error,         // 错误
    Suggestion,    // 建议
    Thinking,      // 思考过程
    Default,       // 默认
}
```

## 快捷键建议

| 快捷键 | 功能 |
|--------|------|
| `T` | 切换思考过程 |
| `1-4` | 选择建议 |
| `↑/↓` | 浏览历史 |
| `Ctrl+C` | 复制消息 |

## 完整示例

```rust
pub async fn handle_ai_response(&mut self, response: String) {
    // 1. 显示思考过程
    self.smart_display.show_thinking("处理响应中...".to_string());
    
    // 2. 添加 AI 消息
    let msg = SmartMessage::new(
        MessageRole::Assistant,
        response
    ).with_type(MessageType::Explanation);
    
    self.smart_display.add_message(msg);
    
    // 3. 隐藏思考
    self.smart_display.hide_thinking();
    
    // 4. 生成建议
    self.smart_display.generate_suggestions(vec![
        "深入解释".to_string(),
        "代码示例".to_string(),
        "最佳实践".to_string(),
        "相关资源".to_string(),
    ]);
    
    // 5. 获取统计
    let stats = self.smart_display.get_stats();
    println!("总消息数: {}", stats.total_messages);
}
```

## 性能优化

### 消息缓存

```rust
// 缓存渲染内容
display.cache_render(0, rendered_text);

// 获取缓存
if let Some(cached) = display.get_cached_render(0) {
    // 使用缓存
}
```

### 脏标记

```rust
// 标记为脏（需要重新渲染）
display.mark_dirty(0);

// 标记所有为脏
display.mark_all_dirty();

// 检查是否脏
if display.is_dirty(0) {
    // 重新渲染
}
```

## 下一步

1. 集成到 UI 渲染系统
2. 实现快捷键处理
3. 添加消息持久化
4. 优化性能

## 相关文件

- `src/ui/smart_chat_display.rs` - 核心实现
- `SMART_CHAT_DISPLAY_GUIDE.md` - 完整指南
- `src/ui/mod.rs` - 模块导出
