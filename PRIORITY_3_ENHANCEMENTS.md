# 优先级 3：增强功能（可选）

这些功能在优先级 1 和 2 完成后可以选择性地实现，用于提升用户体验和系统性能。

---

## 任务 3.1：对话历史持久化

### 目标
将对话历史保存到磁盘，支持对话的加载和恢复。

### 实现方案

#### 1. 定义持久化格式

**文件**: `src/core/persistence.rs` (新建)

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct PersistentConversation {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<PersistentMessage>,
    pub metadata: ConversationMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersistentMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub intent_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConversationMetadata {
    pub title: String,
    pub tags: Vec<String>,
    pub model: String,
    pub token_count: usize,
}

impl PersistentConversation {
    /// 保存到文件
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// 从文件加载
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let conversation = serde_json::from_str(&json)?;
        Ok(conversation)
    }

    /// 列出所有对话
    pub fn list_conversations(dir: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut conversations = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                conversations.push(path.to_string_lossy().to_string());
            }
        }
        Ok(conversations)
    }
}
```

#### 2. 集成到 App

**文件**: `src/app.rs`

```rust
use crate::core::persistence::{PersistentConversation, PersistentMessage};

pub struct App {
    // ... 现有字段
    pub conversation_id: String,
    pub conversations_dir: PathBuf,
}

impl App {
    pub fn new() -> Self {
        let conversations_dir = dirs::config_dir()
            .unwrap_or_default()
            .join("starfellcode/conversations");
        
        fs::create_dir_all(&conversations_dir).ok();
        
        Self {
            // ... 现有初始化
            conversation_id: uuid::Uuid::new_v4().to_string(),
            conversations_dir,
        }
    }

    /// 保存当前对话
    pub fn save_conversation(&self) -> Result<(), Box<dyn std::error::Error>> {
        let persistent = PersistentConversation {
            id: self.conversation_id.clone(),
            created_at: chrono::Local::now().to_rfc3339(),
            updated_at: chrono::Local::now().to_rfc3339(),
            messages: self.chat_history.iter().map(|msg| {
                PersistentMessage {
                    role: format!("{:?}", msg.role),
                    content: msg.content.clone(),
                    timestamp: msg.timestamp.to_rfc3339(),
                    intent_type: None,
                }
            }).collect(),
            metadata: ConversationMetadata {
                title: format!("Conversation {}", &self.conversation_id[..8]),
                tags: vec![],
                model: self.current_model.clone(),
                token_count: 0,
            },
        };

        let path = self.conversations_dir.join(format!("{}.json", self.conversation_id));
        persistent.save(&path)?;
        Ok(())
    }

    /// 加载对话
    pub fn load_conversation(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.conversations_dir.join(format!("{}.json", id));
        let persistent = PersistentConversation::load(&path)?;
        
        self.conversation_id = persistent.id;
        // 恢复消息到 chat_history
        
        Ok(())
    }

    /// 列出所有对话
    pub fn list_conversations(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        PersistentConversation::list_conversations(&self.conversations_dir)
    }
}
```

#### 3. 自动保存

**文件**: `src/app.rs`

```rust
impl App {
    pub fn auto_save(&self) {
        // 每 30 秒自动保存一次
        if self.last_save.elapsed() > Duration::from_secs(30) {
            let _ = self.save_conversation();
            self.last_save = Instant::now();
        }
    }
}
```

### 预期结果
- 对话自动保存到 `~/.config/starfellcode/conversations/`
- 支持对话恢复和历史查看
- 每 30 秒自动保存一次

---

## 任务 3.2：上下文压缩

### 目标
在长对话中压缩早期的对话内容，以节省 token 和提高性能。

### 实现方案

#### 1. 定义压缩策略

**文件**: `src/core/context_compression.rs` (新建)

```rust
pub struct ContextCompressor {
    max_tokens: usize,
    compression_threshold: usize,
}

