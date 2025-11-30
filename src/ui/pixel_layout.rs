/// 像素艺术风格的 TUI 布局 - 参考 v2.html 设计
/// 
/// 特点：
/// - 8x8 像素化头像
/// - 聊天历史区域
/// - 代码块显示（带 diff 颜色）
/// - 底部输入框（带头像和脉冲箭头）
/// - 状态栏

pub struct THEME {
    pub bg_color: Color,
    pub panel_bg: Color,
    pub border: Color,
    pub accent_ai: Color,
    pub accent_user: Color,
    pub code_bg: Color,
    pub diff_add: Color,
    pub diff_add_text: Color,
    pub diff_rem: Color,
    pub diff_rem_text: Color,
}

pub const V2_THEME: THEME = THEME {
    bg_color: Color::Rgb(12, 12, 12),       // #0c0c0c
    panel_bg: Color::Rgb(17, 17, 17),     // #111
    border: Color::Rgb(51, 51, 51),       // #333
    accent_ai: Color::Rgb(34, 211, 238),  // #22d3ee
    accent_user: Color::Rgb(244, 114, 182), // #f472b6
    code_bg: Color::Rgb(0, 0, 0),           // #000
    diff_add: Color::Rgb(15, 57, 28),     // #0f391c
    diff_add_text: Color::Rgb(74, 222, 128), // #4ade80
    diff_rem: Color::Rgb(63, 19, 19),     // #3f1313
    diff_rem_text: Color::Rgb(248, 113, 113), // #f87171
};

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::app::App;

/// 像素化头像数据
pub struct PixelAvatar {
    pub map: &'static [u8],
}

impl PixelAvatar {
    pub fn sys() -> Self {
        Self { map: &[0,0,1,1,1,1,0,0, 0,1,1,1,1,1,1,0, 1,1,2,1,1,2,1,1, 1,1,1,1,1,1,1,1, 1,0,1,1,1,1,0,1, 1,0,0,0,0,0,0,1, 0,1,1,0,0,1,1,0, 0,0,1,1,1,1,0,0] }
    }
    pub fn user() -> Self {
        Self { map: &[0,0,1,1,1,1,0,0, 0,1,1,1,1,1,1,0, 1,1,2,1,1,2,1,1, 1,1,1,1,1,1,1,1, 1,1,1,0,0,1,1,1, 0,1,1,1,1,1,1,0, 0,0,1,0,0,1,0,0, 0,0,1,1,1,1,0,0] }
    }
    pub fn ai() -> Self {
        Self { map: &[0,0,1,1,1,1,0,0, 0,1,1,1,1,1,1,0, 1,1,1,1,1,1,1,1, 1,1,1,1,1,0,0,0, 1,1,1,1,0,0,0,0, 1,1,1,1,1,0,0,0, 0,1,1,1,1,1,1,0, 0,0,1,1,1,1,0,0] }
    }
    pub fn error() -> Self {
        Self { map: &[0,0,1,1,1,1,0,0, 0,1,1,1,1,1,1,0, 1,1,2,1,1,2,1,1, 1,1,1,1,1,1,1,1, 1,1,0,1,1,0,1,1, 0,1,1,1,1,1,1,0, 0,1,0,1,1,0,1,0, 0,0,0,0,0,0,0,0] }
    }

    /// 渲染头像为文本（8x8 网格）
    pub fn render_text(&self) -> Vec<String> {
        let mut lines = vec![];
        for row in 0..8 {
            let mut line = String::new();
            for col in 0..8 {
                let idx = row * 8 + col;
                let pixel = self.map[idx];
                match pixel {
                    0 => line.push(' '),
                    1 => line.push('█'),
                    2 => line.push('●'),
                    _ => line.push(' '),
                }
            }
            lines.push(line);
        }
        lines
    }

    /// 渲染为 Ratatui Spans（带背景色填充）
    pub fn render_lines(&self, color: Color) -> Vec<Line<'static>> {
        let mut lines = vec![];
        for row in 0..8 {
            let mut spans = vec![];
            for col in 0..8 {
                let idx = row * 8 + col;
                let pixel = self.map[idx];
                let pixel_style = match pixel {
                    0 => Style::default(),
                    1 => Style::default().bg(color),
                    2 => Style::default().bg(Color::White),
                    _ => Style::default(),
                };
                spans.push(Span::styled("  ", pixel_style));
            }
            lines.push(Line::from(spans));
        }
        lines
    }
}

/// 主布局渲染函数
pub fn render_pixel_layout(f: &mut Frame, app: &App) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let theme = &V2_THEME;
    let size = f.size();

    // 设置整个背景色
    f.render_widget(Block::default().bg(theme.bg_color), size);

    // 新布局: History | Status | Input
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),     // History area
            Constraint::Length(1),  // Status bar
            Constraint::Length(4),  // Input area
        ])
        .split(size);

    // 1. 聊天历史区域
    render_history_area(f, app, chunks[0]);

    // 2. 状态栏
    render_status_bar(f, app, chunks[1]);

    // 3. 输入区域
    render_input_area(f, app, chunks[2]);
}

