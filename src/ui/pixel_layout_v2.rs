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
        
        // 消息间空行
        all_lines.push(Line::from(""));
        line_to_msg_map.push(msg_idx);
    }
    
    // 计算滚动偏移量
    // chat_scroll_offset = 0 表示显示最新消息(底部对齐)
    // chat_scroll_offset > 0 表示向上滚动查看历史消息
    let total_lines = all_lines.len() as u16;
    let visible_lines = area.height;
    
    let scroll_offset = if total_lines <= visible_lines {
        // 内容少于可见区域,不需要滚动
        0
    } else {
        // 计算从顶部开始的滚动偏移
        // 当 chat_scroll_offset = 0 时,显示底部(最新消息)
        // scroll_offset = total_lines - visible_lines
        // 当向上滚动时,减少 scroll_offset
        total_lines
            .saturating_sub(visible_lines)
            .saturating_sub(app.chat_scroll_offset as u16)
    };
    
    // 使用 Paragraph 的 scroll 方法渲染
    let paragraph = Paragraph::new(all_lines)
        .wrap(Wrap { trim: true })
        .scroll((scroll_offset, 0));
    
    f.render_widget(paragraph, area);
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
    
    // 水平分割:箭头 | 输入框
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),   // 箭头
            Constraint::Min(10),     // 输入框
        ])
        .split(area);

    // 1. 渲染箭头
    let arrow = "▶";
    f.render_widget(
        Paragraph::new(arrow).style(
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD),
        ),
        chunks[0],
    );

    // 2. 渲染输入框
    let input_widget = Paragraph::new(app.input_text.as_str()).style(Style::default().fg(Color::White));
    f.render_widget(input_widget, chunks[1]);

    // 3. 计算并设置光标位置
    // 光标应该在输入文本的当前光标位置
    let cursor_col = app.input_cursor as u16;
    
    // 设置光标位置 (x = 输入区域起始 + 光标偏移, y = 输入区域起始)
    f.set_cursor(
        chunks[1].x + cursor_col,
        chunks[1].y,
    );
}
