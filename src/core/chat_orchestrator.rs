/// 对话编排器 - 统一管理所有对话流程
/// 
/// 职责：
/// 1. 统一的对话入口
/// 2. 意图识别和上下文构建
/// 3. LLM 调用和响应处理
/// 5. 错误恢复和流程控制

use std::sync::Arc;
use crate::ai::client::LLMClient;
use crate::ai::code_modification::{AICodeModificationDetector, CodeModificationOp};
use crate::core::{
    ConversationEngine, ConversationContext, UserIntent,
    RetryHandler, RetryConfig, ErrorRecovery, StreamingOptimizer,
    TokenCalculator, ContextWindowOptimizer, MessageHistory, HookManager,
};
use crate::core::tool_executor::ToolExecutor;
use crate::core::conversation_engine::ProcessedResponse;
use std::collections::HashMap;

/// 对话响应
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub modifications: Vec<CodeModificationOp>,
    pub metadata: HashMap<String, String>,
}

/// 对话编排器
pub struct ChatOrchestrator {
    // 核心组件
    conversation_engine: ConversationEngine,
    llm_client: Arc<LLMClient>,
    message_history: MessageHistory,
    
    // 辅助组件
    retry_handler: RetryHandler,
    error_recovery: ErrorRecovery,
    streaming_optimizer: StreamingOptimizer,
    token_calculator: TokenCalculator,
    context_optimizer: ContextWindowOptimizer,
    
    // 工具系统
    tool_executor: ToolExecutor,
    
    // 代码修改检测
    modification_detector: AICodeModificationDetector,
    
    // 钩子系统
    hooks: HookManager,
}

impl ChatOrchestrator {
    /// 创建新的对话编排器
    pub fn new(llm_client: Arc<LLMClient>) -> Self {
        Self {
            conversation_engine: ConversationEngine::new(),
            llm_client,
            message_history: MessageHistory::new(100, 10000),
            retry_handler: RetryHandler::new(RetryConfig::default()),
            error_recovery: ErrorRecovery::new(Default::default()),
            streaming_optimizer: StreamingOptimizer::new(Default::default()),
            token_calculator: TokenCalculator::from_model_name("gpt-4"),
            context_optimizer: ContextWindowOptimizer::new(Default::default()),
            tool_executor: ToolExecutor::new(Arc::new(crate::tools::ToolRegistry::new())),
            modification_detector: AICodeModificationDetector,
            hooks: HookManager::new(),
        }
    }
    
    /// 统一的对话入口 - 处理用户输入并返回响应
    pub async fn process_user_input(&mut self, input: &str) -> Result<ChatResponse, String> {
        // 1. 意图识别
        let intent = self.identify_intent(input)?;
        
        // 2. 构建上下文
        let context = self.build_context(intent)?;
        
        // 3. 前置钩子
        self.hooks.run_pre_hooks(&context).await
            .map_err(|e| format!("前置钩子失败: {}", e))?;
        
        // 4. 调用 LLM（带重试）
        let response = self.call_llm_with_retry(&context).await?;
        
        // 5. 验证响应
        self.validate_response(&response)?;
        
        // 6. 处理工具调用（如果有）
        let final_response = self.handle_tool_calls(&response).await?;
        
        // 7. 检测代码修改
        let modifications = self.detect_modifications(&final_response)?;
        
        // 8. 后置钩子
        let processed_response = ProcessedResponse {
            content: final_response.clone(),
            modifications: vec![],
            suggestions: vec![],
            key_points: vec![],
            thinking: None,
        };
        self.hooks.run_post_hooks(&processed_response).await
            .map_err(|e| format!("后置钩子失败: {}", e))?;
        
        // 9. 保存到历史
        let _ = self.message_history.add_assistant_message(final_response.clone());
        
        Ok(ChatResponse {
            content: final_response.clone(),
            modifications,
            metadata: HashMap::new(),
        })
    }
    
    /// 流式对话入口 - 处理用户输入并通过回调返回流式响应
    pub async fn process_user_input_streaming<F>(&mut self, input: &str, callback: F) -> Result<ChatResponse, String>
    where
        F: FnMut(String) -> bool + Send + 'static,
    {
        // 1. 意图识别
        let intent = self.identify_intent(input)?;

        // 2. 构建上下文
        let context = self.build_context(intent)?;

        // 3. 前置钩子
        self.hooks.run_pre_hooks(&context).await
            .map_err(|e| format!("前置钩子失败: {}", e))?;

        // 4. 调用 LLM 流式（带重试）
        let response = self.call_llm_streaming_with_retry(&context, callback).await?;
        
        // 5. 验证响应
        self.validate_response(&response)?;
        
        // 6. 处理工具调用（如果有）
        let final_response = self.handle_tool_calls(&response).await?;
        
        // 7. 检测代码修改
        let modifications = self.detect_modifications(&final_response)?;
        
        // 8. 后置钩子
        let processed_response = ProcessedResponse {
            content: final_response.clone(),
            modifications: vec![],
            suggestions: vec![],
            key_points: vec![],
            thinking: None,
        };
        self.hooks.run_post_hooks(&processed_response).await
            .map_err(|e| format!("后置钩子失败: {}", e))?;
        
        // 9. 保存到历史
        let _ = self.message_history.add_assistant_message(final_response.clone());
        
        Ok(ChatResponse {
            content: final_response.clone(),
            modifications,
            metadata: HashMap::new(),
        })
    }
    
