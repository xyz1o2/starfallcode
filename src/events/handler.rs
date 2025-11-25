use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::App;

pub struct EventHandler;

impl EventHandler {
    pub fn handle_chat_event(app: &mut App, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                false // Exit application
            }
            KeyCode::Enter => {
                app.handle_chat_submit();
                true
            }
            KeyCode::Backspace => {
                app.handle_chat_backspace();
                true
            }
            KeyCode::Char(c) => {
                app.handle_chat_input(c);
                true
            }
            _ => true,
        }
    }
}