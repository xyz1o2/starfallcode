use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_editor(f: &mut Frame, app: &App) {
    let size = f.size();

    // Create layout with main editor and chat area at the bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),  // Main editor area (at least 5 rows)
            Constraint::Min(3),  // Chat area
        ])
        .split(size);

    // Render main editor
    render_main_editor(f, app, chunks[0]);

    // Render chat area
    render_chat_area(f, app, chunks[1]);
}

fn render_main_editor(f: &mut Frame, app: &App, area: Rect) {
    // Convert rope to text for display
    let mut text_lines = Vec::new();
    for i in 0..app.buffer.len_lines() {
        let line = app.buffer.line(i).to_string();
        text_lines.push(Line::from(Span::raw(line)));
    }

    // Create paragraph widget for main text
    let title = "📝 Editor (Tab to switch to chat)";

    let paragraph = Paragraph::new(text_lines)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(paragraph, area);

    // Render ghost text if it exists
    if let Some(ghost) = &app.ghost_text {
        render_ghost_text(f, app, ghost, area);
    }

    // Position cursor only if not in chat mode
    if !app.is_chat_focused {
        let (cursor_row, cursor_col) = app.cursor;
        if area.height > 0 && area.width > 0 {
            // Calculate screen position of cursor
            let screen_x = area.x + 1 + cursor_col as u16; // +1 for border
            let screen_y = area.y + 1 + cursor_row as u16; // +1 for border

            // Make sure cursor is within bounds
            if screen_x < area.x + area.width - 1 && screen_y < area.y + area.height - 1 {
                f.set_cursor(screen_x, screen_y);
            }
        }
    }
}

fn render_chat_area(f: &mut Frame, app: &App, area: Rect) {
    // Split chat area into history and input
    let chat_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),      // Chat history
            Constraint::Length(2),   // Chat input
        ])
        .split(area);

    // Render chat history
    render_chat_history(f, app, chat_chunks[0]);

    // Render chat input box
    let title = if app.is_chat_focused {
        "💬 Chat (Press Enter to send, Tab to return to editor, /help for commands)"
    } else {
        "💬 Chat (Press Tab to enter chat)"
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(title);

    // Create paragraph widget for the input
    let input_paragraph = Paragraph::new(app.chat_input.as_str())
        .block(input_block);

    f.render_widget(input_paragraph, chat_chunks[1]);

    // Set cursor for chat input if focused
    if app.is_chat_focused {
        let cursor_x = chat_chunks[1].x + 1 + app.chat_input.len() as u16; // +1 for border
        let cursor_y = chat_chunks[1].y + 1; // First line of the input area

        // Make sure cursor is within bounds
        if cursor_x < chat_chunks[1].x + chat_chunks[1].width - 1 {
            f.set_cursor(cursor_x, cursor_y);
        }
    }
}

fn render_chat_history(f: &mut Frame, app: &App, area: Rect) {
    let mut chat_lines = Vec::new();

    // Add chat history messages
    for msg in &app.chat_history {
        let role_prefix = match msg.role.as_str() {
            "user" => "👤 You",
            "assistant" => "🤖 AI",
            "system" => "⚙️ System",
            _ => "📝 Message",
        };

        let line = Line::from(Span::raw(format!("{}: {}", role_prefix, msg.content)));
        chat_lines.push(line);
        chat_lines.push(Line::from("")); // Empty line for spacing
    }

    // If no messages, show welcome message
    if chat_lines.is_empty() {
        chat_lines.push(Line::from(Span::raw(
            "Welcome to the chat! Type /help for commands or @mention for references.",
        )));
    }

    let history_block = Block::default()
        .borders(Borders::ALL)
        .title("📜 Chat History");

    let history_paragraph = Paragraph::new(chat_lines)
        .block(history_block)
        .scroll((0, 0)); // TODO: Add scrolling support

    f.render_widget(history_paragraph, area);
}

fn render_ghost_text(f: &mut Frame, app: &App, ghost: &crate::app::GhostText, area: Rect) {
    let (ghost_row, ghost_col) = ghost.start_pos;

    // Only render ghost text if it's on the visible screen
    if ghost_row >= app.scroll.0 as usize && ghost_row < app.scroll.0 as usize + area.height as usize {
        // Calculate the position for ghost text
        let screen_y = area.y + 1 + (ghost_row - app.scroll.0 as usize) as u16;
        let screen_x = area.x + 1 + ghost_col as u16;

        // Create ghost text widget
        let ghost_widget = Paragraph::new(Line::from(Span::styled(
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