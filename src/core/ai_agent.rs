/// AI Agent 核心实现
/// 实现类似 grok-cli 的 GrokAgent，支持 LLM 对话和工具调用

use crate::ai::client::{LLMClient, ChatMessage};
use crate::tools::{ToolRegistry, ToolCall, ToolDefinition};
use crate::core::message::{Message, Role};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// AI Agent 配置
#[derive(Clone, Debug)]
pub struct AIAgentConfig {
    pub max_tool_rounds: usize,
    pub model: String,
    pub enable_search: bool,
}

impl Default for AIAgentConfig {
    fn default() -> Self {
        Self {
            max_tool_rounds: 50, // 默认最多 50 轮工具调用
            model: "grok-code-fast-1".to_string(),
            enable_search: false,
        }
    }
}

#[derive(Clone)]
pub struct AIAgent {
    llm_client: Arc<LLMClient>,
    tool_registry: Arc<Mutex<ToolRegistry>>,
    config: AIAgentConfig,
    todo_manager: Arc<tokio::sync::Mutex<crate::tools::todo_tool::TodoManager>>,
}

impl AIAgent {
    /// 创建新的 AI Agent
    pub fn new(
        llm_client: Arc<LLMClient>,
        config: AIAgentConfig,
    ) -> Self {
        let tool_registry = Arc::new(Mutex::new(ToolRegistry::new()));
        let todo_manager = Arc::new(tokio::sync::Mutex::new(crate::tools::todo_tool::TodoManager::new()));

        Self {
            llm_client,
            tool_registry,
            config,
            todo_manager,
        }
    }

    /// 获取工具注册表（用于注册工具）
    pub fn tool_registry(&self) -> Arc<Mutex<ToolRegistry>> {
        self.tool_registry.clone()
    }

    /// 注册所有标准工具
    pub async fn register_standard_tools(&self) {
        use crate::tools::*;
        let mut registry = self.tool_registry.lock().await;

        // 文件工具
        registry.register(Arc::new(FileReadTool));
        registry.register(Arc::new(FileWriteTool));
        registry.register(Arc::new(FileListTool));
        registry.register(Arc::new(StrReplaceTool));

        // 终端工具
        registry.register(Arc::new(CommandExecuteTool));
        registry.register(Arc::new(EnvironmentInfoTool));

        // 代码工具
        registry.register(Arc::new(CodeSearchTool));
        registry.register(Arc::new(FunctionFinderTool));
        registry.register(Arc::new(CodeStructureTool));

        // 项目工具
        registry.register(Arc::new(ProjectStructureTool));
        registry.register(Arc::new(DependencyAnalyzerTool));
        registry.register(Arc::new(BuildTool));

        // Todo 工具（需要共享 manager）
        registry.register(Arc::new(CreateTodoListTool::new(self.todo_manager.clone())));
        registry.register(Arc::new(UpdateTodoListTool::new(self.todo_manager.clone())));
    }

    /// 处理用户消息（完整对话流程）
    pub async fn process_message(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<AgentResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_messages = messages.clone();
        let mut tool_calls_history = Vec::new();
        let mut total_rounds = 0;

        loop {
            if total_rounds >= self.config.max_tool_rounds {
                return Ok(AgentResponse {
                    messages: all_messages,
                    tool_calls: tool_calls_history,
                    status: AgentStatus::MaxRoundsReached,
                });
            }

            // 获取工具定义
            let registry = self.tool_registry.lock().await;
            let tool_definitions = registry.list_definitions();
            drop(registry);

            // 调用 LLM
            let response = self.llm_client.generate_completion(
                all_messages.clone(),
                Some(self.config.model.clone()),
                if !tool_definitions.is_empty() {
                    Some(tool_definitions)
                } else {
                    None
                },
            ).await?;

            // 解析响应（简化版本）
            match self.parse_llm_response(&response).await? {
                LLMResponse::Content(content) => {
                    all_messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content,
                    });

                    return Ok(AgentResponse {
                        messages: all_messages,
                        tool_calls: tool_calls_history,
                        status: AgentStatus::Completed,
                    });
                }
                LLMResponse::ToolCalls(tool_calls) => {
                    total_rounds += 1;

                    // 执行工具调用
                    let registry = self.tool_registry.lock().await;
                    for tool_call in tool_calls {
                        let tool_name = tool_call.tool_name.clone();
                        let result = registry.execute(tool_call).await;

                        tool_calls_history.push(ToolCallResult {
                            tool_name,
                            result: result.clone(),
                        });

                        // 将工具结果添加到消息中
                        all_messages.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: format!("Calling tool: {:?}", result),
                        });
                    }
                    drop(registry);

                    // 继续循环，让 LLM 处理工具结果
                }
            }
        }
    }

    /// 流式处理用户消息
    pub async fn process_message_stream<F>(
        &self,
        messages: Vec<ChatMessage>,
        mut callback: F,
    ) -> Result<AgentResponse, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnMut(String) -> bool + Send + 'static,
    {
        // 简化版本：先实现非流式，后续添加流式支持
        let response = self.process_message(messages).await?;

        // 回调最终响应
        if let Some(last_message) = response.messages.last() {
            callback(last_message.content.clone());
        }

        Ok(response)
    }

    /// 解析 LLM 响应
    async fn parse_llm_response(
        &self,
        response: &str,
    ) -> Result<LLMResponse, Box<dyn std::error::Error + Send + Sync>> {
        // 简化实现：实际应该从 LLM 响应中解析工具调用
        // 这里模拟返回内容（实际应与 LLM API 格式匹配）
        Ok(LLMResponse::Content(response.to_string()))
    }
}

/// LLM 响应类型
enum LLMResponse {
    Content(String),
    ToolCalls(Vec<ToolCall>),
}

/// Agent 响应
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub messages: Vec<ChatMessage>,
    pub tool_calls: Vec<ToolCallResult>,
    pub status: AgentStatus,
}

/// 工具调用结果
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub result: crate::tools::ToolResult,
}

/// Agent 状态
#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Completed,
    MaxRoundsReached,
    Error(String),
}

/// 将内部 Message 转换为 ChatMessage
pub fn convert_to_chat_messages(messages: &[Message]) -> Vec<ChatMessage> {
    messages
        .iter()
        .map(|msg| ChatMessage {
            role: match msg.role {
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
                Role::System => "system".to_string(),
            },
            content: msg.content.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_messages() {
        use crate::core::message::Message;

        let messages = vec![
            Message {
                role: Role::User,
                content: "Hello".to_string(),
            },
            Message {
                role: Role::Assistant,
                content: "Hi there".to_string(),
            },
        ];

        let chat_messages = convert_to_chat_messages(&messages);
        assert_eq!(chat_messages.len(), 2);
        assert_eq!(chat_messages[0].role, "user");
        assert_eq!(chat_messages[1].role, "assistant");
    }

    #[tokio::test]
    async fn test_ai_agent_creation() {
        use crate::ai::config::LLMConfig;

        let config = LLMConfig::from_env().unwrap_or_else(|_| LLMConfig {
            api_key: "test".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
        });

        let llm_client = Arc::new(LLMClient::new(config));
        let agent_config = AIAgentConfig::default();
        let agent = AIAgent::new(llm_client, agent_config);

        // 注册工具
        agent.register_standard_tools().await;

        // 验证工具已注册
        let registry = agent.tool_registry().lock().await;
        assert!(registry.count() > 0);
    }
}
