use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_chat(f: &mut Frame, app: &App) {
    let size = f.size();

    // Main layout: header, chat history, input
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(5),         // Chat history
            Constraint::Length(4),      // Input area
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
                "🤖 AI Pair Programming Chat",
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
                "Welcome to AI Pair Programming Chat! 👋",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::ITALIC),
            ),
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
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("Mentions: "),
            Span::styled("@model", Style::default().fg(Color::Magenta)),
            Span::raw(" | "),
            Span::styled("@provider", Style::default().fg(Color::Magenta)),
            Span::raw(" | "),
            Span::styled("@history", Style::default().fg(Color::Magenta)),
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
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&msg.content),
            ]));
            lines.push(Line::from(""));
        }

        // 显示流式响应（如果正在进行）
        if app.is_streaming {
            lines.push(Line::from(vec![
                Span::styled(
                    "🤖 AI: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ⏳", app.streaming_response.blocking_lock().get_content()),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }
    }

    let history = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 💬 Chat History ")
                .title_alignment(Alignment::Left),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(history, area);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    // Input hint
    let hint = if app.chat_input.is_empty() {
        "Type your message... (Ctrl+C to exit, /help for commands)"
    } else {
        "Press Enter to send, Backspace to delete"
    };

    let hint_line = Paragraph::new(Line::from(vec![
        Span::styled(
            hint,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ),
    ]));

    f.render_widget(hint_line, input_chunks[0]);

    // Input box
    let input_text = Line::from(vec![
        Span::styled(
            ">>> ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(&app.chat_input),
    ]);

    let input_widget = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(input_widget, input_chunks[1]);

    // Set cursor position
    let cursor_x = input_chunks[1].x + 4 + app.chat_input.len() as u16; // 4 for ">>> "
    let cursor_y = input_chunks[1].y + 1;

    if cursor_x < input_chunks[1].x + input_chunks[1].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }
}
