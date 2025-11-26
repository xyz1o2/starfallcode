use crate::ui::types::{
    InfoSection, ModelInfoSection, TokenStatsSection, HelpInfoSection, 
    ErrorLogSection, SessionStatsSection, ErrorEntry, ErrorLevel, 
    ShortcutInfo, ConnectionStatus
};
use crate::ui::theme::ModernTheme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, Gauge, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use chrono::{DateTime, Utc};
use std::time::Duration;

pub struct InfoPanel {
    pub sections: Vec<InfoSection>,
    pub active_section: usize,
    pub auto_update: bool,
    pub scroll_offset: usize,
}

impl InfoPanel {
    pub fn new() -> Self {
        let mut panel = Self {
            sections: Vec::new(),
            active_section: 0,
            auto_update: true,
            scroll_offset: 0,
        };
        
        panel.init_default_sections();
        panel
    }

    /// Initialize default info panel sections
    fn init_default_sections(&mut self) {
        // Model Info Section
        let model_info = InfoSection::ModelInfo(ModelInfoSection {
            current_model: "Not configured".to_string(),
            provider: "None".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            connection_status: ConnectionStatus::Disconnected,
        });

        // Token Stats Section
        let token_stats = InfoSection::TokenStats(TokenStatsSection {
            tokens_used: 0,
            tokens_remaining: None,
            cost_estimate: None,
            session_tokens: 0,
        });

        // Help Info Section
        let help_info = InfoSection::HelpInfo(HelpInfoSection {
            current_context: "Main Chat".to_string(),
            available_shortcuts: vec![
                ShortcutInfo {
                    key: "Tab".to_string(),
                    description: "Switch between panels".to_string(),
                    context: "Global".to_string(),
                },
                ShortcutInfo {
                    key: "Ctrl+C".to_string(),
                    description: "Exit application".to_string(),
                    context: "Global".to_string(),
                },
                ShortcutInfo {
                    key: "Ctrl+L".to_string(),
                    description: "Clear chat history".to_string(),
                    context: "Chat".to_string(),
                },
                ShortcutInfo {
                    key: "F1".to_string(),
                    description: "Show help".to_string(),
                    context: "Global".to_string(),
                },
            ],
            tips: vec![
                "Use /help to see available commands".to_string(),
                "Use @mentions to reference context".to_string(),
                "Press Escape to clear input".to_string(),
            ],
        });

        // Error Log Section
        let error_log = InfoSection::ErrorLog(ErrorLogSection {
            errors: Vec::new(),
            max_entries: 50,
        });

        // Session Stats Section
        let session_stats = InfoSection::SessionStats(SessionStatsSection {
            session_duration: Duration::from_secs(0),
            messages_sent: 0,
            messages_received: 0,
            average_response_time: None,
        });

        self.sections = vec![model_info, token_stats, help_info, error_log, session_stats];
    }

