use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::app::App;

pub struct EventHandler;

impl EventHandler {
    pub fn handle_key_event(app: &mut App, key_event: KeyEvent) -> bool {
        if app.is_chat_focused {
            // Handle chat input mode
            match key_event.code {
                KeyCode::Enter => {
                    app.handle_chat_submit();
                    true
                },
                KeyCode::Backspace => {
                    app.handle_chat_backspace();
                    true
                },
                KeyCode::Char(c) => {
                    app.handle_chat_input(c);
                    true
                },
                KeyCode::Tab => {
                    app.toggle_chat_focus(); // Switch back to editor
                    true
                },
                KeyCode::Esc => {
                    app.toggle_chat_focus(); // Switch back to editor
                    true
                },
                KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    false // Exit application
                },
                _ => true,
            }
        } else {
            // Handle editor mode
            match key_event.code {
                KeyCode::Char(c) => {
                    app.handle_char_input(c);
                    app.trigger_completion();
                    true
                },
                KeyCode::Backspace => {
                    app.handle_backspace();
                    app.trigger_completion();
                    true
                },
                KeyCode::Enter => {
                    app.handle_enter();
                    app.trigger_completion();
                    true
                },
                KeyCode::Left => {
                    app.handle_left();
                    app.trigger_completion();
                    true
                },
                KeyCode::Right => {
                    app.handle_right();
                    app.trigger_completion();
                    true
                },
                KeyCode::Up => {
                    app.handle_up();
                    app.trigger_completion();
                    true
                },
                KeyCode::Down => {
                    app.handle_down();
                    app.trigger_completion();
                    true
                },
                KeyCode::Tab => {
                    // Switch to chat input if not already focused
                    if !app.is_chat_focused {
                        app.toggle_chat_focus();
                    } else {
                        app.accept_ghost_text();
                    }
                    true
                },
                KeyCode::Esc => {
                    app.clear_ghost_text();
                    true
                },
                KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    false // Exit application
                },
                _ => true,
            }
        }
    }
}