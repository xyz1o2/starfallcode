use crate::ui::types::{
    SidebarSection, SidebarAction, ChatHistorySection, QuickCommandsSection, 
    SystemStatusSection, SettingsSection, ChatSession, QuickCommand, 
    CommandCategory, ConnectionStatus, ModelInfo, PerformanceStats
};
use crate::ui::theme::ModernTheme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use chrono::Utc;

pub struct Sidebar {
    pub sections: Vec<SidebarSection>,
    pub selected_section: usize,
    pub selected_item: usize,
    pub scroll_offset: usize,
    pub list_state: ListState,
    pub expanded_sections: std::collections::HashSet<usize>,
}

impl Sidebar {
    pub fn new() -> Self {
        let mut sidebar = Self {
            sections: Vec::new(),
            selected_section: 0,
            selected_item: 0,
            scroll_offset: 0,
            list_state: ListState::default(),
            expanded_sections: std::collections::HashSet::new(),
        };
        
        sidebar.init_default_sections();
        sidebar.expanded_sections.insert(0); // Expand first section by default
        sidebar
    }

    /// Initialize default sidebar sections
    fn init_default_sections(&mut self) {
        // Chat History Section
        let chat_history = SidebarSection::ChatHistory(ChatHistorySection {
            sessions: vec![
                ChatSession {
                    id: "session_1".to_string(),
                    title: "Current Session".to_string(),
                    message_count: 0,
                    last_updated: Utc::now(),
                },
            ],
            selected_session: Some(0),
            max_display_items: 10,
        });

        // Quick Commands Section
        let quick_commands = SidebarSection::QuickCommands(QuickCommandsSection {
            commands: vec![
                QuickCommand {
                    name: "Clear Chat".to_string(),
                    description: "Clear chat history".to_string(),
                    shortcut: Some("Ctrl+L".to_string()),
                    category: "Chat".to_string(),
                },
                QuickCommand {
                    name: "Help".to_string(),
                    description: "Show help information".to_string(),
                    shortcut: Some("F1".to_string()),
                    category: "General".to_string(),
                },
                QuickCommand {
                    name: "Switch Theme".to_string(),
                    description: "Change UI theme".to_string(),
                    shortcut: Some("Ctrl+T".to_string()),
                    category: "UI".to_string(),
                },
            ],
            categories: vec![
                CommandCategory {
                    name: "Chat".to_string(),
                    expanded: true,
                    commands: vec!["Clear Chat".to_string()],
                },
                CommandCategory {
                    name: "General".to_string(),
                    expanded: true,
                    commands: vec!["Help".to_string()],
                },
                CommandCategory {
                    name: "UI".to_string(),
                    expanded: true,
                    commands: vec!["Switch Theme".to_string()],
                },
            ],
        });

        // System Status Section
        let system_status = SidebarSection::SystemStatus(SystemStatusSection {
            connection_status: ConnectionStatus::Disconnected,
            model_info: None,
            performance_stats: PerformanceStats::default(),
        });

        // Settings Section
        let settings = SidebarSection::Settings(SettingsSection {
            theme_name: "Dark Professional".to_string(),
            auto_save: true,
            notifications: true,
        });

        self.sections = vec![chat_history, quick_commands, system_status, settings];
    }

