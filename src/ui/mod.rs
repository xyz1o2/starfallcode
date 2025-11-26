pub mod layout;
pub mod sidebar;
pub mod info_panel;
pub mod theme;
pub mod focus;
pub mod types;
pub mod command_hints;
pub mod mention_suggestions;
pub mod file_search;

use crate::ui::theme::ModernTheme;
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

/// 渲染 Diff 对比
fn render_diff_lines(lines: &mut Vec<Line>, old_content: &str, new_content: &str, _area_width: u16) {
    lines.push(Line::from(vec![
        Span::styled(
            "  ┌─ Diff 对比",
            Style::default().fg(Color::Magenta),
        ),
    ]));

    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();
    let max_lines = old_lines.len().max(new_lines.len());

    for i in 0..max_lines {
        if i < old_lines.len() {
            lines.push(Line::from(vec![
                Span::styled(
                    "  │ - ",
                    Style::default().fg(Color::Red),
                ),
                Span::styled(
                    old_lines[i].to_string(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::DIM),
                ),
            ]));
        }
        if i < new_lines.len() {
            lines.push(Line::from(vec![
                Span::styled(
                    "  │ + ",
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    new_lines[i].to_string(),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
    }

    lines.push(Line::from(vec![
        Span::styled(
            "  └─",
            Style::default().fg(Color::Magenta),
        ),
    ]));
}

pub fn render_history(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::new();

    if app.chat_history.is_empty() && !app.is_streaming {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "✨ Welcome to Starfellcode Pair Programming",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from("💡 Tips:"));
        lines.push(Line::from("  • Type / to see available commands"));
        lines.push(Line::from("  • Use @file to mention files"));
        lines.push(Line::from("  • Enable YOLO mode for quick operations"));
        lines.push(Line::from(""));
    } else {
        let messages = app.chat_history.get_messages();
        let start_idx = if app.chat_scroll_offset < messages.len() {
            messages.len() - app.chat_scroll_offset - 1
        } else {
            0
        };
        
        for msg in messages.iter().skip(start_idx) {
            let (prefix, color) = match msg.role {
                crate::core::message::Role::User => ("👤 You", Color::Blue),
                crate::core::message::Role::Assistant => ("🤖 AI", Color::Green),
                crate::core::message::Role::System => ("⚙️ System", Color::Yellow),
            };

            // 消息头部 - 使用简单的分隔线
            lines.push(Line::from(vec![
                Span::styled(
                    format!("▶ {}", prefix),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " ".to_string() + &"─".repeat((area.width as usize).saturating_sub(prefix.len() + 4)),
                    Style::default().fg(color),
                ),
            ]));

            // 检测 Diff 对比标记
            if msg.content.contains("📝 显示修改对比") && msg.content.contains("---") {
                // 这是一个 Diff 消息，提取旧内容和新内容
                let parts: Vec<&str> = msg.content.split("+++").collect();
                if parts.len() == 2 {
                    let old_part = parts[0].trim();
                    let new_part = parts[1].trim();
                    
                    // 显示提示信息
                    lines.push(Line::from(vec![
                        Span::styled(
                            "  📝 显示修改对比",
                            Style::default().fg(Color::Yellow),
                        ),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled(
                            "  (输入 /confirm-modify 确认或 /cancel-modify 取消)",
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                    lines.push(Line::from(""));
                    
                    render_diff_lines(&mut lines, old_part, new_part, area.width);
                } else {
                    // 普通文本行
                    lines.push(Line::from(vec![
                        Span::styled(
                            "  ",
                            Style::default().fg(color),
                        ),
                        Span::raw(&msg.content),
                    ]));
                }
            } else {
                // 消息内容 - 支持代码块检测
                let mut in_code_block = false;
                
                for content_line in msg.content.lines() {
                    // 检测代码块开始
                    if content_line.trim_start().starts_with("```") {
                        if !in_code_block {
                            in_code_block = true;
                            let code_lang = content_line.trim_start()[3..].to_string();
                            // 代码块开始标记
                            lines.push(Line::from(vec![
                                Span::styled(
                                    "  ┌─ Code",
                                    Style::default().fg(Color::Magenta),
                                ),
                                Span::styled(
                                    format!(" ({})", if code_lang.is_empty() { "text" } else { &code_lang }),
                                    Style::default().fg(Color::Magenta).add_modifier(Modifier::DIM),
                                ),
                            ]));
                        } else {
                            in_code_block = false;
                            // 代码块结束标记
                            lines.push(Line::from(vec![
                                Span::styled(
                                    "  └─",
                                    Style::default().fg(Color::Magenta),
                                ),
                            ]));
                        }
                    } else if in_code_block {
                        // 代码行 - 使用不同的颜色
                        lines.push(Line::from(vec![
                            Span::styled(
                                "  │ ",
                                Style::default().fg(Color::Magenta),
                            ),
                            Span::styled(
                                content_line.to_string(),
                                Style::default().fg(Color::Yellow),
                            ),
                        ]));
                    } else {
                        // 普通文本行
                        lines.push(Line::from(vec![
                            Span::styled(
                                "  ",
                                Style::default().fg(color),
                            ),
                            Span::raw(content_line),
                        ]));
                    }
                }
            }

            // 消息底部 - 简单分隔
            lines.push(Line::from(vec![
                Span::styled(
                    "─".repeat(area.width as usize),
                    Style::default().fg(color),
                ),
            ]));
            lines.push(Line::from(""));
        }

        if app.is_streaming {
            let streaming_content = app.streaming_response.try_lock()
                .map(|resp| resp.content.clone())
                .unwrap_or_default();
            
            lines.push(Line::from(vec![
                Span::styled(
                    "▶ 🤖 AI",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " ".to_string() + &"─".repeat((area.width as usize).saturating_sub(10)),
                    Style::default().fg(Color::Green),
                ),
            ]));

            for content_line in streaming_content.lines() {
                let line_str = content_line.to_string();
                lines.push(Line::from(vec![
                    Span::styled(
                        "  ",
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(
                        line_str,
                        Style::default().fg(Color::Cyan),
                    ),
                ]));
            }

            lines.push(Line::from(vec![
                Span::styled(
                    "  ",
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    "⏳ Streaming...",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
                ),
            ]));
        }
    }

    // 计算需要的行数
    let total_lines = lines.len() as u16;
    let available_height = area.height.saturating_sub(2); // 减去边框
    
    // 如果内容超过可用高度，计算滚动偏移
    let scroll_offset = if total_lines > available_height {
        (total_lines - available_height) as usize
    } else {
        0
    };

    let history = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((scroll_offset as u16, 0))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" 💬 Chat History ")
            .style(Style::default().fg(Color::DarkGray)));

    f.render_widget(history, area);
}

pub fn render_input(f: &mut Frame, app: &mut App, area: Rect) {
    // 动态决定是否显示提示区
    let show_hints = app.mention_suggestions.visible || app.command_hints.visible;
    
    let (input_area, hints_area) = if show_hints {
        // 如果需要显示提示，分割区域
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),  // 输入框（包括边框和提示行）
                Constraint::Min(5),     // 提示区最小 5 行
            ])
            .split(area);
        (chunks[0], chunks[1])
    } else {
        // 否则只显示输入框（固定 4 行）
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),  // 输入框固定 4 行
            ])
            .split(area);
        (chunks[0], Rect::default())
    };

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

    // 在 hints_area 中渲染命令提示或提及建议
    if hints_area.height > 0 && hints_area.width > 0 {
        if app.mention_suggestions.visible {
            // 清除背景
            f.render_widget(ratatui::widgets::Clear, hints_area);
            // 需要可变引用来更新 ListState
            let mention_suggestions = &mut app.mention_suggestions;
            mention_suggestions.render(f, hints_area);
        } else if app.command_hints.visible {
            // 清除背景
            f.render_widget(ratatui::widgets::Clear, hints_area);
            app.command_hints.render(f, hints_area, &ModernTheme::dark_professional());
        }
    }
}

/// 渲染独立的确认对话层（不在聊天历史中）
pub fn render_confirmation_dialog(f: &mut Frame, app: &App, area: Rect) {
    if !app.modification_confirmation_pending || app.pending_modifications.is_empty() {
        return;
    }

    let (op, _diff_opt) = &app.pending_modifications[app.modification_selected_index];
    
    // 获取操作信息
    let (op_name, path) = match op {
        crate::ai::code_modification::CodeModificationOp::Create { path, .. } => ("创建", path.clone()),
        crate::ai::code_modification::CodeModificationOp::Modify { path, .. } => ("修改", path.clone()),
        crate::ai::code_modification::CodeModificationOp::Delete { path } => ("删除", path.clone()),
    };

    let mut lines = vec![];
    
    // 标题
    lines.push(Line::from(vec![
        Span::styled(
            format!("⏳ {}文件", op_name),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
    ]));
    
    // 文件路径
    lines.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(path, Style::default().fg(Color::Cyan)),
    ]));
    
    lines.push(Line::from(""));
    
    // 选项列表 - 根据选择状态动态改变颜色
    let confirm_style = if app.modification_choice == crate::app::ModificationChoice::Confirm {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    
    let cancel_style = if app.modification_choice == crate::app::ModificationChoice::Cancel {
        Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };
    
    let abandon_style = if app.modification_choice == crate::app::ModificationChoice::Abandon {
        Style::default().fg(Color::Black).bg(Color::Gray).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    
    // 确认选项
    if app.modification_choice == crate::app::ModificationChoice::Confirm {
        lines.push(Line::from(vec![
            Span::styled("● ", confirm_style),
            Span::styled("1. 确认 (Y)", confirm_style),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("1. 确认 (Y)", Style::default().fg(Color::Green)),
        ]));
    }
    
    // 取消选项
    if app.modification_choice == crate::app::ModificationChoice::Cancel {
        lines.push(Line::from(vec![
            Span::styled("● ", cancel_style),
            Span::styled("2. 取消 (N)", cancel_style),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("2. 取消 (N)", Style::default().fg(Color::Red)),
        ]));
    }
    
    // 放弃选项
    if app.modification_choice == crate::app::ModificationChoice::Abandon {
        lines.push(Line::from(vec![
            Span::styled("● ", abandon_style),
            Span::styled("3. 放弃 (Esc)", abandon_style),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("3. 放弃 (Esc)", Style::default().fg(Color::Gray)),
        ]));
    }

    // 创建对话框
    let dialog = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(Style::default().fg(Color::Cyan)))
        .wrap(Wrap { trim: true });

    f.render_widget(dialog, area);
}