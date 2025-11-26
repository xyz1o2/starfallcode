use crate::ai::client::LLMClient;
use crate::ai::commands::{CommandParser, CommandType};
use crate::ai::config::LLMConfig;
use crate::ai::streaming::{StreamHandler, StreamingChatResponse};
use crate::ai::code_modification::{AICodeModificationDetector, CodeModificationOp, CodeDiff, CodeMatcher};
use crate::core::history::ChatHistory;
use crate::core::message::{Message, Role};
use crate::ui::command_hints::CommandHints;
use crate::commands::FileCommandHandler;
use crate::prompts;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::ui;

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
    Quit,
    SubmitChat,
}

/// 代码修改确认选择
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModificationChoice {
    Confirm,
    Cancel,
}

pub struct App {
    pub should_quit: bool,
    pub chat_history: ChatHistory,
    pub input_text: String,
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
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            chat_history: ChatHistory::new(100),
            input_text: String::new(),
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
        }
    }

    pub fn init_ai_client_with_config(&mut self, config: LLMConfig) {
        self.llm_config = Some(config);
        self.update_llm_client();
    }

    fn update_llm_client(&mut self) {
        if let Some(config) = &self.llm_config {
            self.llm_client = Some(Arc::new(LLMClient::new(config.clone())));
        }
    }

    pub fn add_user_message(&mut self, text: &str) {
        self.chat_history.add_message(Message {
            role: Role::User,
            content: text.to_string(),
        });
    }

    pub async fn handle_chat_submit(&mut self) {
        let input = self.input_text.clone();
        if input.is_empty() {
            return;
        }

        self.add_user_message(&input);
        self.input_text.clear();
        self.command_hints.clear();

        if input.starts_with('/') {
            self.handle_command(&input).await;
        } else {
            self.start_streaming_chat(&input).await;
        }
    }

    async fn handle_command(&mut self, input: &str) {
        // 首先尝试解析为文件命令
        if let Some(file_cmd) = FileCommandHandler::parse_command(input) {
            let result = self.file_command_handler.execute(file_cmd);
            
            // 显示命令结果
            self.chat_history.add_message(Message {
                role: Role::System,
                content: result.message.clone(),
            });
            
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
            }
            
            return;
        }

        // 其次尝试解析为普通命令
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
        if let Some(ref client) = self.llm_client {
            self.is_streaming = true;
            let handler = StreamHandler::new();
            self.stream_handler = Some(handler.clone());

            let client = client.clone();
            let prompt = prompt.to_string();
            let system_prompt = self.generate_system_prompt();

            tokio::spawn(async move {
                let handler_clone = handler.clone();
                let callback = move |token: String| {
                    let _ = handler_clone.send_token(token);
                    true
                };

                // 构建完整的提示，包含系统提示和用户消息
                let full_prompt = format!("System: {}\n\nUser: {}", system_prompt, prompt);

                match client.generate_completion_stream(&full_prompt, callback).await {
                    Ok(_) => {
                        let _ = handler.send_done();
                    }
                    Err(e) => {
                        let _ = handler.send_error(e.to_string());
                    }
                }
            });
        }
    }

        pub fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Chat history (flexible, takes remaining space)
                Constraint::Length(if self.command_hints.visible { 10 } else { 4 }), // Input area (max 10 with hints)
            ])
            .split(f.size());

        ui::render_header(f, self, chunks[0]);
        ui::render_history(f, self, chunks[1]);
        ui::render_input(f, self, chunks[2]);
        
        // 如果有待确认的修改，显示确认对话
        if self.modification_confirmation_pending {
            ui::render_modification_confirmation(f, self, f.size());
        }
    }

    pub async fn finalize_streaming_response(&mut self) {
        let ai_response = {
            let mut response = self.streaming_response.lock().await;
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
        if let Some(ai_response) = ai_response {
            self.chat_history.add_message(Message {
                role: Role::Assistant,
                content: ai_response.clone(),
            });
            self.process_ai_response_for_modifications(&ai_response);
        }
        
        self.is_streaming = false;
        self.stream_handler = None;
    }
}