    /// Render the sidebar
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &ModernTheme) {
        let border_style = theme.get_border_style(focused);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 📋 Sidebar ")
            .title_alignment(Alignment::Left)
            .border_style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.height < 3 {
            return; // Not enough space to render content
        }

        // Calculate section heights
        let section_count = self.sections.len();
        let available_height = inner_area.height as usize;
        let section_height = if section_count > 0 {
            std::cmp::max(3, available_height / section_count)
        } else {
            available_height
        };

        // Create layout for sections
        let mut constraints = Vec::new();
        for i in 0..section_count {
            if self.expanded_sections.contains(&i) {
                constraints.push(Constraint::Length(section_height as u16));
            } else {
                constraints.push(Constraint::Length(2)); // Collapsed section height
            }
        }

        if !constraints.is_empty() {
            let section_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(inner_area);

            // Render each section
            for (i, section) in self.sections.iter().enumerate() {
                if i < section_chunks.len() {
                    let is_selected = focused && i == self.selected_section;
                    self.render_section(frame, section, section_chunks[i], is_selected, i, theme);
                }
            }
        }
    }

    /// Render a specific section
    fn render_section(
        &self,
        frame: &mut Frame,
        section: &SidebarSection,
        area: Rect,
        selected: bool,
        section_index: usize,
        theme: &ModernTheme,
    ) {
        let is_expanded = self.expanded_sections.contains(&section_index);
        let _style = if selected {
            theme.get_selection_style()
        } else {
            theme.typography.body_style
        };

        match section {
            SidebarSection::ChatHistory(chat_section) => {
                self.render_chat_history_section(frame, chat_section, area, selected, is_expanded, theme);
            }
            SidebarSection::QuickCommands(cmd_section) => {
                self.render_quick_commands_section(frame, cmd_section, area, selected, is_expanded, theme);
            }
            SidebarSection::SystemStatus(status_section) => {
                self.render_system_status_section(frame, status_section, area, selected, is_expanded, theme);
            }
            SidebarSection::Settings(settings_section) => {
                self.render_settings_section(frame, settings_section, area, selected, is_expanded, theme);
            }
        }
    }

    /// Render chat history section
    fn render_chat_history_section(
        &self,
        frame: &mut Frame,
        section: &ChatHistorySection,
        area: Rect,
        selected: bool,
        expanded: bool,
        theme: &ModernTheme,
    ) {
        let title = if expanded { "📜 Chat History ▼" } else { "📜 Chat History ▶" };
        let border_style = if selected {
            theme.borders.active_border
        } else {
            theme.borders.section_border
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        if !expanded {
            frame.render_widget(block, area);
            return;
        }

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.height < 1 {
            return;
        }

        let mut items = Vec::new();
        for (i, session) in section.sessions.iter().enumerate() {
            let is_selected = section.selected_session == Some(i);
            let style = if is_selected {
                theme.get_selection_style()
            } else {
                theme.typography.body_style
            };

            let item_text = format!("• {} ({})", session.title, session.message_count);
            items.push(ListItem::new(Line::from(Span::styled(item_text, style))));
        }

        if items.is_empty() {
            let empty_text = Paragraph::new("No chat sessions")
                .style(theme.typography.caption_style);
            frame.render_widget(empty_text, inner_area);
        } else {
            let list = List::new(items);
            frame.render_widget(list, inner_area);
        }
    }

    /// Render quick commands section
    fn render_quick_commands_section(
        &self,
        frame: &mut Frame,
        section: &QuickCommandsSection,
        area: Rect,
        selected: bool,
        expanded: bool,
        theme: &ModernTheme,
    ) {
        let title = if expanded { "⚡ Quick Commands ▼" } else { "⚡ Quick Commands ▶" };
        let border_style = if selected {
            theme.borders.active_border
        } else {
            theme.borders.section_border
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        if !expanded {
            frame.render_widget(block, area);
            return;
        }

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.height < 1 {
            return;
        }

        let mut items = Vec::new();
        for command in &section.commands {
            let shortcut_text = command.shortcut
                .as_ref()
                .map(|s| format!(" ({})", s))
                .unwrap_or_default();
            
            let item_text = format!("• {}{}", command.name, shortcut_text);
            items.push(ListItem::new(Line::from(Span::styled(
                item_text,
                theme.typography.body_style,
            ))));
        }

        let list = List::new(items);
        frame.render_widget(list, inner_area);
    }

    /// Render system status section
    fn render_system_status_section(
        &self,
        frame: &mut Frame,
        section: &SystemStatusSection,
        area: Rect,
        selected: bool,
        expanded: bool,
        theme: &ModernTheme,
    ) {
        let title = if expanded { "📊 System Status ▼" } else { "📊 System Status ▶" };
        let border_style = if selected {
            theme.borders.active_border
        } else {
            theme.borders.section_border
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        if !expanded {
            frame.render_widget(block, area);
            return;
        }

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.height < 1 {
            return;
        }

        let mut lines = Vec::new();

        // Connection status
        let (status_text, status_color) = match &section.connection_status {
            ConnectionStatus::Connected => ("🟢 Connected", theme.colors.success),
            ConnectionStatus::Connecting => ("🟡 Connecting", theme.colors.warning),
            ConnectionStatus::Disconnected => ("🔴 Disconnected", theme.colors.error),
            ConnectionStatus::Error(_msg) => ("❌ Error", theme.colors.error),
        };
        lines.push(Line::from(Span::styled(status_text, Style::default().fg(status_color))));

        // Model info
        if let Some(model) = &section.model_info {
            lines.push(Line::from(Span::styled(
                format!("Model: {}", model.name),
                theme.typography.body_style,
            )));
        }

        // Performance stats
        lines.push(Line::from(Span::styled(
            format!("Memory: {} MB", section.performance_stats.memory_usage / 1024 / 1024),
            theme.typography.caption_style,
        )));

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner_area);
    }

    /// Render settings section
    fn render_settings_section(
        &self,
        frame: &mut Frame,
        section: &SettingsSection,
        area: Rect,
        selected: bool,
        expanded: bool,
        theme: &ModernTheme,
    ) {
        let title = if expanded { "⚙️ Settings ▼" } else { "⚙️ Settings ▶" };
        let border_style = if selected {
            theme.borders.active_border
        } else {
            theme.borders.section_border
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        if !expanded {
            frame.render_widget(block, area);
            return;
        }

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if inner_area.height < 1 {
            return;
        }

        let lines = vec![
            Line::from(Span::styled(
                format!("Theme: {}", section.theme_name),
                theme.typography.body_style,
            )),
            Line::from(Span::styled(
                format!("Auto-save: {}", if section.auto_save { "On" } else { "Off" }),
                theme.typography.body_style,
            )),
            Line::from(Span::styled(
                format!("Notifications: {}", if section.notifications { "On" } else { "Off" }),
                theme.typography.body_style,
            )),
        ];

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner_area);
    }

    /// Handle input events
    pub fn handle_input(&mut self, key: KeyEvent) -> SidebarAction {
        match key.code {
            KeyCode::Up => {
                if self.selected_section > 0 {
                    self.selected_section -= 1;
                }
                SidebarAction::SelectPrevious
            }
            KeyCode::Down => {
                if self.selected_section < self.sections.len().saturating_sub(1) {
                    self.selected_section += 1;
                }
                SidebarAction::SelectNext
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Toggle section expansion
                if self.expanded_sections.contains(&self.selected_section) {
                    self.expanded_sections.remove(&self.selected_section);
                    SidebarAction::CollapseSection
                } else {
                    self.expanded_sections.insert(self.selected_section);
                    SidebarAction::ExpandSection
                }
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Expand all sections
                for i in 0..self.sections.len() {
                    self.expanded_sections.insert(i);
                }
                SidebarAction::ExpandSection
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Collapse all sections
                self.expanded_sections.clear();
                SidebarAction::CollapseSection
            }
            _ => SidebarAction::SelectNext, // Default action
        }
    }

    /// Update chat history
    pub fn update_chat_history(&mut self, sessions: &[ChatSession]) {
        for section in &mut self.sections {
            if let SidebarSection::ChatHistory(chat_section) = section {
                chat_section.sessions = sessions.to_vec();
                break;
            }
        }
    }

    /// Update system status
    pub fn update_system_status(&mut self, connection: ConnectionStatus, model: Option<ModelInfo>) {
        for section in &mut self.sections {
            if let SidebarSection::SystemStatus(status_section) = section {
                status_section.connection_status = connection;
                status_section.model_info = model;
                break;
            }
        }
    }

    /// Update theme name in settings
    pub fn update_theme_name(&mut self, theme_name: String) {
        for section in &mut self.sections {
            if let SidebarSection::Settings(settings_section) = section {
                settings_section.theme_name = theme_name;
                break;
            }
        }
    }

    /// Get currently selected section index
    pub fn get_selected_section(&self) -> usize {
        self.selected_section
    }

    /// Set selected section
    pub fn set_selected_section(&mut self, index: usize) {
        if index < self.sections.len() {
            self.selected_section = index;
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}