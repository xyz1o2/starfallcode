/// 文件名建议组件
/// 当AI检测到代码生成意图但未指定文件名时，显示建议的文件名供用户选择

use ratatui::{
    Frame,
    layout::{Rect, Constraint, Direction, Layout},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    text::{Line, Span},
};
use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Clone)]
pub struct FilenameSuggestion {
    pub visible: bool,
    pub selected_index: usize,
    pub suggestions: Vec<String>,
    pub code_content: String,
    pub detected_language: String,
}

impl FilenameSuggestion {
    pub fn new() -> Self {
        Self {
            visible: false,
            selected_index: 0,
            suggestions: Vec::new(),
            code_content: String::new(),
            detected_language: String::new(),
        }
    }

    /// 显示文件名建议对话框
    pub fn show(&mut self, code_content: String, language: String) {
        self.visible = true;
        self.code_content = code_content;
        self.detected_language = language.clone();
        self.selected_index = 0;

        // 生成文件名建议
        self.suggestions = self.generate_suggestions(&language);
    }

    /// 隐藏对话框
    pub fn hide(&mut self) {
        self.visible = false;
        self.suggestions.clear();
        self.code_content.clear();
    }

    /// 生成文件名建议
    fn generate_suggestions(&self, language: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        match language.to_lowercase().as_str() {
            "rust" => {
                suggestions.push("main.rs".to_string());
                suggestions.push("lib.rs".to_string());
                suggestions.push("mod.rs".to_string());
                suggestions.push("test.rs".to_string());
            }
            "html" => {
                suggestions.push("index.html".to_string());
                suggestions.push("demo.html".to_string());
                suggestions.push("test.html".to_string());
                suggestions.push("page.html".to_string());
            }
            "javascript" => {
                suggestions.push("main.js".to_string());
                suggestions.push("app.js".to_string());
                suggestions.push("index.js".to_string());
                suggestions.push("test.js".to_string());
            }
            "python" => {
                suggestions.push("main.py".to_string());
                suggestions.push("app.py".to_string());
                suggestions.push("script.py".to_string());
                suggestions.push("test.py".to_string());
            }
            _ => {
                suggestions.push(format!("main.{}", language.to_lowercase()));
                suggestions.push(format!("demo.{}", language.to_lowercase()));
                suggestions.push(format!("test.{}", language.to_lowercase()));
            }
        }

        suggestions
    }

    /// 选择上一个建议
    pub fn select_previous(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    /// 选择下一个建议
    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }

    /// 获取当前选中的文件名
    pub fn get_selected(&self) -> Option<String> {
        self.suggestions.get(self.selected_index).cloned()
    }

    /// 获取所有建议
    pub fn get_suggestions(&self) -> &[String] {
        &self.suggestions
    }

    /// 获取代码内容
    pub fn get_code_content(&self) -> &str {
        &self.code_content
    }

    /// 是否可见
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 渲染对话框
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // 清除背景
        frame.render_widget(Clear, area);

        // 创建对话框布局
        let block = Block::default()
            .title(" 🤖 选择文件名 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // 分割为标题、列表和提示
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 说明文字
                Constraint::Min(5),     // 建议列表
                Constraint::Length(3),  // 操作提示
            ])
            .split(inner_area);

        // 1. 说明文字
        let description = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("检测到 "),
                Span::styled(&self.detected_language, Style::default().fg(Color::Yellow)),
                Span::raw(" 代码块，请选择文件名："),
            ]),
        ])
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::White));
        frame.render_widget(description, chunks[0]);

        // 2. 建议列表
        let items: Vec<ListItem> = self.suggestions
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if i == self.selected_index {
                    "▶ "
                } else {
                    "  "
                };

                ListItem::new(format!("{}{}", prefix, name)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" 建议文件名 ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
        frame.render_widget(list, chunks[1]);

        // 3. 操作提示
        let help_text = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("↑↓ "),
                Span::styled("选择", Style::default().fg(Color::Green)),
                Span::raw(" | "),
                Span::raw("Enter "),
                Span::styled("确认", Style::default().fg(Color::Green)),
                Span::raw(" | "),
                Span::raw("Esc "),
                Span::styled("取消", Style::default().fg(Color::Red)),
                Span::raw(" | "),
                Span::raw("或直接输入路径: /create-file path"),
            ]),
        ])
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help_text, chunks[2]);
    }

    /// 获取推荐的文件名（无前缀）
    pub fn get_recommended_filename(&self) -> Option<String> {
        self.get_selected().and_then(|name| {
            // 移除 UNSPECIFIED_ 前缀
            name.strip_prefix("UNSPECIFIED_").map(|s| s.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_suggestion_generation() {
        let mut suggestion = FilenameSuggestion::new();
        suggestion.show("fn main() {}".to_string(), "rust".to_string());

        assert!(suggestion.is_visible());
        assert_eq!(suggestion.suggestions.len(), 4);
        assert!(suggestion.suggestions.contains(&"main.rs".to_string()));
        assert_eq!(suggestion.detected_language, "rust");
    }

    #[test]
    fn test_navigation() {
        let mut suggestion = FilenameSuggestion::new();
        suggestion.show("<html></html>".to_string(), "html".to_string());

        let first = suggestion.get_selected();
        suggestion.select_next();
        let second = suggestion.get_selected();

        assert_ne!(first, second);

        suggestion.select_previous();
        let back_to_first = suggestion.get_selected();

        assert_eq!(first, back_to_first);
    }

    #[test]
    fn test_get_recommended_filename() {
        let mut suggestion = FilenameSuggestion::new();
        suggestion.show("fn main() {}".to_string(), "rust".to_string());

        // 修改测试数据以匹配新的数据结构
        suggestion.suggestions = vec!["UNSPECIFIED_main.rs".to_string()];
        suggestion.selected_index = 0;

        let recommended = suggestion.get_recommended_filename();
        assert_eq!(recommended, Some("main.rs".to_string()));
    }
}
