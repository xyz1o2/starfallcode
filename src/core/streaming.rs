/// 流式处理模块 - 对应 Gemini CLI 的 StreamEventType
/// 
/// 支持实时流式响应处理，类似 Gemini CLI 的流式事件系统

use tokio::sync::mpsc;

/// 流式事件类型
#[derive(Debug, Clone)]
pub enum StreamEventType {
    /// 普通内容块
    Chunk,
    /// 重试信号 - UI 应该丢弃之前的部分内容
    Retry,
    /// 完成信号
    Complete,
    /// 错误信号
    Error,
}

/// 流式事件
#[derive(Debug, Clone)]
pub struct StreamEvent {
    pub event_type: StreamEventType,
    pub content: String,
    pub metadata: Option<StreamMetadata>,
}

/// 流式事件元数据
#[derive(Debug, Clone)]
pub struct StreamMetadata {
    pub timestamp: i64,
    pub chunk_index: u32,
    pub is_final: bool,
}

impl StreamEvent {
    pub fn chunk(content: impl Into<String>) -> Self {
        Self {
            event_type: StreamEventType::Chunk,
            content: content.into(),
            metadata: None,
        }
    }

    pub fn retry() -> Self {
        Self {
            event_type: StreamEventType::Retry,
            content: String::new(),
            metadata: None,
        }
    }

    pub fn complete(content: impl Into<String>) -> Self {
        Self {
            event_type: StreamEventType::Complete,
            content: content.into(),
            metadata: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            event_type: StreamEventType::Error,
            content: message.into(),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: StreamMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// 流式响应处理器
pub struct StreamHandler {
    sender: mpsc::UnboundedSender<StreamEvent>,
}

impl StreamHandler {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<StreamEvent>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let handler = Self {
            sender,
        };
        (handler, receiver)
    }

    pub fn send_event(&self, event: StreamEvent) -> Result<(), String> {
        self.sender.send(event).map_err(|e| e.to_string())
    }

    pub fn send_chunk(&self, content: impl Into<String>) -> Result<(), String> {
        self.send_event(StreamEvent::chunk(content))
    }

    pub fn send_retry(&self) -> Result<(), String> {
        self.send_event(StreamEvent::retry())
    }

    pub fn send_complete(&self, content: impl Into<String>) -> Result<(), String> {
        self.send_event(StreamEvent::complete(content))
    }

    pub fn send_error(&self, message: impl Into<String>) -> Result<(), String> {
        self.send_event(StreamEvent::error(message))
    }
}

impl Default for StreamHandler {
    fn default() -> Self {
        let (handler, _) = Self::new();
        handler
    }
}

/// 流式响应缓冲区 - 累积流式内容
pub struct StreamBuffer {
    content: String,
    chunks: Vec<String>,
    retry_count: u32,
}

impl StreamBuffer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            chunks: Vec::new(),
            retry_count: 0,
        }
    }

    pub fn append(&mut self, chunk: &str) {
        self.content.push_str(chunk);
        self.chunks.push(chunk.to_string());
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.chunks.clear();
    }

    pub fn on_retry(&mut self) {
        self.retry_count += 1;
        self.clear();
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn get_chunks(&self) -> &[String] {
        &self.chunks
    }

    pub fn get_retry_count(&self) -> u32 {
        self.retry_count
    }
}

impl Default for StreamBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_event_creation() {
        let chunk = StreamEvent::chunk("hello");
        assert!(matches!(chunk.event_type, StreamEventType::Chunk));
        assert_eq!(chunk.content, "hello");

        let retry = StreamEvent::retry();
        assert!(matches!(retry.event_type, StreamEventType::Retry));

        let complete = StreamEvent::complete("done");
        assert!(matches!(complete.event_type, StreamEventType::Complete));
    }

    #[test]
    fn test_stream_buffer() {
        let mut buffer = StreamBuffer::new();
        buffer.append("hello");
        buffer.append(" ");
        buffer.append("world");

        assert_eq!(buffer.get_content(), "hello world");
        assert_eq!(buffer.get_chunks().len(), 3);

        buffer.on_retry();
        assert_eq!(buffer.get_retry_count(), 1);
        assert_eq!(buffer.get_content(), "");
    }

    #[test]
    fn test_stream_handler() {
        let (handler, mut receiver) = StreamHandler::new();
        
        handler.send_chunk("test").unwrap();
        
        if let Ok(Some(event)) = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { Ok(receiver.recv().await) })
        {
            assert!(matches!(event.event_type, StreamEventType::Chunk));
            assert_eq!(event.content, "test");
        }
    }
}
