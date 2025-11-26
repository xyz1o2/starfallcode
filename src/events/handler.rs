use crate::app::{App, AppAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct EventHandler;

impl EventHandler {
    pub fn handle_chat_event(app: &mut App, key: KeyEvent) -> AppAction {
        if app.command_hints.visible {
            match key.code {
                KeyCode::Up => {
                    app.command_hints.select_previous();
                    return AppAction::None;
                }
                KeyCode::Down => {
                    app.command_hints.select_next();
                    return AppAction::None;
                }
                KeyCode::Tab | KeyCode::Enter => {
                    if let Some(completed) = app.command_hints.get_selected_item() {
                        app.input_text = completed;
                    }
                    app.command_hints.visible = false;
                    if key.code == KeyCode::Enter {
                        return AppAction::SubmitChat;
                    }
                    return AppAction::None;
                }
                KeyCode::Esc => {
                    app.command_hints.visible = false;
                    return AppAction::None;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => AppAction::Quit,
            KeyCode::Enter => AppAction::SubmitChat,
            KeyCode::Backspace => {
                app.input_text.pop();
                app.command_hints.update_input(&app.input_text);
                AppAction::None
            }
            KeyCode::Char(c) => {
                app.input_text.push(c);
                app.command_hints.update_input(&app.input_text);
                AppAction::None
            }
            _ => AppAction::None,
        }
    }
}