/// 🎮 Ratatui 像素CLI聊天界面 - 按指南完整实现
/// 参考: RATATUI_V2_IMPLEMENTATION_GUIDE.md

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget, Wrap},
    Frame,
};
use crate::app::App;
use crate::core::message::Role as AppRole;
use crate::ui::avatar::PixelData;
use crate::ui::input_area::render_input_area;
use std::collections::HashMap;

// ============================================================================
// 数据结构
// ============================================================================

// PixelData 与 8x8 渲染已移动到 `ui::avatar` 模块


/// 消息角色
#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

/// 代码行状态
#[derive(Clone, Debug, PartialEq)]
pub enum LineStatus {
    Added,
    Removed,
    Normal,
}

/// 代码行
#[derive(Clone, Debug)]
pub struct CodeLine {
    pub number: usize,
    pub content: String,
    pub status: LineStatus,
}

/// 代码块
#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub language: String,
    pub lines: Vec<CodeLine>,
}

/// 消息
#[derive(Clone, Debug)]
pub struct Message {
    pub role: Role,
    pub avatar_key: String,
    pub content: String,
    pub code_block: Option<CodeBlock>,
}

// ============================================================================
// 颜色主题
// ============================================================================

pub struct Theme {
    pub bg: Color,
    pub panel_bg: Color,
    pub border: Color,
    pub accent_ai: Color,
    pub accent_user: Color,
    pub diff_add: Color,
    pub diff_add_text: Color,
    pub diff_rem: Color,
    pub diff_rem_text: Color,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            bg: Color::Rgb(12, 12, 12),           // #0c0c0c
            panel_bg: Color::Rgb(17, 17, 17),    // #111
            border: Color::Rgb(51, 51, 51),      // #333
            accent_ai: Color::Rgb(34, 211, 238), // #22d3ee
            accent_user: Color::Rgb(244, 114, 182), // #f472b6
            diff_add: Color::Rgb(15, 57, 28),    // #0f391c
            diff_add_text: Color::Rgb(74, 222, 128), // #4ade80
            diff_rem: Color::Rgb(63, 19, 19),    // #3f1313
            diff_rem_text: Color::Rgb(248, 113, 113), // #f87171
        }
    }
}

// ============================================================================
// 头像初始化
// ============================================================================

pub fn init_avatars() -> HashMap<String, PixelData> {
    let mut avatars = HashMap::new();

    // 系统头像 (Cyan) - 8x8
    avatars.insert(
        "sys".to_string(),
        PixelData {
            color: Color::Rgb(34, 211, 238),
            map: vec![
                0,0,1,1,1,1,0,0,
                0,1,1,1,1,1,1,0,
                1,1,2,1,1,2,1,1,
                1,1,1,1,1,1,1,1,
                1,0,1,1,1,1,0,1,
                1,0,0,0,0,0,0,1,
                0,1,1,0,0,1,1,0,
                0,0,1,1,1,1,0,0,
            ],
            width: 8,
            height: 8,
        },
    );

    // 用户头像 (Pink) - 8x8
    avatars.insert(
        "user".to_string(),
        PixelData {
            color: Color::Rgb(244, 114, 182),
            map: vec![
                0,0,1,1,1,1,0,0,
                0,1,1,1,1,1,1,0,
                1,1,2,1,1,2,1,1,
                1,1,1,1,1,1,1,1,
                1,1,1,0,0,1,1,1,
                0,1,1,1,1,1,1,0,
                0,0,1,0,0,1,0,0,
                0,0,1,1,1,1,0,0,
            ],
            width: 8,
            height: 8,
        },
    );

    // AI头像 (Cyan) - 8x8（使用 pac 造型，但上色为 accent_ai）
    avatars.insert(
        "ai".to_string(),
        PixelData {
            color: Color::Rgb(34, 211, 238),
            map: vec![
                0,0,1,1,1,1,0,0,
                0,1,1,1,1,1,1,0,
                1,1,1,1,1,1,1,1,
                1,1,1,1,1,0,0,0,
                1,1,1,1,0,0,0,0,
                1,1,1,1,1,0,0,0,
                0,1,1,1,1,1,1,0,
                0,0,1,1,1,1,0,0,
            ],
            width: 8,
            height: 8,
        },
    );

    avatars
}

