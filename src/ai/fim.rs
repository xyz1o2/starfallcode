use crate::core::buffer::Buffer;

pub struct FIMContext {
    pub prefix: String,
    pub suffix: String,
    pub cursor_pos: (usize, usize), // (row, col)
}

pub struct FIMProcessor;

impl FIMProcessor {
    pub fn extract_fim_context(buffer: &Buffer, cursor_row: usize, cursor_col: usize, max_prefix_lines: usize, max_suffix_lines: usize) -> FIMContext {
        let total_lines = buffer.len_lines();
        
        // Calculate the start and end rows for prefix and suffix
        let prefix_start_row = if cursor_row > max_prefix_lines {
            cursor_row - max_prefix_lines
        } else {
            0
        };
        
        let suffix_end_row = if cursor_row + max_suffix_lines < total_lines {
            cursor_row + max_suffix_lines
        } else {
            total_lines.min(cursor_row + max_suffix_lines)
        };
        
        // Build prefix (text before cursor)
        let mut prefix = String::new();
        for row in prefix_start_row..cursor_row {
            if let Some(line) = buffer.get_line(row) {
                prefix.push_str(&line);
                prefix.push('\n');
            }
        }
        
        // Add the current line up to the cursor position
        if let Some(current_line) = buffer.get_line(cursor_row) {
            if cursor_col <= current_line.len() {
                prefix.push_str(&current_line[..cursor_col]);
            }
        }
        
        // Build suffix (text after cursor)
        let mut suffix = String::new();
        if let Some(current_line) = buffer.get_line(cursor_row) {
            if cursor_col < current_line.len() {
                suffix.push_str(&current_line[cursor_col..]);
            }
        }
        
        for row in (cursor_row + 1)..=suffix_end_row.min(total_lines - 1) {
            if let Some(line) = buffer.get_line(row) {
                suffix.push_str("\n");
                suffix.push_str(&line);
            }
        }
        
        FIMContext {
            prefix,
            suffix,
            cursor_pos: (cursor_row, cursor_col),
        }
    }
    
    pub fn build_fim_prompt(context: &FIMContext) -> String {
        format!(
            "Complete the code between the <PRE> and <SUF> tags. The <MID> tag represents where the completion should go.\n<PRE>\n{}\n</PRE>\n<SUF>\n{}\n</SUF>\n<MID>",
            context.prefix, context.suffix
        )
    }
}