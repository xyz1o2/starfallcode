use crate::core::buffer::Buffer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn move_left(&mut self, buffer: &Buffer) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            if self.row < buffer.len_lines() {
                self.col = buffer.line_len_chars(self.row);
            } else {
                self.col = 0;
            }
        }
    }

    pub fn move_right(&mut self, buffer: &Buffer) {
        let current_line_len = if self.row < buffer.len_lines() {
            buffer.line_len_chars(self.row)
        } else {
            0
        };
        
        if self.col < current_line_len {
            self.col += 1;
        } else if self.row + 1 < buffer.len_lines() {
            self.row += 1;
            self.col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
        }
    }

    pub fn move_down(&mut self, buffer: &Buffer) {
        if self.row + 1 < buffer.len_lines() {
            self.row += 1;
            let current_line_len = buffer.line_len_chars(self.row);
            if self.col > current_line_len {
                self.col = current_line_len;
            }
        }
    }

    pub fn move_to_line_start(&mut self, buffer: &Buffer) {
        self.col = 0;
    }

    pub fn move_to_line_end(&mut self, buffer: &Buffer) {
        if self.row < buffer.len_lines() {
            self.col = buffer.line_len_chars(self.row);
        }
    }

    pub fn position(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    pub fn set_position(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }
}