// ============================================================================
// 核心渲染函数
// ============================================================================

/// 主布局渲染函数
pub fn render_pixel_layout(f: &mut Frame, app: &App) {
    let theme = Theme::new();
    let size = f.size();

    // 背景
    f.render_widget(Block::default().bg(theme.bg), size);

    // 确保输入区最小为3行，给历史区更多空间
    let input_height = 3;
    let status_height = 1;

    // 垂直分割：历史 | 状态栏 | 输入
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(12),                    // 历史区（增加到至少12行）
            Constraint::Length(status_height),      // 状态栏
            Constraint::Length(input_height),       // 输入区
        ])
        .split(size);

    render_history_with_avatars(f, app, chunks[0], &theme);
    render_status_bar(f, chunks[1], &theme);
    render_input_area(f, app, chunks[2], &theme);
}


/// 渲染历史区域(带头像)
fn render_history_with_avatars(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let messages = app.chat_history.get_messages();

    // 构建所有消息的行内容
    let mut all_lines: Vec<Line> = Vec::new();
    let mut line_to_msg_map: Vec<usize> = Vec::new(); // 记录每行属于哪个消息

    for (msg_idx, msg) in messages.iter().enumerate() {
        let (_role_label, role_color) = match msg.role {
            AppRole::User => ("USER", theme.accent_user),
            AppRole::Assistant => ("AI", theme.accent_ai),
            AppRole::System => ("SYSTEM", Color::Yellow),
        };

        // 添加头像行(使用简化的文本表示)
        let avatar_symbol = match msg.role {
            AppRole::User => "👤 ",
            AppRole::Assistant => "🤖 ",
            AppRole::System => "⚙️  ",
        };

        all_lines.push(Line::from(Span::styled(
            avatar_symbol,
            Style::default().fg(role_color).add_modifier(Modifier::BOLD),
        )));
        line_to_msg_map.push(msg_idx);

        // 添加消息内容
        for line in msg.content.lines() {
            all_lines.push(Line::from(format!("  {}", line)));
            line_to_msg_map.push(msg_idx);
        }

        // 消息间空行（除了最后一条消息）
        if msg_idx < messages.len() - 1 {
            all_lines.push(Line::from(""));
            line_to_msg_map.push(msg_idx);
        }
    }

    // 计算滚动偏移量 - 确保显示底部最新消息
    let total_lines = all_lines.len() as u16;
    let visible_lines = area.height;

    // 当 chat_scroll_offset = 0 时，显示最新消息（底部对齐）
    // scroll_offset 表示从顶部跳过多少行
    let scroll_offset = if total_lines > visible_lines {
        // 内容超过可见区域，计算偏移以显示底部
        total_lines.saturating_sub(visible_lines).saturating_sub(app.chat_scroll_offset as u16)
    } else {
        // 内容少于可见区域，从顶部开始显示
        0
    };

    // 创建带边框的历史区域以容纳滚动条
    let history_block = Block::default()
        .bg(theme.panel_bg);

    // 使用 Paragraph 的 scroll 方法渲染
    let paragraph = Paragraph::new(all_lines.clone())
        .wrap(Wrap { trim: true })
        .scroll((scroll_offset, 0))
        .block(history_block.clone());

    // 渲染历史消息
    f.render_widget(paragraph, area);

    // 添加滚动条
    if total_lines > visible_lines {
        let mut scrollbar_state = ratatui::widgets::ScrollbarState::default()
            .content_length(total_lines as usize)
            .position(scroll_offset as usize);

        ratatui::widgets::Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("█")
            .render(area, f.buffer_mut(), &mut scrollbar_state);
    }
}

