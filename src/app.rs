use ropey::Rope;
use std::time::{Duration, Instant};
use tokio::task;
use crate::ai::{client::LLMClient, fim::{FIMProcessor, FIMContext}, context::RAGContextBuilder};

pub struct App {
    pub buffer: Rope,
    pub cursor: (usize, usize),  // (row, col)
    pub scroll: (u16, u16),
    pub ghost_text: Option<GhostText>,
    pub debouncer: Debouncer,
    pub llm_client: Option<LLMClient>,
    pub project_context: Option<crate::ai::context::ProjectContext>,
}

pub struct GhostText {
    pub content: String,
    pub start_pos: (usize, usize),  // (row, col) where ghost text starts
    pub is_streaming: bool,
}

pub struct Debouncer {
    last_input: Instant,
    delay: Duration,
}

impl App {
    pub fn new() -> Self {
        Self {
            buffer: Rope::new(),
            cursor: (0, 0),
            scroll: (0, 0),
            ghost_text: None,
            debouncer: Debouncer::new(Duration::from_millis(300)),
            llm_client: None,
            project_context: None,
        }
    }

    pub fn init_ai_client(&mut self, api_key: String) {
        let config = crate::ai::client::LLMConfig {
            api_key,
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            temperature: 0.7,
            max_tokens: 200,
        };
        
        self.llm_client = Some(LLMClient::new(config));
    }

    pub fn init_project_context(&mut self, project_path: &str) {
        if let Ok(context) = RAGContextBuilder::scan_project(project_path) {
            self.project_context = Some(context);
        }
    }

    pub fn handle_char_input(&mut self, c: char) {
        let (row, col) = self.cursor;
        let char_idx = self.buffer.line_to_char(row) + col;
        self.buffer.insert(char_idx, &c.to_string());
        
        // Move cursor forward
        self.cursor.1 += 1;
    }

    pub fn handle_backspace(&mut self) {
        let (row, col) = self.cursor;
        if col > 0 {
            let char_idx = self.buffer.line_to_char(row) + col - 1;
            self.buffer.remove(char_idx..char_idx + 1);
            self.cursor.1 -= 1;
        }
    }

    pub fn handle_enter(&mut self) {
        let (row, _) = self.cursor;
        let char_idx = self.buffer.line_to_char(row) + self.cursor.1;
        self.buffer.insert(char_idx, "\n");
        self.cursor.0 += 1;
        self.cursor.1 = 0;
    }

    pub fn handle_left(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            let line_len = self.buffer.line(self.cursor.0).len_chars();
            self.cursor.1 = line_len;
        }
    }

    pub fn handle_right(&mut self) {
        let line_len = self.buffer.line(self.cursor.0).len_chars();
        if self.cursor.1 < line_len {
            self.cursor.1 += 1;
        } else if self.cursor.0 + 1 < self.buffer.len_lines() {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
    }

    pub fn handle_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            let line_len = self.buffer.line(self.cursor.0).len_chars();
            if self.cursor.1 > line_len {
                self.cursor.1 = line_len;
            }
        }
    }

    pub fn handle_down(&mut self) {
        if self.cursor.0 + 1 < self.buffer.len_lines() {
            self.cursor.0 += 1;
            let line_len = self.buffer.line(self.cursor.0).len_chars();
            if self.cursor.1 > line_len {
                self.cursor.1 = line_len;
            }
        }
    }

    pub fn trigger_completion(&mut self) {
        if self.debouncer.trigger() {
            self.request_ai_completion();
        }
    }

    fn request_ai_completion(&mut self) {
        if let Some(ref client) = self.llm_client {
            let buffer_content = self.buffer.clone();
            let (cursor_row, cursor_col) = self.cursor;
            let project_context = self.project_context.clone();
            
            // Spawn async task to get AI completion
            task::spawn(async move {
                // Extract FIM context
                let fim_context = FIMProcessor::extract_fim_context(
                    &buffer_content.clone().into(), // We'll need to implement a conversion
                    cursor_row,
                    cursor_col,
                    50, // max_prefix_lines
                    20, // max_suffix_lines
                );
                
                let prompt = FIMProcessor::build_fim_prompt(&fim_context);
                
                // For now, just return a simple completion
                // In a real implementation, we would call the LLM client
                let completion = " // AI completion would appear here".to_string();
                
                // This is where we would update the ghost text, but we need to use channels
                // to communicate back to the main app thread
            });
        }
    }

    pub fn accept_ghost_text(&mut self) {
        if let Some(ghost) = &self.ghost_text {
            let insert_pos = self.buffer.line_to_char(ghost.start_pos.0) + ghost.start_pos.1;
            self.buffer.insert(insert_pos, &ghost.content);
            self.ghost_text = None;
            
            // Update cursor position after insertion
            self.cursor.1 = ghost.start_pos.1 + ghost.content.len();
        }
    }

    pub fn clear_ghost_text(&mut self) {
        self.ghost_text = None;
    }
}

impl Debouncer {
    pub fn new(delay: Duration) -> Self {
        Self {
            last_input: Instant::now(),
            delay,
        }
    }

    pub fn trigger(&mut self) -> bool {
        if self.last_input.elapsed() > self.delay {
            self.last_input = Instant::now();
            true
        } else {
            self.last_input = Instant::now();
            false
        }
    }
}

// Implement conversion from Rope to Buffer for compatibility
impl From<Rope> for crate::core::buffer::Buffer {
    fn from(rope: Rope) -> Self {
        Self {
            content: rope,
        }
    }
}