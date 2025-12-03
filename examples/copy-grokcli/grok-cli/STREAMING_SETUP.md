# Grok CLI - 流式输出实现指南

## 概述

Grok CLI 现已支持 LLM 的流式响应输出，使用 Server-Sent Events (SSE) 格式的流式接口。

## 流式输出的核心变化

### 1. **Client 层 (`src/grok/client.rs`)**

新增 `chat_stream()` 方法，用于获取流式响应：

```rust
pub async fn chat_stream(
    &self,
    messages: Vec<GrokMessage>,
    tools: Option<Vec<GrokTool>>,
    model: Option<String>,
    search_options: Option<SearchOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value, Box<dyn std::error::Error + Send>>> + Send>>, Box<dyn std::error::Error + Send>>
```

**实现细节：**
- 向 API 请求体中添加 `stream: true` 参数
- 使用 `response.text().await` 读取完整响应体
- 按行解析 SSE 格式数据 (`data: {...}` 格式)
- 遇到 `[DONE]` 标记时停止流

### 2. **Agent 层 (`src/agent/mod.rs`)**

新增 `process_user_message_stream()` 方法，用于处理流式响应：

```rust
pub async fn process_user_message_stream(
    &mut self,
    message: &str,
) -> Result<Pin<Box<dyn Stream<Item = Result<StreamingChunk, Box<dyn std::error::Error + Send>>> + Send>>, Box<dyn std::error::Error + Send>>
```

**功能：**
- 接收用户消息
- 调用 `chat_stream()` 获取流式响应
- 实时解析流式数据并生成 `StreamingChunk`
- 支持三种流式块类型：
  - `Content`: 文本内容块
  - `ToolCalls`: 工具调用块
  - `Done`: 流完成块
  - `TokenCount`: Token 计数块

## 使用示例

### 在主应用中启用流式输出

```rust
// 创建 agent
let mut agent = GrokAgent::new(&api_key, base_url, model, Some(max_tool_rounds), Some(true)).await?;

// 使用流式处理
let mut stream = agent.process_user_message_stream("你的问题").await?;

// 使用 futures::stream::StreamExt
use futures::stream::StreamExt;

while let Some(result) = stream.next().await {
    match result {
        Ok(chunk) => {
            match chunk.chunk_type {
                StreamingChunkType::Content => {
                    if let Some(content) = chunk.content {
                        print!("{}", content);
                        io::stdout().flush().ok();
                    }
                }
                StreamingChunkType::ToolCalls => {
                    // 处理工具调用
                    if let Some(tools) = chunk.tool_calls {
                        for tool in tools {
                            println!("🔧 Calling: {}", tool.function.name);
                        }
                    }
                }
                StreamingChunkType::Done => {
                    println!("\n✅ Stream finished");
                }
                StreamingChunkType::TokenCount => {
                    if let Some(count) = chunk.token_count {
                        println!("📊 Tokens used: {}", count);
                    }
                }
                _ => {}
            }
        }
        Err(e) => {
            eprintln!("❌ Stream error: {}", e);
            break;
        }
    }
}
```

## SSE 格式解析

API 返回的流式响应遵循 Server-Sent Events 格式：

```
data: {"choices":[{"delta":{"content":"Hello"},"finish_reason":null}],"usage":{"prompt_tokens":10,"completion_tokens":1,"total_tokens":11}}
data: {"choices":[{"delta":{"content":" world"},"finish_reason":null}],"usage":{"prompt_tokens":10,"completion_tokens":2,"total_tokens":12}}
data: [DONE]
```

## 优势

✅ **实时反馈**: 用户可以立即看到 AI 的响应
✅ **更好的用户体验**: 不需要等待完整响应
✅ **减少延迟感**: 长时间计算时提供进度反馈
✅ **支持工具调用**: 流式显示工具调用和执行结果

## 技术栈

- **`reqwest`**: 异步 HTTP 客户端
- **`futures`**: 异步流处理
- **`async-stream`**: 生成器宏用于创建异步流
- **`serde_json`**: JSON 解析

## 配置

在 `Cargo.toml` 中已包含的依赖：

```toml
reqwest = { version = "0.12", features = ["json"] }
futures = "0.3"
async-stream = "0.3"
```

## 故障排除

### 流式响应为空
- 检查 API key 是否正确设置
- 确保使用的 Grok 模型支持流式输出

### 解析错误
- 验证 SSE 格式是否正确
- 检查网络连接是否稳定

### 超时
- 增加 HTTP 客户端超时时间
- 检查 API 服务器状态
