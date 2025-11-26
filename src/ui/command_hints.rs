use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use crate::ui::theme::ModernTheme;

#[derive(Clone, Debug)]
pub struct CommandHint {
    pub command: String,
    pub description: String,
    pub example: String,
}

pub struct CommandHints {
    pub hints: Vec<CommandHint>,
    pub selected_index: usize,
    pub visible: bool,
    pub filter: String,
}

impl CommandHints {
    pub fn new() -> Self {
        let hints = vec![
            CommandHint {
                command: "/help".to_string(),
                description: "显示帮助信息".to_string(),
                example: "显示所有可用命令".to_string(),
            },
            CommandHint {
                command: "/clear".to_string(),
                description: "清除聊天历史".to_string(),
                example: "删除所有消息".to_string(),
            },
            CommandHint {
                command: "/status".to_string(),
                description: "显示应用状态".to_string(),
                example: "显示当前模型和提供商".to_string(),
            },
            CommandHint {
                command: "/model".to_string(),
                description: "显示/设置模型".to_string(),
                example: "/model gpt-4".to_string(),
            },
            CommandHint {
                command: "/provider".to_string(),
                description: "显示/切换提供商".to_string(),
                example: "/provider openai".to_string(),
            },
            CommandHint {
                command: "/temp".to_string(),
                description: "设置温度参数".to_string(),
                example: "/temp 0.7".to_string(),
            },
            CommandHint {
                command: "/tokens".to_string(),
                description: "设置最大令牌数".to_string(),
                example: "/tokens 2000".to_string(),
            },
            CommandHint {
                command: "/history".to_string(),
                description: "显示聊天历史".to_string(),
                example: "列出最近的消息".to_string(),
            },
        ];

        Self {
            hints,
            selected_index: 0,
            visible: false,
            filter: String::new(),
        }
    }

    /// 当用户输入 `/` 时激活提示
    pub fn activate(&mut self, input: &str) {
        if input.starts_with('/') {
            self.visible = true;
            self.filter = input[1..].to_lowercase();
            self.selected_index = 0;
        } else {
            self.visible = false;
        }
    }

    /// 获取过滤后的提示
    pub fn get_filtered_hints(&self) -> Vec<&CommandHint> {
        if self.filter.is_empty() {
            // 如果没有过滤条件，显示所有命令
            self.hints.iter().collect()
        } else {
            // 根据过滤条件查找命令
            let search_term = format!("/{}", self.filter);
            self.hints
                .iter()
                .filter(|h| h.command.starts_with(&search_term) || h.command.contains(&search_term))
                .collect()
        }
    }

    /// 选择下一个提示
    pub fn select_next(&mut self) {
        let filtered = self.get_filtered_hints();
        if !filtered.is_empty() {
            self.selected_index = (self.selected_index + 1) % filtered.len();
        }
    }

    /// 选择上一个提示
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

    /// 获取当前选中的提示
    pub fn get_selected(&self) -> Option<&CommandHint> {
        let filtered = self.get_filtered_hints();
        filtered.get(self.selected_index).copied()
    }

    /// 渲染提示面板
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &ModernTheme) {
        if !self.visible {
            return;
        }

        let filtered = self.get_filtered_hints();
        
        let mut items = Vec::new();
        
        if filtered.is_empty() {
            // 显示"无匹配命令"提示
            items.push(ListItem::new(Line::from(Span::styled(
                "No matching commands",
                Style::default().fg(theme.colors.warning),
            ))));
        } else {
            for (idx, hint) in filtered.iter().enumerate() {
                let style = if idx == self.selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(theme.colors.primary)
                } else {
                    Style::default().fg(theme.colors.text_primary)
                };

                let item_text = format!("{} - {}", hint.command, hint.description);
                items.push(ListItem::new(Line::from(Span::styled(item_text, style))));
            }
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 💡 命令提示 ")
            .border_style(Style::default().fg(theme.colors.secondary));

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// 清空过滤
    pub fn clear(&mut self) {
        self.visible = false;
        self.filter.clear();
        self.selected_index = 0;
    }
}

impl Default for CommandHints {
    fn default() -> Self {
        Self::new()
    }
}
