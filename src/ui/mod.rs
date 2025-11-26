pub mod layout;
pub mod sidebar;
pub mod main_chat;
pub mod info_panel;
pub mod status_bar;
pub mod theme;
pub mod focus;
pub mod types;
pub mod command_hints;

pub use layout::LayoutManager;
pub use sidebar::Sidebar;
pub use main_chat::MainChatArea;
pub use info_panel::InfoPanel;
pub use status_bar::ModernStatusBar;
pub use theme::ModernTheme;
pub use focus::FocusManager;
pub use command_hints::CommandHints;
pub use types::*;

use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Temporary render function to replace the old chat rendering
/// This is a minimal implementation to get the project compiling
pub fn render_modern_ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // Main layout: header, chat history, input
    let input_area_height = if app.command_hints.visible {
        12 // 当提示可见时，分配更多空间 (4 for input + 8 for hints)
    } else {
        4  // 默认高度
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(5),         // Chat history
            Constraint::Length(input_area_height), // 动态输入区域
        ])
        .split(size);

    // Render header
    render_header(f, app, chunks[0]);

    // Render chat history
    render_history(f, app, chunks[1]);

    // Render input area
    render_input(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
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

fn render_history(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::new();

    if app.chat_history.is_empty() && !app.is_streaming {
        lines.push(Line::from(vec![
            Span::styled(
                "Welcome to Modern AI Chat! 👋",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("New Modern UI Components Loaded Successfully! "),
            Span::styled("✅", Style::default().fg(Color::Green)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("Commands: "),
            Span::styled("/help", Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled("/clear", Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled("/status", Style::default().fg(Color::Yellow)),
        ]));
    } else {
        for msg in &app.chat_history {
            let (prefix, color) = match msg.role.as_str() {
                "user" => ("👤 You", Color::Blue),
                "assistant" => ("🤖 AI", Color::Green),
                "system" => ("⚙️ System", Color::Yellow),
                _ => ("📝 Message", Color::White),
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
            lines.push(Line::from(vec![
                Span::styled(
                    "🤖 AI: ",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ⏳", app.streaming_response.blocking_lock().get_content()),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }
    }

    let history = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" 💬 Modern Chat History "))
        .wrap(Wrap { trim: true });

    f.render_widget(history, area);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
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

    let hint = if app.chat_input.is_empty() {
        "Type your message... (输入 / 查看命令 - Ctrl+C to exit)"
    } else {
        "Press Enter to send, Backspace to delete"
    };
    let hint_line = Paragraph::new(Line::from(Span::styled(
        hint,
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
    )));
    f.render_widget(hint_line, input_chunks[0]);

    let input_widget = Paragraph::new(app.chat_input.as_str())
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Cyan)))
        .wrap(Wrap { trim: true });
    f.render_widget(input_widget, input_chunks[1]);

    f.set_cursor(
        input_chunks[1].x + 1 + app.chat_input.len() as u16,
        input_chunks[1].y + 1,
    );

    // 在 hints_area 中渲染命令提示
    if app.command_hints.visible && hints_area.height > 0 {
        app.command_hints.render(f, hints_area, &ModernTheme::dark_professional());
    }
}