    /// 意图识别 - 分析用户输入的真实意图
    fn identify_intent(&self, input: &str) -> Result<UserIntent, String> {
        if input.starts_with("@") {
            // 文件提及
            let parts: Vec<&str> = input.split_whitespace().collect();
            let mut paths = Vec::new();
            let mut query = String::new();
            
            for part in parts {
                if part.starts_with("@") {
                    paths.push(part[1..].to_string());
                } else {
                    query.push_str(part);
                    query.push(' ');
                }
            }
            
            Ok(UserIntent::FileMention {
                paths,
                query: query.trim().to_string(),
            })
        } else if input.contains("review") || input.contains("审查") {
            // 代码审查
            Ok(UserIntent::CodeReview {
                files: Vec::new(),
                focus: input.to_string(),
            })
        } else if input.contains("debug") || input.contains("调试") {
            // 调试问题
            Ok(UserIntent::Debug {
                issue: input.to_string(),
                files: Vec::new(),
            })
        } else if input.contains("generate") || input.contains("生成") {
            // 代码生成
            Ok(UserIntent::CodeGeneration {
                description: input.to_string(),
                language: None,
            })
        } else {
            // 普通聊天
            Ok(UserIntent::Chat {
                query: input.to_string(),
                context_files: Vec::new(),
            })
        }
    }
    
    /// 构建对话上下文
    fn build_context(&self, intent: UserIntent) -> Result<ConversationContext, String> {
        let user_input = match &intent {
            UserIntent::Chat { query, .. } => query.clone(),
            UserIntent::FileMention { query, .. } => query.clone(),
            UserIntent::CodeReview { focus, .. } => focus.clone(),
            UserIntent::Debug { issue, .. } => issue.clone(),
            UserIntent::CodeGeneration { description, .. } => description.clone(),
            UserIntent::Command { name, .. } => name.clone(),
        };
        
        Ok(ConversationContext::new(user_input, intent))
    }
    