impl ContextCompressor {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            compression_threshold: max_tokens / 2,
        }
    }

    /// 压缩对话历史
    pub fn compress(&self, messages: &[Message]) -> Vec<Message> {
        let total_tokens: usize = messages.iter()
            .map(|m| self.estimate_tokens(&m.content))
            .sum();

        if total_tokens < self.compression_threshold {
            return messages.to_vec();
        }

        let mut compressed = Vec::new();
        let mut current_tokens = 0;

        // 保留最后的消息（最重要）
        for msg in messages.iter().rev() {
            let msg_tokens = self.estimate_tokens(&msg.content);
            if current_tokens + msg_tokens > self.max_tokens {
                break;
            }
            compressed.push(msg.clone());
            current_tokens += msg_tokens;
        }

        // 反转回原始顺序
        compressed.reverse();

        // 如果压缩后的消息太少，添加摘要
        if compressed.len() < 3 {
            let summary = self.create_summary(&messages[..messages.len() - compressed.len()]);
            compressed.insert(0, Message {
                role: MessageRole::System,
                content: summary,
                timestamp: chrono::Local::now(),
            });
        }

        compressed
    }

    /// 估计 token 数
    fn estimate_tokens(&self, text: &str) -> usize {
        // 简单估计：1 token ≈ 4 个字符
        (text.len() + 3) / 4
    }

    /// 创建对话摘要
    fn create_summary(&self, messages: &[Message]) -> String {
        let mut summary = String::from("Previous conversation summary:\n");
        
        // 提取关键信息
        for msg in messages {
            if msg.content.len() > 100 {
                summary.push_str(&format!(
                    "- {}: {}\n",
                    msg.role,
                    &msg.content[..100]
                ));
            }
        }

        summary
    }
}
```

#### 2. 集成到 ConversationEngine

**文件**: `src/core/conversation_engine.rs`

```rust
pub struct ConversationEngine {
    // ... 现有字段
    pub context_compressor: ContextCompressor,
}

impl ConversationEngine {
    pub fn get_compressed_history(&self, max_tokens: usize) -> Vec<Message> {
        let compressor = ContextCompressor::new(max_tokens);
        compressor.compress(&self.conversation_history)
    }
}
```

### 预期结果
- 长对话自动压缩
- Token 使用量减少 30-50%
- 保留关键信息和上下文

---

## 任务 3.3：多轮对话优化

### 目标
优化多轮对话的流程，提高对话质量和响应速度。

### 实现方案

#### 1. 对话状态管理

**文件**: `src/core/conversation_state.rs` (新建)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationState {
    Idle,
    WaitingForUserInput,
    ProcessingInput,
    WaitingForLLMResponse,
    ProcessingResponse,
    AwaitingUserConfirmation,
    Completed,
}

pub struct ConversationSession {
    pub state: ConversationState,
    pub turn_count: usize,
    pub last_user_input: Option<String>,
    pub last_ai_response: Option<String>,
    pub pending_confirmations: Vec<PendingConfirmation>,
}

pub struct PendingConfirmation {
    pub id: String,
    pub action: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Local>,
}

impl ConversationSession {
    pub fn new() -> Self {
        Self {
            state: ConversationState::Idle,
            turn_count: 0,
            last_user_input: None,
            last_ai_response: None,
            pending_confirmations: Vec::new(),
        }
    }

    pub fn next_turn(&mut self) {
        self.turn_count += 1;
        self.state = ConversationState::WaitingForUserInput;
    }

    pub fn add_confirmation(&mut self, action: String, description: String) {
        self.pending_confirmations.push(PendingConfirmation {
            id: uuid::Uuid::new_v4().to_string(),
            action,
            description,
            created_at: chrono::Local::now(),
        });
        self.state = ConversationState::AwaitingUserConfirmation;
    }
}
```

#### 2. 对话优化策略

**文件**: `src/core/conversation_optimizer.rs` (新建)

```rust
pub struct ConversationOptimizer;

impl ConversationOptimizer {
    /// 检测对话是否需要澄清
    pub fn needs_clarification(user_input: &str) -> bool {
        // 检查输入是否过于模糊
        user_input.len() < 5 || user_input.contains("?") && user_input.len() < 20
    }

    /// 生成澄清问题
    pub fn generate_clarification_questions(user_input: &str) -> Vec<String> {
        vec![
            format!("你是指关于 '{}' 的什么方面？", user_input),
            "能否提供更多细节？".to_string(),
            "你想要什么样的帮助？".to_string(),
        ]
    }

    /// 检测对话是否偏离主题
    pub fn is_off_topic(current_topic: &str, new_input: &str) -> bool {
        // 简单的关键词匹配
        let topic_keywords: Vec<&str> = current_topic.split_whitespace().collect();
        let input_keywords: Vec<&str> = new_input.split_whitespace().collect();
        
        let matches = topic_keywords.iter()
            .filter(|kw| input_keywords.contains(kw))
            .count();
        
        matches == 0
    }

    /// 生成对话摘要
    pub fn summarize_conversation(messages: &[Message]) -> String {
        // 提取关键点
        let mut summary = String::new();
        for msg in messages {
            if msg.content.len() > 50 {
                summary.push_str(&format!("- {}\n", &msg.content[..50]));
            }
        }
        summary
    }
}
```

