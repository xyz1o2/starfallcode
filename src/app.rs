use crate::ai::client::{LLMClient, ChatMessage};
use crate::ai::commands::{CommandParser, CommandType};
use crate::ai::config::LLMConfig;
use crate::ai::streaming::{StreamHandler, StreamingChatResponse};
use crate::core::message::{Message, Role};
use crate::core::history::ChatHistory;
use crate::core::{GeminiArchitecture, ConversationEngine, ChatOrchestrator};
use crate::ui::command_hints::CommandHints;
use crate::commands::file_commands::FileCommandHandler;
use crate::prompts;
use crate::ai::code_modification::{AICodeModificationDetector, CodeModificationOp, CodeDiff, CodeMatcher};
use crate::core::vibe_coding::{VibeWorkflowManager, VibeStage};
use crate::commands::VibeCommandHandler;
use crate::ui::filename_suggestion::FilenameSuggestion;
use ratatui::{Frame, widgets::ScrollbarState};
use std::sync::{Arc, Mutex};
use crate::ui;

// ============ Action 系统 ============

/// Action - 事件驱动的应用状态管理
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // 基础操作
    Quit,
    Tick,
    Render,
    
    // 输入操作
    CharInput(char),
    Backspace,
    Delete,
    EnterKey,
    EscapeKey,
    
    // 导航操作
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    
    // 聊天操作
    SendMessage,
    ClearHistory,
    
    // 快捷键
    Help,
    Status,
    
    // 其他
    None,
}

/// Action 队列
pub struct ActionQueue {
    queue: Vec<Action>,
}

impl ActionQueue {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
        }
    }

    pub fn push(&mut self, action: Action) {
        if action != Action::None {
            self.queue.push(action);
        }
    }

    pub fn pop(&mut self) -> Option<Action> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Default for ActionQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// 格式化 Diff 对比
fn format_diff(old: &str, new: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let max_lines = old_lines.len().max(new_lines.len());
    
    let mut result = String::new();
    for i in 0..max_lines {
        if i < old_lines.len() {
            result.push_str(&format!("- {}\n", old_lines[i]));
        }
        if i < new_lines.len() {
            result.push_str(&format!("+ {}\n", new_lines[i]));
        }
    }
    result
}

#[derive(Debug, PartialEq)]
pub enum AppAction {
    None,
    SubmitChat,
    Quit,
}

/// 代码修改确认选择
#[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ModificationChoice {
        Confirm,  // 1. 确认
        Cancel,   // 2. 取消
        Abandon,  // 3. 放弃
    }

pub struct App {
    pub should_quit: bool,
    pub chat_history: ChatHistory,
    pub input_text: String,
    pub input_cursor: usize, // Track cursor position in input text
    pub llm_config: Option<LLMConfig>,
    pub llm_client: Option<Arc<LLMClient>>,
    pub is_streaming: bool,
    pub stream_handler: Option<StreamHandler>,
    pub streaming_response: Arc<Mutex<StreamingChatResponse>>,
    pub command_hints: CommandHints,
    pub file_command_handler: FileCommandHandler,

    // AI 代码修改确认相关
    pub pending_modifications: Vec<(CodeModificationOp, Option<CodeDiff>)>,
    pub modification_confirmation_pending: bool,
    pub modification_selected_index: usize,
    pub modification_choice: ModificationChoice,

    // 聊天历史滚动
    pub chat_scroll_offset: usize,
    pub scrollbar_state: ScrollbarState,

    // Action 系统
    pub action_queue: ActionQueue,

    // 输入框滚动
    pub input_scroll_offset: usize,

    // 鼠标选择
    pub selected_text: String,
    pub selection_end: Option<(u16, u16)>,

    // @ 提及建议
    pub mention_suggestions: crate::ui::mention_suggestions::MentionSuggestions,

    // 文件搜索引擎
    pub file_search: crate::ui::file_search::FileSearchEngine,

    // 高效渲染引擎
    pub render_engine: crate::ui::render_engine::RenderEngine,

    // 动画帧计数
    pub frame_count: u32,

    // Gemini 架构
    pub gemini: GeminiArchitecture,

    // Conversation Engine
    pub conversation_engine: ConversationEngine,

    // ChatOrchestrator - 统一的对话编排器
    pub chat_orchestrator: Option<ChatOrchestrator>,

    // Vibe Coding Workflow Manager (5阶段工作流)
    pub vibe_workflow: VibeWorkflowManager,

    // Vibe Coding 命令处理器
    pub vibe_command_handler: VibeCommandHandler,

    // 文件名建议对话框（当AI检测到代码生成意图但未指定文件名时）
    pub filename_suggestion: FilenameSuggestion,

