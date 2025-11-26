use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::App;

pub struct EventHandler;

impl EventHandler {
    pub fn handle_chat_event(app: &mut App, key_event: KeyEvent) -> bool {
        // 如果命令提示可见，优先处理导航
        if app.command_hints.visible {
            match key_event.code {
                KeyCode::Up => {
                    app.command_hints.select_previous();
                    return true;
                }
                KeyCode::Down => {
                    app.command_hints.select_next();
                    return true;
                }
                KeyCode::Enter | KeyCode::Tab => {
                    // Enter 或 Tab 键自动完成选中的命令
                    if let Some(hint) = app.command_hints.get_selected() {
                        app.chat_input = hint.command.clone();
                    }
                    // 如果是 Enter，则继续执行提交逻辑
                    if key_event.code == KeyCode::Enter {
                        app.handle_chat_submit();
                    } else {
                        // 如果是 Tab，只自动完成，不提交
                        app.command_hints.clear();
                    }
                    return true;
                }
                KeyCode::Esc => {
                    app.command_hints.clear();
                    return true;
                }
                _ => {}
            }
        }

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