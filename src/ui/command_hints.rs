use crate::ui::theme::ModernTheme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

struct CommandHint {
    command: String,
    description: String,
}

pub struct CommandHints {
    pub visible: bool,
    input: String,
    hints: Vec<CommandHint>,
    selected_index: usize,
}

impl CommandHints {
    pub fn new() -> Self {
        Self {
            visible: false,
            input: String::new(),
            hints: vec![
                CommandHint { command: "/help".to_string(), description: "Show help".to_string() },
                CommandHint { command: "/clear".to_string(), description: "Clear chat history".to_string() },
                CommandHint { command: "/status".to_string(), description: "Show app status".to_string() },
                CommandHint { command: "/model".to_string(), description: "Set LLM model".to_string() },
                CommandHint { command: "/provider".to_string(), description: "Set LLM provider".to_string() },
                CommandHint { command: "/temp".to_string(), description: "Set temperature".to_string() },
                CommandHint { command: "/tokens".to_string(), description: "Set max tokens".to_string() },
                CommandHint { command: "/history".to_string(), description: "Show history".to_string() },
            ],
            selected_index: 0,
        }
    }

    pub fn update_input(&mut self, input: &str) {
        self.input = input.to_lowercase();
        self.visible = self.input.starts_with('/');
        self.selected_index = 0;
    }

    fn get_filtered_hints(&self) -> Vec<&CommandHint> {
        if !self.visible {
            return vec![];
        }
        self.hints
            .iter()
            .filter(|h| h.command.starts_with(&self.input))
            .collect()
    }

    pub fn select_next(&mut self) {
        let filtered = self.get_filtered_hints();
        if !filtered.is_empty() {
            self.selected_index = (self.selected_index + 1) % filtered.len();
        }
    }

    pub fn select_previous(&mut self) {
        let filtered = self.get_filtered_hints();
        if !filtered.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                filtered.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn get_selected_item(&self) -> Option<String> {
        let filtered = self.get_filtered_hints();
        filtered.get(self.selected_index).map(|h| h.command.clone())
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.visible = false;
        self.selected_index = 0;
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &ModernTheme) {
        if !self.visible {
            return;
        }

        let filtered = self.get_filtered_hints();
        let items: Vec<ListItem> = if filtered.is_empty() {
            vec![ListItem::new(Span::styled(
                "No matching commands",
                Style::default().fg(Color::Red).add_modifier(Modifier::ITALIC),
            ))]
        } else {
            filtered
                .iter()
                .enumerate()
                .map(|(i, hint)| {
                    let content = Line::from(vec![
                        Span::styled(
                            format!("{:<15}", hint.command),
                            Style::default().fg(theme.colors.primary).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" - "),
                        Span::styled(&hint.description, Style::default().fg(theme.colors.text_secondary)),
                    ]);
                    if i == self.selected_index {
                        ListItem::new(content).style(Style::default().bg(theme.colors.selection))
                    } else {
                        ListItem::new(content)
                    }
                })
                .collect()
        };

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" 🚀 Commands "));
        f.render_widget(list, area);
    }
}

impl Default for CommandHints {
    fn default() -> Self {
        Self::new()
    }
}
