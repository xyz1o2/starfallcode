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
use std::collections::HashMap;

// ============================================================================
// 数据结构
// ============================================================================

/// 像素头像数据
#[derive(Clone, Debug)]
pub struct PixelData {
    pub color: Color,
    pub map: [u8; 64], // 8x8 = 64 像素（与 examples/v2.html 一致）
}

/// 紧凑头像 + 侧边有色边框（无顶底边），列宽 = 5 像素 + 2 边框 = 7。
fn render_avatar_compact_boxed(avatar_data: &PixelData, border: Color) -> Vec<Line<'static>> {
    let inner = render_avatar_compact(avatar_data);
    let mut out: Vec<Line<'static>> = Vec::with_capacity(inner.len() + 2);
    let b = Style::default().bg(border);
    // 顶部边框（5像素 + 左右各1 = 7 列）
    out.push(Line::from(Span::styled(" ".repeat(7), b)));
    for line in inner.into_iter() {
        let mut spans = Vec::with_capacity(line.spans.len() + 2);
        spans.push(Span::styled(" ", b));
        spans.extend(line.spans);
        spans.push(Span::styled(" ", b));
        out.push(Line::from(spans));
    }
    // 底部边框
    out.push(Line::from(Span::styled(" ".repeat(7), b)));
    out
}

/// 更紧凑的头像：将 8x8 采样为 5x5，然后使用半块字符压缩为 5×3。
fn render_avatar_compact(avatar_data: &PixelData) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let black = Color::Rgb(0, 0, 0);

    // 采样函数：将目标 [0..5) 映射到源 [0..8)
    let sample = |r_t: usize, c_t: usize| -> u8 {
        let sr = (((r_t * 8) + 2) / 5).min(7);
        let sc = (((c_t * 8) + 2) / 5).min(7);
        avatar_data.map[sr * 8 + sc]
    };
    let to_color = |v: u8| match v {
        1 => avatar_data.color,
        2 => Color::White,
        _ => black,
    };

    for tr in (0..5).step_by(2) {
        let mut spans: Vec<Span<'static>> = Vec::new();
        for tc in 0..5 {
            let top = sample(tr, tc);
            let bottom = sample(tr + 1, tc);
            spans.push(Span::styled(
                "▀",
                Style::default().fg(to_color(top)).bg(to_color(bottom)),
            ));
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// 使用半块字符将 8x8 像素压缩为 8x4 行。
/// 每个单元用 '▀'，fg 表示上半像素，bg 表示下半像素。
fn render_avatar_halfblock(avatar_data: &PixelData) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let black = Color::Rgb(0, 0, 0);
    for row in (0..8).step_by(2) {
        let mut spans: Vec<Span<'static>> = Vec::new();
        for col in 0..8 {
            let top = avatar_data.map[row * 8 + col];
            let bottom = avatar_data.map[(row + 1) * 8 + col];

            let to_color = |v: u8| match v {
                1 => avatar_data.color,
                2 => Color::White,
                _ => black,
            };
            let fg = to_color(top);
            let bg = to_color(bottom);
            spans.push(Span::styled("▀", Style::default().fg(fg).bg(bg)));
        }
        lines.push(Line::from(spans));
    }
    lines
}

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
            map: [
                0,0,1,1,1,1,0,0,
                0,1,1,1,1,1,1,0,
                1,1,2,1,1,2,1,1,
                1,1,1,1,1,1,1,1,
                1,0,1,1,1,1,0,1,
                1,0,0,0,0,0,0,1,
                0,1,1,0,0,1,1,0,
                0,0,1,1,1,1,0,0,
            ],
        },
    );

    // 用户头像 (Pink) - 8x8
    avatars.insert(
        "user".to_string(),
        PixelData {
            color: Color::Rgb(244, 114, 182),
            map: [
                0,0,1,1,1,1,0,0,
                0,1,1,1,1,1,1,0,
                1,1,2,1,1,2,1,1,
                1,1,1,1,1,1,1,1,
                1,1,1,0,0,1,1,1,
                0,1,1,1,1,1,1,0,
                0,0,1,0,0,1,0,0,
                0,0,1,1,1,1,0,0,
            ],
        },
    );

    // AI头像 (Cyan) - 8x8（使用 pac 造型，但上色为 accent_ai）
    avatars.insert(
        "ai".to_string(),
        PixelData {
            color: Color::Rgb(34, 211, 238),
            map: [
                0,0,1,1,1,1,0,0,
                0,1,1,1,1,1,1,0,
                1,1,1,1,1,1,1,1,
                1,1,1,1,1,0,0,0,
                1,1,1,1,0,0,0,0,
                1,1,1,1,1,0,0,0,
                0,1,1,1,1,1,1,0,
                0,0,1,1,1,1,0,0,
            ],
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
            Constraint::Length(5),   // 输入区（紧凑头像）
        ])
        .split(size);

    render_history_with_avatars(f, app, chunks[0], &theme);
    render_status_bar(f, chunks[1], &theme);
    render_input_area(f, app, chunks[2], &theme);
}

