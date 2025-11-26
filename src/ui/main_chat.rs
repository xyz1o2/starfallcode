use crate::ui::types::{MessageStatus, ChatAction};
use crate::ui::theme::ModernTheme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct EnhancedChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub status: MessageStatus,
    pub metadata: MessageMetadata,
}

#[derive(Clone, Debug, Default)]
pub struct MessageMetadata {
    pub tokens: Option<usize>,
    pub processing_time_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct TypingIndicator {
    pub message: String,
    pub animation_frame: usize,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct MainChatArea {
    pub messages: Vec<EnhancedChatMessage>,
    pub input_text: String,
    pub cursor_position: usize,
    pub scroll_offset: usize,
    pub max_scroll: usize,
    pub auto_scroll: bool,
    pub typing_indicator: Option<TypingIndicator>,
}

impl MainChatArea {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_text: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            max_scroll: 0,
            auto_scroll: true,
            typing_indicator: None,
        }
    }

    /// Render the main chat area
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &ModernTheme, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(5),
            ])
            .split(area);

        self.render_chat_history(frame, chunks[0], theme);
        self.render_input_area(frame, chunks[1], theme, focused);
    }

    /// Render chat history
    fn render_chat_history(&self, frame: &mut Frame, area: Rect, theme: &ModernTheme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 💬 Chat History ")
            .title_alignment(Alignment::Left)
            .border_style(theme.get_border_style(false));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines = Vec::new();

        if self.messages.is_empty() {
            lines.extend(self.create_welcome_message(theme));
        } else {
            for message in &self.messages {
                lines.extend(self.format_message(message, theme, inner.width));
                lines.push(Line::from(""));
            }
        }

        if let Some(indicator) = &self.typing_indicator {
            lines.extend(self.format_typing_indicator(indicator, theme));
        }

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, inner);
        self.render_scrollbar(frame, area, theme);
    }

    /// Render input area
    fn render_input_area(&self, frame: &mut Frame, area: Rect, theme: &ModernTheme, focused: bool) {
        let border_style = if focused {
            theme.get_border_style(true)
        } else {
            theme.get_border_style(false)
        };

        // Split input area into hint and input box
        let input_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),   // Input hint
                Constraint::Min(1),      // Input box
            ])
            .split(area);

        // Render input hint
        let hint_text = if self.input_text.is_empty() {
            "Type your message... (Enter to send, Ctrl+C to exit, /help for commands)"
        } else {
            "Press Enter to send, Escape to clear"
        };

        let hint = Paragraph::new(Line::from(Span::styled(
            hint_text,
            Style::default().fg(theme.colors.secondary),
        )));
        frame.render_widget(hint, input_chunks[0]);

        // Render input box
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(" 📝 Input ")
            .title_alignment(Alignment::Left)
            .border_style(border_style);

        let input_inner = input_block.inner(input_chunks[1]);
        frame.render_widget(input_block, input_chunks[1]);

        // Render input text with prompt
        let prompt = ">>> ";
        let input_line = Line::from(vec![
            Span::styled(prompt, Style::default().fg(theme.colors.primary)),
            Span::styled(&self.input_text, Style::default().fg(theme.colors.text_primary)),
        ]);

        let input_paragraph = Paragraph::new(vec![input_line])
            .wrap(Wrap { trim: false });

        frame.render_widget(input_paragraph, input_inner);

        // Set cursor position if focused
        if focused {
            let cursor_x = input_inner.x + prompt.len() as u16 + self.cursor_position as u16;
            let cursor_y = input_inner.y;
            
            if cursor_x < input_inner.x + input_inner.width {
                frame.set_cursor(cursor_x, cursor_y);
            }
        }
    }

    /// Create welcome message lines
    fn create_welcome_message(&self, theme: &ModernTheme) -> Vec<Line> {
        vec![
            Line::from(Span::styled(
                "Welcome to AI Pair Programming Chat! 👋",
                theme.typography.heading_style,
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("Commands: "),
                Span::styled("/help", Style::default().fg(theme.colors.info)),
                Span::raw(" | "),
                Span::styled("/clear", Style::default().fg(theme.colors.info)),
                Span::raw(" | "),
                Span::styled("/status", Style::default().fg(theme.colors.info)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Mentions: "),
                Span::styled("@model", Style::default().fg(theme.colors.secondary)),
                Span::raw(" | "),
                Span::styled("@provider", Style::default().fg(theme.colors.secondary)),
                Span::raw(" | "),
                Span::styled("@history", Style::default().fg(theme.colors.secondary)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Start typing to begin your conversation...",
                theme.typography.caption_style,
            )),
        ]
    }

    /// Format a message for display
    fn format_message(&self, message: &EnhancedChatMessage, theme: &ModernTheme, width: u16) -> Vec<Line> {
        let mut lines = Vec::new();

        // Message header with role and timestamp
        let (role_icon, role_color) = match message.role.as_str() {
            "user" => ("👤", theme.colors.primary),
            "assistant" => ("🤖", theme.colors.secondary),
            "system" => ("⚙️", theme.colors.warning),
            _ => ("📝", theme.colors.text_primary),
        };

        let timestamp_str = message.timestamp.format("%H:%M:%S").to_string();
        let header_line = Line::from(vec![
            Span::styled(
                format!("{} {}: ", role_icon, message.role.to_uppercase()),
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{}]", timestamp_str),
                Style::default().fg(theme.colors.secondary),
            ),
        ]);
        lines.push(header_line);

        // Message content (with word wrapping)
        let content_lines = self.wrap_text(&message.content, width.saturating_sub(4) as usize);
        for content_line in content_lines {
            lines.push(Line::from(Span::styled(
                format!("  {}", content_line),
                Style::default().fg(theme.colors.text_primary),
            )));
        }

        // Message status indicator
        match &message.status {
            MessageStatus::Receiving => {
                lines.push(Line::from(Span::styled(
                    "  ⏳ Receiving...",
                    Style::default().fg(theme.colors.warning),
                )));
            }
            MessageStatus::Error(err) => {
                lines.push(Line::from(Span::styled(
                    format!("  ❌ Error: {}", err),
                    Style::default().fg(theme.colors.error),
                )));
            }
            _ => {}
        }

        // Metadata (tokens, processing time, etc.)
        if let Some(tokens) = message.metadata.tokens {
            lines.push(Line::from(Span::styled(
                format!("  📊 {} tokens", tokens),
                Style::default().fg(theme.colors.secondary),
            )));
        }

        lines
    }

    /// Format typing indicator
    fn format_typing_indicator(&self, indicator: &TypingIndicator, theme: &ModernTheme) -> Vec<Line> {
        let animation_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let char_index = indicator.animation_frame % animation_chars.len();
        let animation_char = animation_chars[char_index];

        vec![
            Line::from(vec![
                Span::styled(
                    "🤖 AI: ",
                    Style::default()
                        .fg(theme.colors.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} {}", animation_char, indicator.message),
                    Style::default().fg(theme.colors.info),
                ),
            ])
        ]
    }

    /// Render scrollbar
    fn render_scrollbar(&self, frame: &mut Frame, area: Rect, _theme: &ModernTheme) {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::default()
            .content_length(self.messages.len())
            .position(self.scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(&ratatui::layout::Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }

    /// Handle input events
    pub fn handle_input(&mut self, key: KeyEvent) -> ChatAction {
        match key.code {
            KeyCode::Enter => {
                ChatAction::SendMessage
            }
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                ChatAction::ClearHistory
            }
            KeyCode::Char(c) => {
                self.input_text.insert(self.cursor_position, c);
                self.cursor_position += 1;
                ChatAction::None
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input_text.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
                ChatAction::None
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input_text.len() {
                    self.input_text.remove(self.cursor_position);
                }
                ChatAction::None
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                ChatAction::None
            }
            KeyCode::Right => {
                if self.cursor_position < self.input_text.len() {
                    self.cursor_position += 1;
                }
                ChatAction::None
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                ChatAction::None
            }
            KeyCode::End => {
                self.cursor_position = self.input_text.len();
                ChatAction::None
            }
            KeyCode::Esc => {
                self.input_text.clear();
                self.cursor_position = 0;
                ChatAction::None
            }
            KeyCode::PageUp => {
                self.scroll_up(5);
                ChatAction::ScrollUp
            }
            KeyCode::PageDown => {
                self.scroll_down(5);
                ChatAction::ScrollDown
            }
            KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_up(1);
                ChatAction::ScrollUp
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_down(1);
                ChatAction::ScrollDown
            }
            _ => ChatAction::None,
        }
    }

    /// Add a message to the chat
    pub fn add_message(&mut self, message: EnhancedChatMessage) {
        self.messages.push(message);
        
        // Auto-scroll to bottom when new message is added
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Start typing indicator
    pub fn start_typing_indicator(&mut self, message: Option<String>) {
        self.typing_indicator = Some(TypingIndicator {
            message: message.unwrap_or_else(|| "AI is thinking...".to_string()),
            animation_frame: 0,
            last_update: Utc::now(),
        });
    }

    /// Update typing indicator animation
    pub fn update_typing_indicator(&mut self) {
        if let Some(indicator) = &mut self.typing_indicator {
            let now = Utc::now();
            if now.signed_duration_since(indicator.last_update).num_milliseconds() > 100 {
                indicator.animation_frame += 1;
                indicator.last_update = now;
            }
        }
    }

    /// Stop typing indicator
    pub fn stop_typing_indicator(&mut self) {
        self.typing_indicator = None;
    }

    /// Update streaming message content
    pub fn update_streaming_message(&mut self, content: &str) {
        if let Some(last_message) = self.messages.last_mut() {
            if matches!(last_message.status, MessageStatus::Receiving) {
                last_message.content = content.to_string();
            }
        }
    }

    /// Get current input text
    pub fn get_input_text(&self) -> &str {
        &self.input_text
    }

    /// Clear input text
    pub fn clear_input(&mut self) {
        self.input_text.clear();
        self.cursor_position = 0;
    }

    /// Clear all messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
    }

    /// Scroll up by specified lines
    pub fn scroll_up(&mut self, lines: usize) {
        self.auto_scroll = false;
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Scroll down by specified lines
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = (self.scroll_offset + lines).min(self.max_scroll);
        
        // Re-enable auto-scroll if we're at the bottom
        if self.scroll_offset >= self.max_scroll {
            self.auto_scroll = true;
        }
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll;
        self.auto_scroll = true;
    }

    /// Update max scroll based on content
    pub fn update_max_scroll(&mut self, visible_height: usize) {
        let total_lines = self.messages.len() * 3; // Approximate lines per message
        self.max_scroll = total_lines.saturating_sub(visible_height);
    }

    /// Wrap text to fit within specified width
    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > width {
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                }
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        if lines.is_empty() {
            lines.push(String::new());
        }
        
        lines
    }
}

impl Default for MainChatArea {
    fn default() -> Self {
        Self::new()
    }
}