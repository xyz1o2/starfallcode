pub mod layout;
pub mod sidebar;
pub mod main_chat;
pub mod info_panel;
pub mod theme;
pub mod focus;
pub mod types;
pub mod command_hints;

pub use theme::ModernTheme;
use crate::app::App;
use unicode_width::UnicodeWidthStr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let model_str = app.llm_config
        .as_ref()
        .map(|c| c.model.as_str())
        .unwrap_or("Not configured");
    let provider_str = app.llm_config
        .as_ref()
        .map(|c| c.provider.to_string())
        .unwrap_or_default();
    
    let header_text = vec![
        Line::from(vec![
            Span::styled(
                "🤖 AI Pair Programming Chat (Modern UI)",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Model: "),
            Span::styled(
                model_str,
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" | Provider: "),
            Span::styled(
                provider_str.as_str(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "─".repeat(area.width as usize),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Left);

    f.render_widget(header, area);
}

pub fn render_history(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::new();

    if app.chat_history.is_empty() && !app.is_streaming {
        // ... (welcome message remains the same)
    } else {
        for msg in app.chat_history.get_messages() {
            let (prefix, color) = match msg.role {
                crate::core::message::Role::User => ("👤 You", Color::Blue),
                crate::core::message::Role::Assistant => ("🤖 AI", Color::Green),
                crate::core::message::Role::System => ("⚙️ System", Color::Yellow),
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", prefix),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(&msg.content),
            ]));
            lines.push(Line::from(""));
        }

        if app.is_streaming {
            let streaming_content = app.streaming_response.try_lock()
                .map(|resp| resp.content.clone())
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::styled(
                    "🤖 AI: ",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ⏳", streaming_content),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }
    }

    let history = Paragraph::new(lines)
        .wrap(Wrap { trim: true });

    f.render_widget(history, area);
}

pub fn render_input(f: &mut Frame, app: &App, area: Rect) {
    // 将接收到的区域分割为输入区和提示区
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // 固定输入区高度为4
            Constraint::Min(0),    // 剩余空间给提示区
        ])
        .split(area);

    let input_area = chunks[0];
    let hints_area = chunks[1];

    // 在 input_area 中渲染输入框
    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(input_area);

    let hint = if app.input_text.is_empty() {
        "Type your message... (Type / for commands - Ctrl+C to exit)"
    } else {
        "Press Enter to send, Backspace to delete"
    };
    let hint_line = Paragraph::new(Line::from(Span::styled(
        hint,
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
    )));
    f.render_widget(hint_line, input_chunks[0]);

    let input_widget = Paragraph::new(app.input_text.as_str())
        .block(Block::default().borders(Borders::ALL).title(" 💬 Input ").style(Style::default().fg(Color::Cyan)));
    f.render_widget(input_widget, input_chunks[1]);

    // 光标位置：使用 unicode-width 计算准确的显示宽度
    // x: 区域左边界 + 左边框(1) + 显示宽度
    // y: 区域顶部 + 上边框(1)
    let display_width = app.input_text.width() as u16;
    
    let cursor_x = input_chunks[1].x + 1 + display_width;
    let cursor_y = input_chunks[1].y + 1;
    
    // 确保光标在有效范围内
    if cursor_x < input_chunks[1].right() && cursor_y < input_chunks[1].bottom() {
        f.set_cursor(cursor_x, cursor_y);
    }

    // 在 hints_area 中渲染命令提示
    if app.command_hints.visible && hints_area.height > 0 {
        app.command_hints.render(f, hints_area, &ModernTheme::dark_professional());
    }
}