    /// 调用 LLM 流式（带重试）
    async fn call_llm_streaming_with_retry<F>(&self, context: &ConversationContext, callback: F) -> Result<String, String>
    where
        F: FnMut(String) -> bool + Send + 'static,
    {
        let user_input = match &context.intent {
            UserIntent::Chat { query, .. } => query.clone(),
            UserIntent::FileMention { query, .. } => query.clone(),
            UserIntent::CodeReview { focus, .. } => focus.clone(),
            UserIntent::Debug { issue, .. } => issue.clone(),
            UserIntent::CodeGeneration { description, .. } => description.clone(),
            UserIntent::Command { name, .. } => name.clone(),
        };

        let messages = vec![
            crate::ai::client::ChatMessage {
                role: "user".to_string(),
                content: user_input,
            }
        ];

        // 将回调包装在 Arc<Mutex> 中，使其可以在多次重试中共享
        let callback_arc = std::sync::Arc::new(std::sync::Mutex::new(callback));
        let mut last_error = String::new();

        for attempt in 0..3 {
            let response = Arc::new(std::sync::Mutex::new(String::new()));
            let response_for_callback = Arc::clone(&response);
            let callback_for_attempt = Arc::clone(&callback_arc);

            let streaming_callback = move |token: String| -> bool {
                // 存储响应
                if let Ok(mut r) = response_for_callback.lock() {
                    r.push_str(&token);
                }
                // 调用用户提供的回调
                if let Ok(mut cb) = callback_for_attempt.lock() {
                    cb(token)
                } else {
                    false
                }
            };

            match self.llm_client.generate_completion_stream(messages.clone(), None, streaming_callback).await {
                Ok(_) => {
                    if let Ok(r) = response.lock() {
                        return Ok(r.clone());
                    }
                    return Err("无法获取响应".to_string());
                }
                Err(e) => {
                    last_error = e.to_string();
                    if attempt < 2 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }
        Err(format!("LLM 调用失败（3 次重试后）: {}", last_error))
    }
    
    /// 调用 LLM（带重试）
    async fn call_llm_with_retry(&self, context: &ConversationContext) -> Result<String, String> {
        let user_input = match &context.intent {
            UserIntent::Chat { query, .. } => query.clone(),
            UserIntent::FileMention { query, .. } => query.clone(),
            UserIntent::CodeReview { focus, .. } => focus.clone(),
            UserIntent::Debug { issue, .. } => issue.clone(),
            UserIntent::CodeGeneration { description, .. } => description.clone(),
            UserIntent::Command { name, .. } => name.clone(),
        };
        
        let messages = vec![
            crate::ai::client::ChatMessage {
                role: "user".to_string(),
                content: user_input,
            }
        ];
        
        let mut last_error = String::new();
        for attempt in 0..3 {
            let response = Arc::new(std::sync::Mutex::new(String::new()));
            let response_for_callback = Arc::clone(&response);
            
            let callback = move |token: String| -> bool {
                if let Ok(mut r) = response_for_callback.lock() {
                    r.push_str(&token);
                }
                true
            };
            
            match self.llm_client.generate_completion_stream(messages.clone(), None, callback).await {
                Ok(_) => {
                    if let Ok(r) = response.lock() {
                        return Ok(r.clone());
                    }
                    return Err("无法获取响应".to_string());
                }
                Err(e) => {
                    last_error = e.to_string();
                    if attempt < 2 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }
        Err(format!("LLM 调用失败（3 次重试后）: {}", last_error))
    }
    
    /// 验证响应
    fn validate_response(&self, response: &str) -> Result<(), String> {
        if response.is_empty() {
            return Err("LLM 返回空响应".to_string());
        }
        
        if response.len() > 100000 {
            return Err("响应过长（超过 100KB）".to_string());
        }
        
        Ok(())
    }
    
    /// 处理工具调用
    async fn handle_tool_calls(&self, response: &str) -> Result<String, String> {
        // 检查响应中是否包含工具调用标记
        if response.contains("<|start_header|>") || response.contains("```tool") {
            // 提取工具调用信息
            let final_response = response.to_string();
            
            // 简单的工具调用处理：
            // 1. 检测工具调用标记
            // 2. 记录工具调用
            // 3. 返回响应（实际工具执行由应用层处理）
            
            // 这里可以添加更复杂的工具调用处理逻辑
            // 例如：解析工具参数、执行工具、获取结果、递归调用 LLM
            
            Ok(final_response)
        } else {
            Ok(response.to_string())
        }
    }
    
    /// 检测代码修改
    fn detect_modifications(&self, response: &str) -> Result<Vec<CodeModificationOp>, String> {
        let modifications = AICodeModificationDetector::detect_modifications(response);
        Ok(modifications)
    }
    
    /// 获取消息历史
    pub fn get_message_history(&self) -> &MessageHistory {
        &self.message_history
    }
    
    /// 获取 Token 统计
    pub fn get_token_stats(&self) -> String {
        format!(
            "消息数: {}, 总 Token: ~{}",
            self.message_history.get_messages().len(),
            self.message_history.get_messages().len() * 50 // 粗略估计
        )
    }
    
    /// 获取流式处理性能指标
    pub fn get_streaming_metrics(&self) -> String {
        let metrics = self.streaming_optimizer.get_metrics();
        format!(
            "事件数: {}, 总字节: {}, 平均延迟: {:.2}ms, 吞吐量: {:.0} events/s",
            metrics.total_events,
            metrics.total_bytes,
            metrics.average_latency_ms,
            metrics.throughput_events_per_sec
        )
    }
    
    /// 优化消息历史上下文
    pub fn optimize_context(&mut self) {
        // 使用 ContextOptimizer 优化长对话
        let messages = self.message_history.get_messages();
        if messages.len() > 10 {
            // 如果消息过多，可以应用上下文优化
            // 这里可以实现滑动窗口或智能摘要
        }
    }
    
    /// 清空历史
    pub fn clear_history(&mut self) {
        self.message_history.clear();
    }
    
    /// 获取对话引擎的引用
    pub fn get_conversation_engine(&self) -> &ConversationEngine {
        &self.conversation_engine
    }
    
    /// 获取对话引擎的可变引用
    pub fn get_conversation_engine_mut(&mut self) -> &mut ConversationEngine {
        &mut self.conversation_engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_identification() {
        let orchestrator = ChatOrchestrator::new(Arc::new(LLMClient::new(Default::default())));
        
        // 测试文件提及
        let intent = orchestrator.identify_intent("@src/main.rs 这个文件有什么问题？");
        assert!(matches!(intent, Ok(UserIntent::FileMention { .. })));
        
        // 测试代码审查
        let intent = orchestrator.identify_intent("请 review 这段代码");
        assert!(matches!(intent, Ok(UserIntent::CodeReview { .. })));
        
        // 测试调试
        let intent = orchestrator.identify_intent("帮我 debug 这个问题");
        assert!(matches!(intent, Ok(UserIntent::Debug { .. })));
        
        // 测试普通聊天
        let intent = orchestrator.identify_intent("你好");
        assert!(matches!(intent, Ok(UserIntent::Chat { .. })));
    }
    
    #[test]
    fn test_response_validation() {
        let orchestrator = ChatOrchestrator::new(Arc::new(LLMClient::new(Default::default())));
        
        // 测试空响应
        assert!(orchestrator.validate_response("").is_err());
        
        // 测试正常响应
        assert!(orchestrator.validate_response("这是一个正常的响应").is_ok());
    }
}