/// 渲染头像盒子（8x8 内部 + 1 字符边框，宽 10，高 10）。
fn render_avatar_box(avatar_data: &PixelData, border: Color) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let border_style = Style::default().bg(border);
    let black = Color::Rgb(0, 0, 0);

    // 顶部边框（10 列：左1 + 内8 + 右1）
    lines.push(Line::from(vec![Span::styled(" ".repeat(10), border_style)]));

    // 内部 8 行
    for row in 0..8 {
        let mut spans: Vec<Span<'static>> = Vec::new();
        // 左边框
        spans.push(Span::styled(" ", border_style));

        for col in 0..8 {
            let idx = row * 8 + col;
            let pixel = avatar_data.map[idx];

            let style = match pixel {
                0 => Style::default().bg(black), // 透明像素渲染为盒子内部黑色
                1 => Style::default().bg(avatar_data.color),
                2 => Style::default().bg(Color::White),
                _ => Style::default().bg(black),
            };
            // 单空格像素（更紧凑）
            spans.push(Span::styled(" ", style));
        }
        // 右边框
        spans.push(Span::styled(" ", border_style));
        lines.push(Line::from(spans));
    }

    // 底部边框
    lines.push(Line::from(vec![Span::styled(" ".repeat(10), border_style)]));

    lines
}

/// 渲染历史区域（带头像）
fn render_history_with_avatars(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    use crate::core::message::Role as AppRole;

    let messages = app.chat_history.get_messages();
    let mut y_offset = 0u16;

    for msg in messages {
        if y_offset >= area.height {
            break;
        }

        let (role_label, role_color) = match msg.role {
            AppRole::User => ("USER", theme.accent_user),
            AppRole::Assistant => ("AI", theme.accent_ai),
            AppRole::System => ("SYSTEM", Color::Yellow),
        };

        // 头像像素图（与 v2.html 一致）
        let (avatar_map, pixel_color): ([u8; 64], Color) = match msg.role {
            AppRole::User => ([
                0,0,1,1,1,1,0,0,
                0,1,1,1,1,1,1,0,
                1,1,2,1,1,2,1,1,
                1,1,1,1,1,1,1,1,
                1,1,1,0,0,1,1,1,
                0,1,1,1,1,1,1,0,
                0,0,1,0,0,1,0,0,
                0,0,1,1,1,1,0,0,
            ], theme.accent_user),
            AppRole::Assistant => ([
                0,0,1,1,1,1,0,0,
                0,1,1,1,1,1,1,0,
                1,1,1,1,1,1,1,1,
                1,1,1,1,1,0,0,0,
                1,1,1,1,0,0,0,0,
                1,1,1,1,1,0,0,0,
                0,1,1,1,1,1,1,0,
                0,0,1,1,1,1,0,0,
            ], Color::Rgb(250, 204, 21)), // pac 黄
            AppRole::System => ([
                0,0,1,1,1,1,0,0,
                0,1,1,1,1,1,1,0,
                1,1,2,1,1,2,1,1,
                1,1,1,1,1,1,1,1,
                1,0,1,1,1,1,0,1,
                1,0,0,0,0,0,0,1,
                0,1,1,0,0,1,1,0,
                0,0,1,1,1,1,0,0,
            ], theme.accent_ai),
        };
        let avatar_data = PixelData { color: pixel_color, map: avatar_map };

        // 渲染内容：角色标签单独一行（匹配 v2.html，添加 $ 前缀）
        let mut content_lines: Vec<Line> = Vec::new();
        content_lines.push(Line::from(vec![
            Span::styled("$", Style::default().fg(Color::Rgb(136, 136, 136))),
            Span::raw(" "),
            Span::styled(
                role_label,
                Style::default()
                    .fg(role_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        for line in msg.content.lines() {
            content_lines.push(Line::from(line));
        }

        // 重新计算消息高度：紧凑盒子头像 5 行 与 内容行数取最大
        let msg_height = 5u16.max(content_lines.len() as u16);
        // 更新内容区域高度（通过重建 msg_area/h_layout）
        let msg_area = Rect {
            x: area.x,
            y: area.y + y_offset,
            width: area.width,
            height: msg_height.min(area.height.saturating_sub(y_offset)),
        };
        let h_layout = Layout::default()
            .direction(Direction::Horizontal)
            // 头像列 8：5 像素 + 2 边框 + 1 间隙
            .constraints([Constraint::Length(8), Constraint::Min(10)])
            .split(msg_area);

        // 渲染头像（紧凑 + 角色主题边框，贴近 v2.html）
        let border_color = match msg.role {
            AppRole::User => theme.accent_user,
            AppRole::Assistant => theme.accent_ai,
            AppRole::System => theme.accent_ai,
        };
        let avatar_lines = render_avatar_compact_boxed(&avatar_data, border_color);
        f.render_widget(Paragraph::new(avatar_lines), h_layout[0]);

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
            Constraint::Length(8),   // 紧凑头像（6列 + 2 间隙）
            Constraint::Length(2),   // 箭头
            Constraint::Min(10),     // 输入框
        ])
        .split(area);

    // 1. 渲染用户头像（8x8）
    let user_avatar = PixelData {
        color: theme.accent_user,
        map: [
            0,0,1,1,1,1,0,0,
            0,1,1,1,1,1,1,0,
            1,1,2,1,1,2,1,1,
            1,1,1,1,1,1,1,1,
            1,1,1,0,0,1,1,1,
            0,1,1,1,1,1,1,0,
            0,0,1,0,0,1,0,0,
            0,0,1,1,1,1,0,0,
        ],
    };

    let avatar_lines = render_avatar_compact_boxed(&user_avatar, theme.accent_user);
    f.render_widget(Paragraph::new(avatar_lines), chunks[0]);

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
