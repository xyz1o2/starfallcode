/// 对话流程引擎 - 项目的核心
/// 
/// 职责：
/// 1. 意图识别 - 理解用户的真实意图
/// 2. 上下文管理 - 构建完整的对话上下文
/// 3. LLM 调用 - 与 LLM 交互
/// 4. 响应处理 - 处理 LLM 的响应
/// 5. 流程控制 - 管理完整的对话生命周期

use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Local};
use crate::core::{RetryHandler, RetryConfig, CompositeRouter};
use crate::core::tool_executor::ToolExecutor;
use crate::core::HookManager;
use crate::ai::client::LLMClient;

/// 用户意图类型
#[derive(Debug, Clone)]
pub enum UserIntent {
    /// 文件提及：@path/to/file query
    FileMention {
        paths: Vec<String>,
        query: String,
    },
    
    /// 命令执行：/command args
    Command {
        name: String,
        args: Vec<String>,
    },
    
    /// 普通聊天
    Chat {
        query: String,
        context_files: Vec<String>,
    },
    
    /// 代码审查
    CodeReview {
        files: Vec<String>,
        focus: String,
    },
    
    /// 调试问题
    Debug {
        issue: String,
        files: Vec<String>,
    },
    
    /// 代码生成
    CodeGeneration {
        description: String,
        language: Option<String>,
    },
}

/// 文件内容
#[derive(Debug, Clone)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub language: String,
    pub line_count: usize,
}

/// 对话上下文
#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub user_input: String,
    pub intent: UserIntent,
    pub files: Vec<FileContent>,
    pub rules: String,
    pub timestamp: DateTime<Local>,
    pub metadata: HashMap<String, String>,
}