    // AI Agent - 类似 grok-cli 的 GrokAgent，支持工具调用
    pub ai_agent: Option<crate::core::AIAgent>,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            chat_history: ChatHistory::new(100),
            input_text: String::new(),
            input_cursor: 0,
            llm_config: None,
            llm_client: None,
            is_streaming: false,
            stream_handler: None,
            streaming_response: Arc::new(Mutex::new(StreamingChatResponse::new())),
            command_hints: CommandHints::new(),
            file_command_handler: FileCommandHandler::new(),
            pending_modifications: Vec::new(),
            modification_confirmation_pending: false,
            modification_selected_index: 0,
            modification_choice: ModificationChoice::Confirm,
            chat_scroll_offset: 0,
            scrollbar_state: ScrollbarState::default(),
            action_queue: ActionQueue::new(),
            input_scroll_offset: 0,
            selected_text: String::new(),
            selection_end: None,
            mention_suggestions: crate::ui::mention_suggestions::MentionSuggestions::new(),
            file_search: crate::ui::file_search::FileSearchEngine::new(),
            render_engine: crate::ui::render_engine::RenderEngine::new(),
            frame_count: 0,
            gemini: GeminiArchitecture::new(),
            conversation_engine: ConversationEngine::new(),
            chat_orchestrator: None,
            vibe_workflow: VibeWorkflowManager::new(),
            vibe_command_handler: VibeCommandHandler::new(),
            filename_suggestion: FilenameSuggestion::new(),
            ai_agent: None,
        }
    }

    pub fn init_ai_client_with_config(&mut self, config: LLMConfig) {
        self.llm_config = Some(config);
        self.update_llm_client();
    }

    fn update_llm_client(&mut self) {
        if let Some(config) = &self.llm_config {
            let client = Arc::new(LLMClient::new(config.clone()));
            self.llm_client = Some(client.clone());
            // 重新创建 conversation_engine 并设置 llm_client
            self.conversation_engine = ConversationEngine::new()
                .with_llm_client(client.clone());
            // 设置 GeminiArchitecture 的 LLM 客户端
            self.gemini.set_llm_client(client.clone());
            // 初始化 ChatOrchestrator
            self.chat_orchestrator = Some(ChatOrchestrator::new(client.clone()));

            // 初始化 AI Agent（类似 grok-cli 的 GrokAgent）
            let agent_config = crate::core::AIAgentConfig {
                max_tool_rounds: 50,
                model: config.model.clone(),
                enable_search: false,
            };
            let ai_agent = crate::core::AIAgent::new(client, agent_config);

            // 注册标准工具
            let agent_clone = ai_agent.clone();
            tokio::spawn(async move {
                agent_clone.register_standard_tools().await;
            });

            self.ai_agent = Some(ai_agent);
        }
    }

    pub fn add_user_message(&mut self, text: &str) {
        self.chat_history.add_message(Message {
            role: Role::User,
            content: text.to_string(),
        });
        // 自动滚动到底部
        self.scroll_to_bottom();
    }

    pub async fn handle_chat_submit(&mut self) {
        let input = self.input_text.clone();
        if input.is_empty() {
            return;
        }

        self.add_user_message(&input);
        self.input_text.clear();
        self.input_cursor = 0; // Reset cursor position
        self.command_hints.clear();
        self.mention_suggestions.close();

        if input.starts_with('/') {
            self.handle_command(&input).await;
        } else if self.llm_client.is_some() {
            // 使用 StreamHandler 进行流式输出
            let handler = StreamHandler::new();
            self.stream_handler = Some(handler.clone());
            self.is_streaming = true;

            // 在聊天历史中预先插入一条空的 AI 消息，用于流式更新
            self.chat_history.add_message(Message {
                role: Role::Assistant,
                content: String::new(),
            });
            self.scroll_to_bottom();

            let client = self.llm_client.as_ref().unwrap().clone();
            let input_clone = input.clone();

            tokio::spawn(async move {
                let handler_clone = handler.clone();
                let callback = move |token: String| {
                    let _ = handler_clone.send_token(token);
                    true
                };

                let messages = vec![ChatMessage {
                    role: "user".to_string(),
                    content: input_clone,
                }];

                match client.generate_completion_stream(messages, None, callback).await {
                    Ok(_) => {
                        let _ = handler.send_done();
                    }
                    Err(e) => {
                        let _ = handler.send_error(e.to_string());
                    }
                }
            });
        } else {
            // 如果 LLM client 未初始化，使用备用方案（Gemini）
            let processed_input = self.process_mentions(&input);
            match self.gemini.chat(processed_input.clone()).await {
                Ok(response) => {
                    self.chat_history.add_message(Message {
                        role: Role::Assistant,
                        content: response.clone(),
                    });
                    self.scroll_to_bottom();
                    self.process_ai_response_for_modifications(&response);
                }
                Err(err) => {
                    self.chat_history.add_message(Message {
                        role: Role::System,
                        content: format!("❌ Gemini 错误: {}", err),
                    });
                    self.scroll_to_bottom();
                }
            }
        }
    }

    /// 处理消息中的 @ 提及，读取文件内容并注入
    fn process_mentions(&self, input: &str) -> String {
        let mut result = input.to_string();
        let mut file_contents = String::new();

        // 查找所有 @path 模式
        let mut i = 0;
        let chars: Vec<char> = input.chars().collect();
        
        while i < chars.len() {
            if chars[i] == '@' {
                // 找到 @ 符号，提取路径
                let mut path = String::new();
                i += 1;
                
                // 收集路径字符（直到空格或结束）
                while i < chars.len() && chars[i] != ' ' && chars[i] != '\n' {
                    path.push(chars[i]);
                    i += 1;
                }
                
                // 尝试读取文件
                if !path.is_empty() {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            file_contents.push_str(&format!(
                                "\n\n<file_content path=\"{}\">\n{}\n</file_content>\n",
                                path, content
                            ));
                            // 从结果中移除 @path
                            result = result.replace(&format!("@{}", path), "");
                        }
                        Err(_) => {
                            // 文件不存在，保留 @path 在消息中
                        }
                    }
                }
            } else {
                i += 1;
            }
        }

        // 清理消息（移除多余空格）并添加文件内容
        let cleaned = result.trim().to_string();
        if file_contents.is_empty() {
            cleaned
        } else {
            format!("{}{}", cleaned, file_contents)
        }
    }

    async fn handle_command(&mut self, input: &str) {
        // 首先尝试解析为vibe命令（/vibc）
        if let Ok(vibe_cmd) = crate::commands::VibeCommandHandler::parse(input) {
            let result = self.vibe_command_handler.execute(vibe_cmd);

            // 显示命令结果
            self.chat_history.add_message(Message {
                role: Role::System,
                content: format!("[vibc] {}", result.message),
            });
            self.scroll_to_bottom();

            // 如果有额外数据，显示它
            if let Some(data) = result.data {
                self.chat_history.add_message(Message {
                    role: Role::System,
                    content: data,
                });
                self.scroll_to_bottom();
            }

            return;
        }

        // 其次尝试解析为文件命令
        if let Some(file_cmd) = FileCommandHandler::parse_command(input) {
            let result = self.file_command_handler.execute(file_cmd);

            // 显示命令结果
            self.chat_history.add_message(Message {
                role: Role::System,
                content: result.message.clone(),
            });
            self.scroll_to_bottom();

            // 如果有 Diff 对比，显示它
            if let Some(diff) = result.diff {
                let diff_content = format!(
                    "--- {} (原始)\n+++{} (新版本)\n{}",
                    diff.file_path,
                    diff.file_path,
                    format_diff(&diff.old_content, &diff.new_content)
                );
                self.chat_history.add_message(Message {
                    role: Role::System,
                    content: diff_content,
                });
                self.scroll_to_bottom();
            }

            return;
        }

        // 再次尝试解析为普通命令
        if let Some(cmd) = CommandParser::parse(input) {
            let response = match cmd.command_type {
                CommandType::Help => CommandParser::get_help_text(),
                CommandType::Clear => {
                    self.chat_history.clear();
                    "✓ Chat history cleared".to_string()
                }
                // NOTE: Other command handlers would go here
                _ => format!("Unknown command: {}", input),
            };

            self.chat_history.add_message(Message {
                role: Role::System,
                content: response,
            });
            self.scroll_to_bottom();
        }
    }

    /// 处理 AI 响应中的代码修改指令
    pub fn process_ai_response_for_modifications(&mut self, response: &str) {
        // 首先检测明确的修改指令
        let mut ops = AICodeModificationDetector::detect_modifications(response);

        // 如果没有明确指令，检测隐含的修改意图
        if ops.is_empty() {
            ops = AICodeModificationDetector::detect_implicit_modifications(response);
        }

        if ops.is_empty() {
            return;
        }

        // 检查是否有未指定文件名的操作（需要用户确认）
        let mut needs_filename_confirmation = false;
        for op in &ops {
            if let CodeModificationOp::Modify { path, search: _, replace: _ } = op {
                if path.starts_with("UNSPECIFIED_") {
                    needs_filename_confirmation = true;
                    break;
                }
            }
        }

        if needs_filename_confirmation {
            // ✅ 显示文件名建议对话框，让用户选择
            // 提取代码内容和语言
            let mut unspecified_content = String::new();
            let mut detected_language = String::new();

            for op in &ops {
                if let CodeModificationOp::Modify { path, search: _, replace } = op {
                    if path.starts_with("UNSPECIFIED_") {
                        unspecified_content = replace.clone();
                        // 从路径中提取语言（如 UNSPECIFIED_main.rs -> rs）
                        if let Some(ext_start) = path.rfind('.') {
                            detected_language = path[ext_start + 1..].to_string();
                        }
                        break;
                    }
                }
            }

            self.filename_suggestion.show(unspecified_content, detected_language);
            return; // 不进入修改确认流程，让用户通过对话框选择
        }

        // 为每个修改操作生成 Diff
        for op in ops {
            let diff = match &op {
                CodeModificationOp::Create { path, content } => {
                    // 创建操作：显示新内容
                    Some(CodeDiff {
                        file_path: path.clone(),
                        old_content: String::new(),
                        new_content: content.clone(),
                    })
                }
                CodeModificationOp::Modify { path, search, replace } => {
                    // 修改操作：尝试匹配并生成 Diff
                    match CodeMatcher::find_and_replace(path, search, replace) {
                        Ok(diff) => Some(diff),
                        Err(e) => {
                            // 匹配失败，显示错误信息
                            self.chat_history.add_message(Message {
                                role: Role::System,
                                content: format!("❌ 代码匹配失败: {}", e),
                            });
                            None
                        }
                    }
                }
                CodeModificationOp::Delete { path } => {
                    // 删除操作：显示文件路径
                    Some(CodeDiff {
                        file_path: path.clone(),
                        old_content: format!("(删除文件: {})", path),
                        new_content: String::new(),
                    })
                }
            };

            if let Some(diff) = diff {
                self.pending_modifications.push((op, Some(diff)));
            }
        }

        // 如果有待确认的修改，激活确认对话
        if !self.pending_modifications.is_empty() {
            self.modification_confirmation_pending = true;
            self.modification_selected_index = 0;
            self.modification_choice = ModificationChoice::Confirm;

            // 确认对话现在作为独立的 UI 层显示，不添加到聊天历史
        }
    }

    /// 生成系统提示，用于改进 AI 配对编程的回复质量
    /// 
    /// 使用 prompts 模块中的提示词生成器，根据对话历史长度生成适应性提示
    fn generate_system_prompt(&self) -> String {
        let message_count = self.chat_history.get_messages().len();
        prompts::get_pair_programming_prompt(message_count)
    }

    pub async fn start_streaming_chat(&mut self, prompt: &str) {
        if self.llm_client.is_none() {
            return;
        }
        self.is_streaming = true;
        
        // 在聊天历史中预先插入一条空的 AI 消息，用于流式更新
        self.chat_history.add_message(Message {
            role: Role::Assistant,
            content: String::new(),
        });
        self.scroll_to_bottom();
        
        let handler = StreamHandler::new();
        self.stream_handler = Some(handler.clone());

        let client = self.llm_client.as_ref().unwrap().clone();
        let prompt = prompt.to_string();
        let system_prompt = self.generate_system_prompt();

        tokio::spawn(async move {
            let handler_clone = handler.clone();
            let callback = move |token: String| {
                let _ = handler_clone.send_token(token);
                true
            };

            // 构建消息数组，包含系统提示和用户消息
            let messages = vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ];

            match client.generate_completion_stream(messages, None, callback).await {
                Ok(_) => {
                    let _ = handler.send_done();
                }
                Err(e) => {
                    let _ = handler.send_error(e.to_string());
                }
            }
        });
    }

    pub fn render(&mut self, f: &mut Frame) {
        // 使用像素艺术风格布局 (v2 - 4x4 头像)
        self.frame_count = self.frame_count.wrapping_add(1);
        ui::pixel_layout_v2::render_pixel_layout(f, self);
    }

    pub async fn finalize_streaming_response(&mut self) {
        let ai_response_opt = {
            let mut response = self.streaming_response.lock().unwrap();
            if !response.content.is_empty() {
                let content = response.content.clone();
                response.reset();
                Some(content)
            } else {
                response.reset();
                None
            }
        };
        
        // 在释放 response 借用后，处理 AI 响应中的代码修改指令
        if let Some(ai_response) = ai_response_opt {
            // 此时聊天历史中的最后一条 AI 消息已经在流式过程中被更新完毕
            // 这里不再重复添加，只用于解析代码修改指令
            
            // 自动滚动到底部
            // self.scroll_to_bottom();
            
            // 检测修改指令并立即显示确认对话
            // 不等待用户继续输入
            self.process_ai_response_for_modifications(&ai_response);
        }
        
        self.is_streaming = false;
        self.stream_handler = None;
    }
    
    /// 滚动到聊天历史底部
    pub fn scroll_to_bottom(&mut self) {
        self.chat_scroll_offset = 0;
    }
}