use crate::app::App;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_editor<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(size);
    
    // Convert rope to text for display
    let mut text_lines = Vec::new();
    for i in 0..app.buffer.len_lines() {
        let line = app.buffer.line(i).to_string();
        text_lines.push(Spans::from(Span::raw(line)));
    }
    
    // Create paragraph widget for main text
    let paragraph = Paragraph::new(text_lines)
        .block(Block::default().borders(Borders::ALL).title("Editor"));
    
    f.render_widget(paragraph, chunks[0]);
    
    // Render ghost text if it exists
    if let Some(ghost) = &app.ghost_text {
        render_ghost_text(f, app, ghost, chunks[0]);
    }
    
    // Position cursor
    let (cursor_row, cursor_col) = app.cursor;
    if chunks[0].height > 0 && chunks[0].width > 0 {
        // Calculate screen position of cursor
        let screen_x = chunks[0].x + 1 + cursor_col as u16; // +1 for border
        let screen_y = chunks[0].y + 1 + cursor_row as u16; // +1 for border
        
        // Make sure cursor is within bounds
        if screen_x < chunks[0].x + chunks[0].width - 1 && screen_y < chunks[0].y + chunks[0].height - 1 {
            f.set_cursor(screen_x, screen_y);
        }
    }
}

fn render_ghost_text<B: Backend>(f: &mut Frame<B>, app: &App, ghost: &crate::app::GhostText, area: Rect) {
    let (ghost_row, ghost_col) = ghost.start_pos;
    
    // Only render ghost text if it's on the visible screen
    if ghost_row >= app.scroll.0 as usize && ghost_row < app.scroll.0 as usize + area.height as usize {
        // Calculate the position for ghost text
        let screen_y = area.y + 1 + (ghost_row - app.scroll.0 as usize) as u16;
        let screen_x = area.x + 1 + ghost_col as u16;
        
        // Create ghost text widget
        let ghost_widget = Paragraph::new(Spans::from(Span::styled(
            &ghost.content,
            Style::default().fg(Color::Rgb(120, 120, 180)).add_modifier(ratatui::style::Modifier::DIM),
        )));
        
        // Calculate area for ghost text
        let ghost_area = Rect {
            x: screen_x,
            y: screen_y,
            width: std::cmp::min(ghost.content.len() as u16, area.width - screen_x + area.x),
            height: 1,
        };
        
        // Render ghost text
        f.render_widget(ghost_widget, ghost_area);
    }
}