### 预期结果
- 对话状态清晰管理
- 自动澄清模糊输入
- 检测并处理偏离主题的情况
- 生成对话摘要

---

## 任务 3.4：错误恢复机制

### 目标
实现完善的错误恢复机制，提高系统稳定性。

### 实现方案

#### 1. 错误恢复策略

**文件**: `src/core/error_recovery.rs` (新建)

```rust
use std::time::Duration;

#[derive(Debug)]
pub enum RecoveryStrategy {
    Retry { max_attempts: u32, backoff: Duration },
    Fallback { alternative: String },
    Abort { reason: String },
}

pub struct ErrorRecoveryManager {
    max_retries: u32,
    backoff_multiplier: f64,
}

impl ErrorRecoveryManager {
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            backoff_multiplier: 2.0,
        }
    }

    /// 执行带重试的操作
    pub async fn execute_with_retry<F, T>(
        &self,
        mut operation: F,
    ) -> Result<T, String>
    where
        F: FnMut() -> futures::future::BoxFuture<'static, Result<T, String>>,
    {
        let mut attempt = 0;
        let mut backoff = Duration::from_millis(100);

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.max_retries => {
                    attempt += 1;
                    eprintln!("Attempt {} failed: {}. Retrying in {:?}...", attempt, e, backoff);
                    tokio::time::sleep(backoff).await;
                    backoff = Duration::from_millis(
                        (backoff.as_millis() as f64 * self.backoff_multiplier) as u64
                    );
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// 生成错误报告
    pub fn generate_error_report(error: &str, context: &str) -> String {
        format!(
            "Error Report:\n\
             Error: {}\n\
             Context: {}\n\
             Timestamp: {}\n\
             Recovery: Attempted automatic recovery",
            error,
            context,
            chrono::Local::now()
        )
    }

    /// 保存错误日志
    pub fn save_error_log(report: &str, log_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("error_{}.log", chrono::Local::now().timestamp());
        let path = log_dir.join(filename);
        std::fs::write(path, report)?;
        Ok(())
    }
}
```

#### 2. 集成到应用

**文件**: `src/app.rs`

```rust
pub struct App {
    // ... 现有字段
    pub error_recovery: ErrorRecoveryManager,
    pub error_log_dir: PathBuf,
}

impl App {
    pub async fn handle_error(&mut self, error: String, context: &str) {
        // 生成错误报告
        let report = self.error_recovery.generate_error_report(&error, context);
        
        // 保存错误日志
        let _ = self.error_recovery.save_error_log(&report, &self.error_log_dir);
        
        // 显示错误给用户
        self.status_message = format!("Error: {}. Attempting recovery...", error);
        
        // 尝试恢复
        // ...
    }
}
```

### 预期结果
- 自动重试失败的操作
- 指数退避策略
- 详细的错误日志
- 优雅的错误恢复

---

## 实现优先级

```
3.1 对话历史持久化 (4-6 小时)
    ↓
3.2 上下文压缩 (3-4 小时)
    ↓
3.3 多轮对话优化 (4-5 小时)
    ↓
3.4 错误恢复机制 (3-4 小时)
```

**总预计时间**: 14-19 小时

---

## 验证清单

- [ ] 对话正确保存和加载
- [ ] 长对话自动压缩
- [ ] 对话状态正确管理
- [ ] 错误自动重试
- [ ] 错误日志正确记录
- [ ] 系统稳定性提高
- [ ] 单元测试通过

---

## 相关依赖

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
dirs = "5.0"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
```

---

## 注意事项

- 这些功能是可选的，不影响核心功能
- 建议在优先级 1 和 2 完成后再实现
- 每个功能都可以独立实现
- 充分测试以确保稳定性
