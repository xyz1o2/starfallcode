/// Input Area Component
/// Renders the user input field with arrow indicator and cursor management

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};
use crate::app::App;

/// Renders the input area with arrow indicator
pub fn render_input_area(f: &mut Frame, app: &App, area: Rect, theme: &crate::ui::pixel_layout_v2::Theme) {
    // Background
    f.render_widget(Paragraph::new("").style(Style::default().bg(Color::Rgb(8, 8, 8))), area);

    // Horizontal split: arrow | input box
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),   // arrow
            Constraint::Min(10),     // input box
        ])
        .split(area);

    // 1. Render arrow indicator
    let arrow = "▶";
    f.render_widget(
        Paragraph::new(arrow).style(
            Style::default()
                .fg(theme.accent_user)
                .add_modifier(Modifier::BOLD),
        ),
        chunks[0],
    );

    // 2. Render input text
    let input_widget = Paragraph::new(app.input_text.as_str()).style(Style::default().fg(Color::White));
    f.render_widget(input_widget, chunks[1]);

    // 3. Calculate and set cursor position
    // Calculate the display width from start of string to cursor position
    let cursor_col = calculate_cursor_column(&app.input_text, app.input_cursor);

    // Set cursor position (x = input area start + cursor offset, y = input area start)
    f.set_cursor(
        chunks[1].x + cursor_col,
        chunks[1].y,
    );
}

/// Calculate the display column position for cursor based on character display width
/// Handles multi-byte characters correctly (important for Chinese/Japanese/Korean input)
fn calculate_cursor_column(text: &str, cursor_char_index: usize) -> u16 {
    // Get all characters up to the cursor position
    let chars_before_cursor: Vec<char> = text.chars().take(cursor_char_index).collect();

    // Calculate total display width using unicode-width
    let total_width: usize = chars_before_cursor.iter()
        .map(|c| unicode_width::UnicodeWidthChar::width(*c).unwrap_or(1))
        .sum();

    total_width as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::app::App;

    #[test]
    fn test_render_input_area() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        app.input_text = "Hello World".to_string();
        app.input_cursor = 5;

        let theme = crate::ui::pixel_layout_v2::Theme::new();

        terminal.draw(|f| {
            let area = f.size();
            render_input_area(f, &app, area, &theme);
        }).unwrap();
    }
}