/// 渲染历史区域（旧版本，不带头像）
fn render_history(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    use crate::core::message::Role as AppRole;

    let mut lines: Vec<Line> = Vec::new();

    // 获取消息
    let messages = app.chat_history.get_messages();

    for msg in messages {
        // 确定头像和颜色
        let (role_label, role_color) = match msg.role {
            AppRole::User => ("USER", theme.accent_user),
            AppRole::Assistant => ("AI", theme.accent_ai),
            AppRole::System => ("SYSTEM", Color::Yellow),
        };

        // 添加角色标签
        lines.push(Line::from(Span::styled(
            role_label,
            Style::default()
                .fg(role_color)
                .add_modifier(Modifier::BOLD),
        )));

        // 添加消息内容
        for line in msg.content.lines() {
            lines.push(Line::from(line));
        }

        // 消息间隔
        lines.push(Line::from(""));
    }

    // 渲染
    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.chat_scroll_offset as u16, 0));

    f.render_widget(para, area);
}

/// 渲染状态栏
fn render_status_bar(f: &mut Frame, area: Rect, _theme: &Theme) {
    let status_line = Line::from(vec![
        Span::styled(
            "STATUS: CONNECTED",
            Style::default().fg(Color::Rgb(119, 119, 119)),
        ),
        Span::raw(" ".repeat(area.width.saturating_sub(30) as usize)),
        Span::styled(
            "CTRL+C to EXIT",
            Style::default().fg(Color::Rgb(119, 119, 119)),
        ),
    ]);

    let para = Paragraph::new(status_line).style(Style::default().bg(Color::Rgb(34, 34, 34)));

    f.render_widget(para, area);
}


/// ============================================================================
/// 鼠标选择文本功能
/// ============================================================================

use std::cmp::{min, max};

/// Extract selected text from chat area based on mouse coordinates
pub fn extract_text_from_chat_area(
    app: &crate::app::App,
    mouse_col: u16,
    mouse_row: u16,
    terminal_width: u16,
    terminal_height: u16
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut selected_text = String::new();

    // Check if we have a selection area
    if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
        // Convert terminal coordinates to text coordinates
        let min_col = min(start.0, end.0) as usize;
        let max_col = max(start.0, end.0) as usize;
        let min_row = min(start.1, end.1) as usize;
        let max_row = max(start.1, end.1) as usize;

        // Extract text from chat history based on the selection
        for msg in app.chat_history.get_messages() {
            let msg_lines: Vec<&str> = msg.content.lines().collect();

            for (line_idx, line) in msg_lines.iter().enumerate() {
                let line_start_row = line_idx;
                let line_end_row = line_idx;

                // Check if this line is within the vertical selection range
                if line_start_row >= min_row && line_end_row <= max_row {
                    let line_len = line.chars().count();

                    // Check if this line is within the horizontal selection range
                    if min_col < line_len && max_col > 0 {
                        let start_char = min(min_col, line_len);
                        let end_char = min(max_col, line_len);

                        // Extract the selected portion of the line
                        let line_chars: Vec<char> = line.chars().collect();
                        let selected_chars: String = line_chars[start_char..end_char].iter().collect();

                        selected_text.push_str(&selected_chars);
                    }
                }
            }

            // Add separator between messages
            if !selected_text.is_empty() {
                selected_text.push_str("\n\n");
            }
        }
    }

    Ok(selected_text.trim().to_string())
}

/// Render visual feedback for text selection
pub fn render_selection_highlight(
    f: &mut Frame,
    app: &crate::app::App,
    area: Rect,
    theme: &Theme
) {
    if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
        // Calculate selection rectangle
        let min_col = min(start.0, end.0);
        let max_col = max(start.0, end.0);
        let min_row = min(start.1, end.1);
        let max_row = max(start.1, end.1);

        // Only render if selection is within the chat area
        if min_row < area.bottom() && max_row >= area.top() {
            // For simplicity, we'll render a simple highlight effect
            // In a full implementation, you would overlay the selected text with a different style
            let highlight_area = Rect {
                x: area.x + min_col,
                y: area.y + min_row.saturating_sub(area.top()),
                width: max_col - min_col,
                height: max_row - min_row + 1,
            };

            // Render semi-transparent selection overlay
            let selection_block = ratatui::widgets::Block::default()
                .style(ratatui::style::Style::default()
                    .bg(Color::Rgb(50, 50, 100))
                    .add_modifier(Modifier::DIM));

            f.render_widget(selection_block, highlight_area);
        }
    }
}
