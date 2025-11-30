/// 🎮 Ratatui 像素CLI聊天界面 - 按指南完整实现
/// 参考: RATATUI_V2_IMPLEMENTATION_GUIDE.md

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use crate::app::App;
use crate::core::message::Role as AppRole;
use crate::ui::avatar::PixelData;
use crate::ui::svg_avatar;
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

    // 垂直分割：历史 | 状态栏 | 输入
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),      // 历史区
            Constraint::Length(1),   // 状态栏
            Constraint::Length(4),   // 输入区（缩小为 4行）
        ])
        .split(size);

    render_history_with_avatars(f, app, chunks[0], &theme);
    render_status_bar(f, chunks[1], &theme);
    render_input_area(f, app, chunks[2], &theme);
}


/// 渲染历史区域（带头像）
fn render_history_with_avatars(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let messages = app.chat_history.get_messages();
    let mut y_offset = 0u16;

    for msg in messages {
        if y_offset >= area.height {
            break;
        }

        let (_role_label, _role_color) = match msg.role {
            AppRole::User => ("USER", theme.accent_user),
            AppRole::Assistant => ("AI", theme.accent_ai),
            AppRole::System => ("SYSTEM", Color::Yellow),
        };


        // 渲染内容：直接显示消息内容，不包含角色标签
        let mut content_lines: Vec<Line> = Vec::new();
        for line in msg.content.lines() {
            content_lines.push(Line::from(line));
        }

        // 计算消息高度：取头像高度(4行)和内容行数的最大值
        let avatar_height = 4u16;
        let content_height = content_lines.len() as u16;
        let msg_height = avatar_height.max(content_height);
        // 更新内容区域高度（通过重建 msg_area/h_layout）
        let msg_area = Rect {
            x: area.x,
            y: area.y + y_offset,
            width: area.width,
            height: msg_height.min(area.height.saturating_sub(y_offset)),
        };
        let h_layout = Layout::default()
            .direction(Direction::Horizontal)
            // 头像列：4像素 × 2空格/像素 = 8 字符宽 + 2 列间隙
            .constraints([Constraint::Length(10), Constraint::Min(10)])
            .split(msg_area);

        // 使用 Canvas Widget 渲染头像
        let avatar_widget = svg_avatar::get_avatar_widget(&msg.role);
        f.render_widget(avatar_widget, h_layout[0]);

        let content_para = Paragraph::new(content_lines).wrap(Wrap { trim: true });
        f.render_widget(content_para, h_layout[1]);

        y_offset = y_offset.saturating_add(msg_height + 2); // +2 留白更接近 v2.html
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

/// 渲染输入区域
fn render_input_area(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    // 背景
    f.render_widget(Paragraph::new("").style(Style::default().bg(Color::Rgb(8, 8, 8))), area);
    // 水平分割：头像 | 箭头 | 输入框
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10),  // 4像素 × 2空格 = 8字符 + 2间隙
            Constraint::Length(2),   // 箭头
            Constraint::Min(10),     // 输入框
        ])
        .split(area);
    // 使用 Canvas Widget 渲染用户头像
    let avatar_widget = svg_avatar::get_avatar_widget(&AppRole::User);
    f.render_widget(avatar_widget, chunks[0]);

    // 2. 渲染箭头
    let arrow = "▶";
    f.render_widget(
        Paragraph::new(arrow).style(
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD),
        ),
        chunks[1],
    );

    // 3. 渲染输入框（空时显示 placeholder）
    let input_widget = if app.input_text.is_empty() {
        let placeholder = Line::from(Span::styled(
            "Type 'add', 'del', 'fix' or chat...",
            Style::default().fg(Color::Rgb(120, 120, 120)),
        ));
        Paragraph::new(vec![placeholder]).style(Style::default().fg(Color::White))
    } else {
        Paragraph::new(app.input_text.as_str()).style(Style::default().fg(Color::White))
    };
    f.render_widget(input_widget, chunks[2]);

    // 4. 显示光标（使用字符数而不是字节数）
    let cursor_pos = app.input_text.chars().count() as u16;
    f.set_cursor(
        chunks[2].x + cursor_pos,
        chunks[2].y,
    );
}