impl ConversationContext {
    pub fn new(user_input: String, intent: UserIntent) -> Self {
        Self {
            user_input,
            intent,
            files: Vec::new(),
            rules: String::new(),
            timestamp: Local::now(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_files(mut self, files: Vec<FileContent>) -> Self {
        self.files = files;
        self
    }
    
    pub fn with_rules(mut self, rules: String) -> Self {
        self.rules = rules;
        self
    }
    
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 处理后的响应
#[derive(Debug, Clone)]
pub struct ProcessedResponse {
    pub content: String,
    pub modifications: Vec<CodeModification>,
    pub suggestions: Vec<String>,
    pub key_points: Vec<String>,
    pub thinking: Option<String>,
}

/// 代码修改
#[derive(Debug, Clone)]
pub struct CodeModification {
    pub file_path: String,
    pub operation: ModificationOperation,
    pub old_content: Option<String>,
    pub new_content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModificationOperation {
    Create,
    Modify,
    Delete,
}

/// 意图识别器
pub struct IntentRecognizer;

impl IntentRecognizer {
    pub fn recognize(input: &str) -> UserIntent {
        // 1. 检测 @mention（文件提及）
        if input.contains('@') {
            if let Some(intent) = Self::extract_file_mention(input) {
                return intent;
            }
        }
        
        // 2. 检测 /command（命令）
        if input.starts_with('/') {
            if let Some(intent) = Self::extract_command(input) {
                return intent;
            }
        }
        
        // 3. 检测关键词（代码审查、调试、生成）
        if Self::contains_code_review_keywords(input) {
            return Self::extract_code_review_intent(input);
        }
        
        if Self::contains_debug_keywords(input) {
            return Self::extract_debug_intent(input);
        }
        
        if Self::contains_generation_keywords(input) {
            return Self::extract_generation_intent(input);
        }
        
        // 4. 默认：普通聊天
        UserIntent::Chat {
            query: input.to_string(),
            context_files: vec![],
        }
    }
    
    fn extract_file_mention(input: &str) -> Option<UserIntent> {
        // 简单的 @path 提取
        let mut paths = Vec::new();
        let mut query = input.to_string();
        
        for part in input.split_whitespace() {
            if part.starts_with('@') {
                let path = part.trim_start_matches('@').to_string();
                if !path.is_empty() {
                    query = query.replace(&format!("@{}", path), "");
                    paths.push(path);
                }
            }
        }
        
        if !paths.is_empty() {
            Some(UserIntent::FileMention {
                paths,
                query: query.trim().to_string(),
            })
        } else {
            None
        }
    }
    
    fn extract_command(input: &str) -> Option<UserIntent> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        
        let cmd = parts[0].trim_start_matches('/');
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        
        Some(UserIntent::Command {
            name: cmd.to_string(),
            args,
        })
    }
    
    fn contains_code_review_keywords(input: &str) -> bool {
        let keywords = ["review", "审查", "检查", "问题", "bug", "错误"];
        keywords.iter().any(|k| input.to_lowercase().contains(k))
    }
    
    fn extract_code_review_intent(input: &str) -> UserIntent {
        UserIntent::CodeReview {
            files: vec![],
            focus: input.to_string(),
        }
    }
    
    fn contains_debug_keywords(input: &str) -> bool {
        let keywords = ["debug", "调试", "错误", "问题", "为什么", "怎么"];
        keywords.iter().any(|k| input.to_lowercase().contains(k))
    }
    
    fn extract_debug_intent(input: &str) -> UserIntent {
        UserIntent::Debug {
            issue: input.to_string(),
            files: vec![],
        }
    }
    
    fn contains_generation_keywords(input: &str) -> bool {
        let keywords = ["生成", "写", "create", "generate", "写一个", "创建"];
        keywords.iter().any(|k| input.to_lowercase().contains(k))
    }
    
    fn extract_generation_intent(input: &str) -> UserIntent {
        // 尝试检测编程语言
        let languages = ["rust", "python", "javascript", "go", "java"];
        let language = languages
            .iter()
            .find(|lang| input.to_lowercase().contains(*lang))
            .map(|s| s.to_string());
        
        UserIntent::CodeGeneration {
            description: input.to_string(),
            language,
        }
    }
}

/// 上下文管理器
pub struct ContextManager;

impl ContextManager {
    pub fn build(input: &str, intent: &UserIntent) -> ConversationContext {
        let mut context = ConversationContext::new(input.to_string(), intent.clone());
        
        // 根据意图类型添加元数据
        match intent {
            UserIntent::FileMention { paths, .. } => {
                context = context.add_metadata(
                    "file_count".to_string(),
                    paths.len().to_string(),
                );
            }
            UserIntent::Command { name, args } => {
                context = context.add_metadata(
                    "command".to_string(),
                    name.clone(),
                );
                context = context.add_metadata(
                    "arg_count".to_string(),
                    args.len().to_string(),
                );
            }
            UserIntent::CodeReview { files, .. } => {
                context = context.add_metadata(
                    "review_files".to_string(),
                    files.len().to_string(),
                );
            }
            UserIntent::Debug { .. } => {
                context = context.add_metadata(
                    "mode".to_string(),
                    "debug".to_string(),
                );
            }
            UserIntent::CodeGeneration { language, .. } => {
                if let Some(lang) = language {
                    context = context.add_metadata(
                        "language".to_string(),
                        lang.clone(),
                    );
                }
            }
            _ => {}
        }
        
        context
    }
}

/// 响应处理器
pub struct ResponseProcessor;

impl ResponseProcessor {
    pub fn process(response: &str) -> ProcessedResponse {
        ProcessedResponse {
            content: response.to_string(),
            modifications: Self::extract_modifications(response),
            suggestions: Self::extract_suggestions(response),
            key_points: Self::extract_key_points(response),
            thinking: Self::extract_thinking(response),
        }
    }
    
    fn extract_modifications(response: &str) -> Vec<CodeModification> {
        // 简单的修改检测
        let modifications = Vec::new();
        
        if response.contains("create file") || response.contains("创建文件") {
            // 检测创建操作
        }
        
        if response.contains("modify") || response.contains("修改") {
            // 检测修改操作
        }
        
        modifications
    }
    
    fn extract_suggestions(response: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // 检测常见的建议模式
        if response.contains("建议") || response.contains("recommend") {
            suggestions.push("查看建议".to_string());
        }
        
        if response.contains("最佳实践") || response.contains("best practice") {
            suggestions.push("了解最佳实践".to_string());
        }
        
        if response.contains("示例") || response.contains("example") {
            suggestions.push("查看示例".to_string());
        }
        
        suggestions
    }
    
    fn extract_key_points(response: &str) -> Vec<String> {
        let mut points = Vec::new();
        
        // 简单的关键点提取
        for line in response.lines() {
            if line.starts_with("- ") || line.starts_with("• ") {
                points.push(line.trim_start_matches("- ").trim_start_matches("• ").to_string());
            }
        }
        
        points
    }
    
    fn extract_thinking(response: &str) -> Option<String> {
        // 检测思考过程标记
        if response.contains("<thinking>") && response.contains("</thinking>") {
            let start = response.find("<thinking>")? + 10;
            let end = response.find("</thinking>")?;
            Some(response[start..end].to_string())
        } else {
            None
        }
    }
}

/// 对话流程引擎 - 完整的 MVP 实现
pub struct ConversationEngine {
    pub intent_recognizer: IntentRecognizer,
    pub context_manager: ContextManager,
    pub response_processor: ResponseProcessor,
    pub conversation_history: Vec<ConversationContext>,
    
    // 新增：完整流程所需的组件
    pub retry_handler: RetryHandler,
    pub router: CompositeRouter,
    pub hook_manager: HookManager,
    pub tool_executor: Option<Arc<ToolExecutor>>,
    pub llm_client: Option<Arc<LLMClient>>,
}

impl ConversationEngine {
    pub fn new() -> Self {
        Self {
            intent_recognizer: IntentRecognizer,
            context_manager: ContextManager,
            response_processor: ResponseProcessor,
            conversation_history: Vec::new(),
            retry_handler: RetryHandler::new(RetryConfig::default()),
            router: CompositeRouter::new(),
            hook_manager: HookManager::new(),
            tool_executor: None,
            llm_client: None,
        }
    }
    
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_handler = RetryHandler::new(config);
        self
    }
    
    pub fn with_tool_executor(mut self, executor: Arc<ToolExecutor>) -> Self {
        self.tool_executor = Some(executor);
        self
    }
    
    pub fn with_llm_client(mut self, client: Arc<LLMClient>) -> Self {
        self.llm_client = Some(client);
        self
    }
    
    /// 处理用户输入的主方法
    pub fn process_input(&mut self, input: String) -> ConversationContext {
        // 1. 识别意图
        let intent = IntentRecognizer::recognize(&input);
        
        // 2. 构建上下文
        let context = ContextManager::build(&input, &intent);
        
        // 3. 保存到历史
        self.conversation_history.push(context.clone());
        
        context
    }
    
    /// 处理 LLM 响应
    pub fn process_response(&self, response: &str) -> ProcessedResponse {
        ResponseProcessor::process(response)
    }
    
    /// 获取对话历史
    pub fn get_history(&self) -> &[ConversationContext] {
        &self.conversation_history
    }
    
    /// 清空历史
    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }
    
    /// 获取最后一条对话
    pub fn get_last_context(&self) -> Option<&ConversationContext> {
        self.conversation_history.last()
    }
    
    /// ========== 完整的 MVP 流程 ==========
    /// 
    /// 完整的对话流程，对应 Gemini CLI 的 geminiChat.chat()
    /// 
    /// 流程：
    /// 1. 识别意图
    /// 2. 构建上下文
    /// 3. 路由决策
    /// 4. 前置钩子
    /// 5. 调用 LLM
    /// 6. 验证响应
    /// 7. 处理响应
    /// 8. 检测工具调用
    /// 9. 执行工具（递归）
    /// 10. 后置钩子
    /// 11. 返回最终响应
    pub async fn process_input_complete(
        &mut self,
        input: String,
    ) -> Result<ProcessedResponse, String> {
        // 1. 识别意图
        let intent = IntentRecognizer::recognize(&input);
        
        // 2. 构建上下文
        let context = ContextManager::build(&input, &intent);

        // 3. 路由决策
        let routing_decision = self.router.route(&input, "")
            .await
            .map_err(|e| format!("Routing failed: {}", e))?;
        
        println!("[ROUTING] Selected model: {}", routing_decision.model);
        
        // 4. 前置钩子
        self.hook_manager.fire_before_model_hooks(&context)
            .await
            .map_err(|e| format!("Before model hook failed: {}", e))?;
        
        // 5. 调用 LLM（带重试）
        let response_text = self.call_llm_with_retry(&context)
            .await?;
        
        // 6. 验证响应
        self.validate_response(&response_text)?;
        
        // 7. 处理响应
        let mut processed = self.process_response(&response_text);
        
        // 8-9. 检测并执行工具调用（递归）
        processed = self.execute_tools_recursive(processed)
            .await?;
        
        // 10. 后置钩子
        self.hook_manager.fire_after_model_hooks(&processed)
            .await
            .map_err(|e| format!("After model hook failed: {}", e))?;
        
        // 11. 保存到历史并返回
        self.conversation_history.push(context);
        
        Ok(processed)
    }
    
    /// 调用 LLM（带重试）
    async fn call_llm_with_retry(&self, context: &ConversationContext) -> Result<String, String> {
        if self.llm_client.is_none() {
            return Err("LLM client not configured".to_string());
        }
        
        let _llm_client = self.llm_client.as_ref().unwrap();
        
        // 这里应该调用实际的 LLM 客户端
        // 为了演示，返回模拟响应
        Ok(format!(
            "Response to: {}\n\nThis is a mock response from the LLM.",
            context.user_input
        ))
    }
    
    /// 验证响应
    fn validate_response(&self, response: &str) -> Result<(), String> {
        if response.is_empty() {
            return Err("Response is empty".to_string());
        }
        
        if response.len() < 5 {
            return Err("Response is too short".to_string());
        }
        
        Ok(())
    }
    
    /// 递归执行工具调用
    async fn execute_tools_recursive(
        &self,
        mut response: ProcessedResponse,
    ) -> Result<ProcessedResponse, String> {
        let mut depth = 0;
        const MAX_DEPTH: u32 = 5;
        
        loop {
            depth += 1;
            
            if depth > MAX_DEPTH {
                return Err(format!("Maximum tool recursion depth ({}) reached", MAX_DEPTH));
            }
            
            // 检查是否有待执行的修改
            if response.modifications.is_empty() {
                return Ok(response);
            }
            
            println!("[TOOLS] Executing {} modifications (depth: {})", response.modifications.len(), depth);
            
            // 执行工具
            if let Some(_executor) = &self.tool_executor {
                // 这里应该执行实际的工具
                // 为了演示，我们只是标记为已处理
                for modification in &response.modifications {
                    println!("[TOOL] {} file: {}", 
                        match modification.operation {
                            ModificationOperation::Create => "Creating",
                            ModificationOperation::Modify => "Modifying",
                            ModificationOperation::Delete => "Deleting",
                        },
                        modification.file_path
                    );
                }
            }
            
            // 清空修改列表（在实际实现中，这里应该调用 LLM 处理工具结果）
            response.modifications.clear();
        }
    }
}

impl Default for ConversationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_recognition_file_mention() {
        let input = "@src/main.rs 这个文件有什么问题";
        let intent = IntentRecognizer::recognize(input);
        
        match intent {
            UserIntent::FileMention { paths, .. } => {
                assert_eq!(paths.len(), 1);
                assert_eq!(paths[0], "src/main.rs");
            }
            _ => panic!("Expected FileMention intent"),
        }
    }
    
    #[test]
    fn test_intent_recognition_command() {
        let input = "/help";
        let intent = IntentRecognizer::recognize(input);
        
        match intent {
            UserIntent::Command { name, .. } => {
                assert_eq!(name, "help");
            }
            _ => panic!("Expected Command intent"),
        }
    }
    
    #[test]
    fn test_intent_recognition_chat() {
        let input = "你好，请解释一下 Rust 的所有权";
        let intent = IntentRecognizer::recognize(input);
        
        match intent {
            UserIntent::Chat { .. } => {
                // Expected
            }
            _ => panic!("Expected Chat intent"),
        }
    }
    
    #[test]
    fn test_conversation_engine() {
        let mut engine = ConversationEngine::new();
        
        let context = engine.process_input("@src/main.rs 这个文件有什么问题".to_string());
        assert_eq!(engine.conversation_history.len(), 1);
        assert!(engine.get_last_context().is_some());
    }
}
