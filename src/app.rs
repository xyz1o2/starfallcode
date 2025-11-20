use ropey::Rope;
use std::time::{Duration, Instant};

pub struct App {
    pub buffer: Rope,
    pub cursor: (usize, usize),  // (row, col)
    pub scroll: (u16, u16),
    pub ghost_text: Option<GhostText>,
    pub debouncer: Debouncer,
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
        }
    }

    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Char(c) => {
                // Insert character at cursor position
                let (row, col) = self.cursor;
                let char_idx = self.buffer.line_to_char(row) + col;
                self.buffer.insert(char_idx, &c.to_string());
                
                // Move cursor forward
                self.cursor.1 += 1;
                
                // Trigger debouncer for AI completion
                if self.debouncer.trigger() {
                    // For now, just set a simple ghost text for testing
                    self.ghost_text = Some(GhostText {
                        content: "hello_world()".to_string(),
                        start_pos: self.cursor,
                        is_streaming: false,
                    });
                }
            }
            crossterm::event::KeyCode::Backspace => {
                let (row, col) = self.cursor;
                if col > 0 {
                    let char_idx = self.buffer.line_to_char(row) + col - 1;
                    self.buffer.remove(char_idx..char_idx + 1);
                    self.cursor.1 -= 1;
                }
            }
            crossterm::event::KeyCode::Enter => {
                let (row, _) = self.cursor;
                let char_idx = self.buffer.line_to_char(row) + self.cursor.1;
                self.buffer.insert(char_idx, "\n");
                self.cursor.0 += 1;
                self.cursor.1 = 0;
            }
            crossterm::event::KeyCode::Left => {
                if self.cursor.1 > 0 {
                    self.cursor.1 -= 1;
                }
            }
            crossterm::event::KeyCode::Right => {
                let line_len = self.buffer.line(self.cursor.0).len_chars();
                if self.cursor.1 < line_len {
                    self.cursor.1 += 1;
                }
            }
            crossterm::event::KeyCode::Up => {
                if self.cursor.0 > 0 {
                    self.cursor.0 -= 1;
                    let line_len = self.buffer.line(self.cursor.0).len_chars();
                    if self.cursor.1 > line_len {
                        self.cursor.1 = line_len;
                    }
                }
            }
            crossterm::event::KeyCode::Down => {
                if self.cursor.0 < self.buffer.len_lines() - 1 {
                    self.cursor.0 += 1;
                    let line_len = self.buffer.line(self.cursor.0).len_chars();
                    if self.cursor.1 > line_len {
                        self.cursor.1 = line_len;
                    }
                }
            }
            _ => {}
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