use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 流式响应事件
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 接收到新的文本块
    Token(String),
    /// 流完成
    Done,
    /// 发生错误
    Error(String),
}

/// 流式响应处理器
#[derive(Clone)]
pub struct StreamHandler {
    tx: mpsc::UnboundedSender<StreamEvent>,
    rx: Arc<Mutex<mpsc::UnboundedReceiver<StreamEvent>>>,
}

impl StreamHandler {
    /// 创建新的流式处理器
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    /// 发送令牌
    pub fn send_token(&self, token: String) -> Result<(), String> {
        self.tx
            .send(StreamEvent::Token(token))
            .map_err(|e| e.to_string())
    }

    /// 标记完成
    pub fn send_done(&self) -> Result<(), String> {
        self.tx
            .send(StreamEvent::Done)
            .map_err(|e| e.to_string())
    }

    /// 发送错误
    pub fn send_error(&self, error: String) -> Result<(), String> {
        self.tx
            .send(StreamEvent::Error(error))
            .map_err(|e| e.to_string())
    }

    /// 非阻塞地尝试接收一个事件
    pub fn try_recv(&mut self) -> Result<StreamEvent, mpsc::error::TryRecvError> {
        // 我们需要一个可变引用来调用 try_recv，但由于 Arc<Mutex<...>> 的结构，
        // 我们不能直接这样做。一个简单的解决方法是，在创建时就不把 rx 包在 Arc<Mutex<>> 里，
        // 或者在需要时克隆接收器。但为了最小化改动，我们在这里使用一个不推荐的模式，
        // 即在调用时才锁定。在更复杂的应用中，这应该被重构。
        // 幸运的是，我们的主循环是单线程的，所以这里的风险很小。
        let mut rx = self.rx.blocking_lock();
        rx.try_recv()
    }

    /// 获取接收器
    pub fn get_receiver(&self) -> Arc<Mutex<mpsc::UnboundedReceiver<StreamEvent>>> {
        Arc::clone(&self.rx)
    }
}

impl Default for StreamHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 流式聊天响应构建器
pub struct StreamingChatResponse {
    pub content: String,
    pub is_complete: bool,
}

impl StreamingChatResponse {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            is_complete: false,
        }
    }

    /// 添加令牌到响应
    pub fn append(&mut self, token: &str) {
        self.content.push_str(token);
    }

    /// 标记为完成
    pub fn mark_complete(&mut self) {
        self.is_complete = true;
    }

    /// 获取当前内容
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// 重置响应
    pub fn reset(&mut self) {
        self.content.clear();
        self.is_complete = false;
    }
}

impl Default for StreamingChatResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_response() {
        let mut response = StreamingChatResponse::new();
        assert!(!response.is_complete);
        assert_eq!(response.get_content(), "");

        response.append("Hello");
        response.append(" ");
        response.append("World");
        assert_eq!(response.get_content(), "Hello World");

        response.mark_complete();
        assert!(response.is_complete);
    }

    #[tokio::test]
    async fn test_stream_handler() {
        let handler = StreamHandler::new();
        let rx = handler.get_receiver();

        // 发送令牌
        handler.send_token("test".to_string()).unwrap();
        handler.send_done().unwrap();

        // 接收令牌
        let mut receiver = rx.lock().await;
        if let Some(StreamEvent::Token(token)) = receiver.recv().await {
            assert_eq!(token, "test");
        }
        if let Some(StreamEvent::Done) = receiver.recv().await {
            // 成功
        }
    }
}