/// 渲染状态栏
fn render_status_bar(f: &mut Frame, _app: &App, area: Rect) {
    let theme = &V2_THEME;
    let status_text = "STATUS: CONNECTED";
    let help_text = "CTRL+C to EXIT";

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme.border))
        .bg(Color::Rgb(34, 34, 34)); // #222 from CSS

    let status_line = Line::from(vec![
        Span::styled(status_text, Style::default().fg(Color::Rgb(119, 119, 119))),
        Span::raw(" ".repeat((area.width as usize).saturating_sub(status_text.len() + help_text.len()))),
        Span::styled(help_text, Style::default().fg(Color::Rgb(119, 119, 119))),
    ]);

    let status = Paragraph::new(status_line)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(status, area);
}

/// 渲染聊天历史区域 - 使用水平布局实现两列
fn render_history_area(f: &mut Frame, app: &App, area: Rect) {
    use crate::core::message::Role;
    use ratatui::layout::{Constraint, Direction, Layout};

    let theme = &V2_THEME;
    let messages = app.chat_history.get_messages();

    // 计算每条消息需要的高度并渲染
    let mut current_y = app.chat_scroll_offset as u16;
    
    for msg in messages {
        if current_y >= area.height {
            break;
        }

        let (avatar, role_label, role_color) = match msg.role {
            Role::User => (PixelAvatar::user(), "USER", theme.accent_user),
            Role::Assistant => (PixelAvatar::ai(), "AI", theme.accent_ai),
            Role::System => (PixelAvatar::sys(), "SYSTEM", Color::Yellow),
        };

        let avatar_lines = avatar.render_lines(role_color);
        let avatar_height = avatar_lines.len() as u16;

        // 创建消息区域
        let msg_area = Rect {
            x: area.x,
            y: area.y + current_y,
            width: area.width,
            height: (avatar_height + 1).min(area.height - current_y),
        };

        // 水平分割：头像列 + 内容列
        let h_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(16), Constraint::Min(20)])
            .split(msg_area);

        let avatar_area = h_layout[0];
        let content_area = h_layout[1];

        // 渲染头像（8 行）
        let avatar_para = Paragraph::new(avatar_lines);
        f.render_widget(avatar_para, avatar_area);

        // 渲染内容（角色标签 + 消息）
        let mut content_lines = vec![Line::from(vec![Span::styled(
            role_label,
            Style::default()
                .fg(role_color)
                .add_modifier(Modifier::BOLD),
        )])];

        for line in msg.content.lines() {
            content_lines.push(Line::from(line));
        }

        let content_para = Paragraph::new(content_lines)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::Rgb(220, 220, 220)));

        f.render_widget(content_para, content_area);

        current_y += avatar_height + 1;
    }
}

/// 渲染输入区域（HUD 风格）
fn render_input_area(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::text::Text;
    let theme = &V2_THEME;

    // 整个输入区域的背景
    f.render_widget(Block::default().bg(Color::Rgb(8, 8, 8)), area);

    // 布局：头像 | 输入包装器
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
            Constraint::Length(10), // 8x8 avatar + padding
            Constraint::Min(10),
        ])
        .split(area);

    // 1. 渲染用户头像
    let avatar = PixelAvatar::user();
    let avatar_lines = avatar.render_lines(theme.accent_user);
    let avatar_widget = Paragraph::new(Text::from(avatar_lines))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_user)));
    f.render_widget(avatar_widget, chunks[0]);

    // 2. 渲染输入框
    let pulse_char = if (app.frame_count / 15) % 2 == 0 { "▶" } else { "▸" };
    
    // 计算输入框可用宽度（需要留出脉冲字符和空格的空间）
    let available_width = chunks[1].width.saturating_sub(2) as usize;
    
    let input_text = &app.input_text;
    let char_count = input_text.chars().count();
    
    // 计算显示的文本（支持水平滚动）
    // 当文本超过宽度时，从末尾向前显示，保证光标始终可见
    let (display_text, display_start_pos) = if char_count > available_width {
        // 显示最后 available_width 个字符
        let start_pos = char_count.saturating_sub(available_width);
        let text = input_text.chars().skip(start_pos).collect::<String>();
        (text, start_pos)
    } else {
        (input_text.to_string(), 0)
    };
    
    let input_line = Line::from(vec![
        Span::styled(
            pulse_char,
            Style::default().fg(theme.accent_user).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::raw(display_text.clone()),
    ]);

    let input_widget = Paragraph::new(input_line)
        .style(Style::default().fg(Color::White));

    f.render_widget(input_widget, chunks[1]);

    // 3. 设置光标位置
    // 光标始终在文本末尾（最后一个字符后面）
    // 光标x坐标 = 脉冲字符(1) + 空格(1) + 显示文本的长度
    let display_char_count = display_text.chars().count() as u16;
    let cursor_x = chunks[1].x + 2 + display_char_count;
    let cursor_y = chunks[1].y + (chunks[1].height / 2);
    
    // 确保光标在有效范围内
    if cursor_x <= chunks[1].right() && cursor_y < chunks[1].bottom() {
        f.set_cursor(cursor_x, cursor_y);
    }
}