    /// Render the info panel
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &ModernTheme) {
        let border_style = theme.get_border_style(focused);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" ℹ️ Information ")
            .title_alignment(Alignment::Left)
            .border_style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.height < 3 {
            return; // Not enough space to render content
        }

        // Create section tabs
        let tab_height = 2;
        let content_height = inner_area.height.saturating_sub(tab_height);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(tab_height),
                Constraint::Length(content_height),
            ])
            .split(inner_area);

        // Render section tabs
        self.render_section_tabs(frame, chunks[0], theme, focused);

        // Render active section content
        if self.active_section < self.sections.len() {
            self.render_section_content(
                frame,
                &self.sections[self.active_section],
                chunks[1],
                theme,
                focused,
            );
        }
    }

    /// Render section tabs
    fn render_section_tabs(&self, frame: &mut Frame, area: Rect, theme: &ModernTheme, focused: bool) {
        let tab_names = vec!["Model", "Tokens", "Help", "Errors", "Stats"];
        let tab_width = area.width / tab_names.len() as u16;

        let mut tab_lines = Vec::new();
        let mut tab_line = Vec::new();

        for (i, name) in tab_names.iter().enumerate() {
            let is_active = i == self.active_section;
            let style = if is_active && focused {
                theme.get_selection_style()
            } else if is_active {
                Style::default().fg(theme.colors.primary)
            } else {
                theme.typography.caption_style
            };

            let tab_text = format!(" {} ", name);
            let padding = tab_width.saturating_sub(tab_text.len() as u16);
            let padded_text = format!("{}{}", tab_text, " ".repeat(padding as usize));

            tab_line.push(Span::styled(padded_text, style));
        }

        tab_lines.push(Line::from(tab_line));
        
        // Add separator line
        tab_lines.push(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            theme.borders.section_border.fg.unwrap_or(theme.colors.border_inactive),
        )));

        let tabs_paragraph = Paragraph::new(tab_lines);
        frame.render_widget(tabs_paragraph, area);
    }

    /// Render section content
    fn render_section_content(
        &self,
        frame: &mut Frame,
        section: &InfoSection,
        area: Rect,
        theme: &ModernTheme,
        focused: bool,
    ) {
        match section {
            InfoSection::ModelInfo(model_section) => {
                self.render_model_info_section(frame, model_section, area, theme);
            }
            InfoSection::TokenStats(token_section) => {
                self.render_token_stats_section(frame, token_section, area, theme);
            }
            InfoSection::HelpInfo(help_section) => {
                self.render_help_info_section(frame, help_section, area, theme);
            }
            InfoSection::ErrorLog(error_section) => {
                self.render_error_log_section(frame, error_section, area, theme);
            }
            InfoSection::SessionStats(stats_section) => {
                self.render_session_stats_section(frame, stats_section, area, theme);
            }
        }
    }

    /// Render model info section
    fn render_model_info_section(
        &self,
        frame: &mut Frame,
        section: &ModelInfoSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        let mut lines = Vec::new();

        // Connection status
        let (status_text, status_color) = match &section.connection_status {
            ConnectionStatus::Connected => ("🟢 Connected", theme.colors.success),
            ConnectionStatus::Connecting => ("🟡 Connecting...", theme.colors.warning),
            ConnectionStatus::Disconnected => ("🔴 Disconnected", theme.colors.error),
            ConnectionStatus::Error(msg) => ("❌ Error", theme.colors.error),
        };

        lines.push(Line::from(vec![
            Span::styled("Status: ", theme.typography.body_style),
            Span::styled(status_text, Style::default().fg(status_color)),
        ]));
        lines.push(Line::from(""));

        // Model details
        lines.push(Line::from(vec![
            Span::styled("Model: ", theme.typography.body_style),
            Span::styled(&section.current_model, Style::default().fg(theme.colors.primary)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Provider: ", theme.typography.body_style),
            Span::styled(&section.provider, theme.typography.body_style),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Temperature: ", theme.typography.body_style),
            Span::styled(
                format!("{:.1}", section.temperature),
                theme.typography.body_style,
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Max Tokens: ", theme.typography.body_style),
            Span::styled(
                section.max_tokens.to_string(),
                theme.typography.body_style,
            ),
        ]));

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    /// Render token stats section
    fn render_token_stats_section(
        &self,
        frame: &mut Frame,
        section: &TokenStatsSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("Session Tokens: ", theme.typography.body_style),
            Span::styled(
                section.session_tokens.to_string(),
                Style::default().fg(theme.colors.primary),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Total Used: ", theme.typography.body_style),
            Span::styled(
                section.tokens_used.to_string(),
                theme.typography.body_style,
            ),
        ]));

        if let Some(remaining) = section.tokens_remaining {
            lines.push(Line::from(vec![
                Span::styled("Remaining: ", theme.typography.body_style),
                Span::styled(remaining.to_string(), theme.typography.body_style),
            ]));

            // Token usage gauge
            let usage_ratio = section.tokens_used as f64 / (section.tokens_used + remaining) as f64;
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Usage:", theme.typography.body_style)));
        }

        if let Some(cost) = section.cost_estimate {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Est. Cost: $", theme.typography.body_style),
                Span::styled(
                    format!("{:.4}", cost),
                    Style::default().fg(theme.colors.warning),
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    /// Render help info section
    fn render_help_info_section(
        &self,
        frame: &mut Frame,
        section: &HelpInfoSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),   // Context
                Constraint::Min(3),      // Shortcuts
                Constraint::Min(2),      // Tips
            ])
            .split(area);

        // Current context
        let context_line = Line::from(vec![
            Span::styled("Context: ", theme.typography.body_style),
            Span::styled(&section.current_context, Style::default().fg(theme.colors.primary)),
        ]);
        let context_paragraph = Paragraph::new(vec![context_line]);
        frame.render_widget(context_paragraph, chunks[0]);

        // Shortcuts
        let mut shortcut_items = Vec::new();
        for shortcut in &section.available_shortcuts {
            let item_text = format!("{}: {}", shortcut.key, shortcut.description);
            shortcut_items.push(ListItem::new(Line::from(Span::styled(
                item_text,
                theme.typography.body_style,
            ))));
        }

        let shortcuts_block = Block::default()
            .title("Shortcuts")
            .borders(Borders::TOP);
        let shortcuts_list = List::new(shortcut_items).block(shortcuts_block);
        frame.render_widget(shortcuts_list, chunks[1]);

        // Tips
        let mut tip_lines = Vec::new();
        for tip in &section.tips {
            tip_lines.push(Line::from(vec![
                Span::styled("💡 ", Style::default().fg(theme.colors.info)),
                Span::styled(tip, theme.typography.caption_style),
            ]));
        }

        let tips_block = Block::default()
            .title("Tips")
            .borders(Borders::TOP);
        let tips_paragraph = Paragraph::new(tip_lines)
            .block(tips_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(tips_paragraph, chunks[2]);
    }

    /// Render error log section
    fn render_error_log_section(
        &self,
        frame: &mut Frame,
        section: &ErrorLogSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        if section.errors.is_empty() {
            let no_errors = Paragraph::new(Line::from(Span::styled(
                "No errors logged ✅",
                Style::default().fg(theme.colors.success),
            )));
            frame.render_widget(no_errors, area);
            return;
        }

        let mut error_items = Vec::new();
        for error in section.errors.iter().rev().take(10) { // Show last 10 errors
            let (level_icon, level_color) = match error.level {
                ErrorLevel::Info => ("ℹ️", theme.colors.info),
                ErrorLevel::Warning => ("⚠️", theme.colors.warning),
                ErrorLevel::Error => ("❌", theme.colors.error),
                ErrorLevel::Critical => ("🚨", theme.colors.error),
            };

            let timestamp = error.timestamp.format("%H:%M:%S").to_string();
            let item_text = format!("{} [{}] {}", level_icon, timestamp, error.message);
            
            error_items.push(ListItem::new(Line::from(Span::styled(
                item_text,
                Style::default().fg(level_color),
            ))));
        }

        let error_list = List::new(error_items);
        frame.render_widget(error_list, area);
    }

    /// Render session stats section
    fn render_session_stats_section(
        &self,
        frame: &mut Frame,
        section: &SessionStatsSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        let mut lines = Vec::new();

        // Session duration
        let duration_str = format!(
            "{}h {}m {}s",
            section.session_duration.as_secs() / 3600,
            (section.session_duration.as_secs() % 3600) / 60,
            section.session_duration.as_secs() % 60
        );

        lines.push(Line::from(vec![
            Span::styled("Duration: ", theme.typography.body_style),
            Span::styled(duration_str, Style::default().fg(theme.colors.primary)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Messages Sent: ", theme.typography.body_style),
            Span::styled(
                section.messages_sent.to_string(),
                theme.typography.body_style,
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Messages Received: ", theme.typography.body_style),
            Span::styled(
                section.messages_received.to_string(),
                theme.typography.body_style,
            ),
        ]));

        if let Some(avg_time) = section.average_response_time {
            lines.push(Line::from(vec![
                Span::styled("Avg Response: ", theme.typography.body_style),
                Span::styled(
                    format!("{:.1}s", avg_time.as_secs_f64()),
                    theme.typography.body_style,
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    /// Handle input events
    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left => {
                if self.active_section > 0 {
                    self.active_section -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.active_section < self.sections.len().saturating_sub(1) {
                    self.active_section += 1;
                }
                true
            }
            KeyCode::Char('1') | KeyCode::Char('2') | KeyCode::Char('3') | KeyCode::Char('4') | KeyCode::Char('5') => {
                if let KeyCode::Char(c) = key.code {
                    let index = (c as u8 - b'1') as usize;
                    if index < self.sections.len() {
                        self.active_section = index;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Update model info
    pub fn update_model_info(&mut self, model: String, provider: String, connection: ConnectionStatus) {
        for section in &mut self.sections {
            if let InfoSection::ModelInfo(model_section) = section {
                model_section.current_model = model;
                model_section.provider = provider;
                model_section.connection_status = connection;
                break;
            }
        }
    }

    /// Update token stats
    pub fn update_token_stats(&mut self, session_tokens: u32, total_tokens: u32) {
        for section in &mut self.sections {
            if let InfoSection::TokenStats(token_section) = section {
                token_section.session_tokens = session_tokens;
                token_section.tokens_used = total_tokens;
                break;
            }
        }
    }

    /// Add error to log
    pub fn add_error(&mut self, level: ErrorLevel, message: String, details: Option<String>) {
        for section in &mut self.sections {
            if let InfoSection::ErrorLog(error_section) = section {
                let error_entry = ErrorEntry {
                    timestamp: Utc::now(),
                    level,
                    message,
                    details,
                };
                
                error_section.errors.push(error_entry);
                
                // Limit error log size
                if error_section.errors.len() > error_section.max_entries {
                    error_section.errors.remove(0);
                }
                break;
            }
        }
    }

    /// Update session stats
    pub fn update_session_stats(&mut self, duration: Duration, sent: u32, received: u32, avg_response: Option<Duration>) {
        for section in &mut self.sections {
            if let InfoSection::SessionStats(stats_section) = section {
                stats_section.session_duration = duration;
                stats_section.messages_sent = sent;
                stats_section.messages_received = received;
                stats_section.average_response_time = avg_response;
                break;
            }
        }
    }

    /// Cycle to next section
    pub fn cycle_section(&mut self) {
        self.active_section = (self.active_section + 1) % self.sections.len();
    }

    /// Get active section index
    pub fn get_active_section(&self) -> usize {
        self.active_section
    }

    /// Set active section
    pub fn set_active_section(&mut self, index: usize) {
        if index < self.sections.len() {
            self.active_section = index;
        }
    }
}

impl Default for InfoPanel {
    fn default() -> Self {
        Self::new()
    }
}