use crate::ai::client::LLMClient;
use crate::ai::commands::{CommandParser, CommandType};
use crate::ai::config::LLMConfig;
use crate::ai::streaming::{StreamHandler, StreamingChatResponse};
use crate::core::history::ChatHistory;
use crate::core::message::{Message, Role};
use crate::ui::command_hints::CommandHints;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::ui;

#[derive(Debug, PartialEq)]
pub enum AppAction {
    None,
    Quit,
    SubmitChat,
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

    pub async fn start_streaming_chat(&mut self, prompt: &str) {
        if let Some(ref client) = self.llm_client {
            self.is_streaming = true;
            let handler = StreamHandler::new();
            self.stream_handler = Some(handler.clone());

            let client = client.clone();
            let prompt = prompt.to_string();

            tokio::spawn(async move {
                let handler_clone = handler.clone();
                let callback = move |token: String| {
                    let _ = handler_clone.send_token(token);
                    true
                };

                match client.generate_completion_stream(&prompt, callback).await {
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
                Constraint::Min(5),    // Chat history
                Constraint::Length(if self.command_hints.visible { 12 } else { 4 }), // Input area
            ])
            .split(f.size());

        ui::render_header(f, self, chunks[0]);
        ui::render_history(f, self, chunks[1]);
        ui::render_input(f, self, chunks[2]);
    }

    pub async fn finalize_streaming_response(&mut self) {
        let mut response = self.streaming_response.lock().await;
        if !response.content.is_empty() {
            self.chat_history.add_message(Message {
                role: Role::Assistant,
                content: response.content.clone(),
            });
        }
        response.reset();
        self.is_streaming = false;
        self.stream_handler = None;
    }
}