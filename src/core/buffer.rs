use ropey::Rope;
use std::sync::{Arc, Mutex};

/// Core buffer structure for the editor
pub struct Buffer {
    pub content: Rope,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            content: Rope::new(),
        }
    }

    pub fn from_string(content: String) -> Self {
        Self {
            content: Rope::from_str(&content),
        }
    }

    pub fn insert_char(&mut self, row: usize, col: usize, c: char) {
        let char_idx = self.content.line_to_char(row) + col;
        self.content.insert(char_idx, &c.to_string());
    }

    pub fn insert_text(&mut self, row: usize, col: usize, text: &str) {
        let char_idx = self.content.line_to_char(row) + col;
        self.content.insert(char_idx, text);
    }

    pub fn remove_char(&mut self, row: usize, col: usize) {
        let char_idx = self.content.line_to_char(row) + col;
        if char_idx < self.content.len_chars() {
            self.content.remove(char_idx..char_idx + 1);
        }
    }

    pub fn remove_range(&mut self, start_row: usize, start_col: usize, end_row: usize, end_col: usize) {
        let start_idx = self.content.line_to_char(start_row) + start_col;
        let end_idx = self.content.line_to_char(end_row) + end_col;
        if start_idx < self.content.len_chars() && end_idx <= self.content.len_chars() && start_idx < end_idx {
            self.content.remove(start_idx..end_idx);
        }
    }

    pub fn get_line(&self, line_num: usize) -> Option<String> {
        if line_num < self.content.len_lines() {
            Some(self.content.line(line_num).to_string())
        } else {
            None
        }
    }

    pub fn get_text(&self) -> String {
        self.content.to_string()
    }

    pub fn len_lines(&self) -> usize {
        self.content.len_lines()
    }

    pub fn line_len_chars(&self, line_num: usize) -> usize {
        if line_num < self.content.len_lines() {
            self.content.line(line_num).len_chars()
        } else {
            0
        }
    }
}