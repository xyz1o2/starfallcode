use crate::app::{App, AppAction, ModificationChoice};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct EventHandler;

impl EventHandler {
    pub fn handle_chat_event(app: &mut App, key: KeyEvent) -> AppAction {
        // 最高优先级：处理 AI 代码修改确认对话
        if app.modification_confirmation_pending && !app.pending_modifications.is_empty() {
            match key.code {
                KeyCode::Up => {
                    // 切换到"确认"
                    app.modification_choice = ModificationChoice::Confirm;
                    return AppAction::None;
                }
                KeyCode::Down => {
                    // 切换到"取消"
                    app.modification_choice = ModificationChoice::Cancel;
                    return AppAction::None;
                }
                KeyCode::Enter => {
                    // 执行选择
                    if app.modification_choice == ModificationChoice::Confirm {
                        // 执行修改
                        for (op, _diff) in &app.pending_modifications {
                            match op {
                                crate::ai::code_modification::CodeModificationOp::Create { path, content } => {
                                    // 创建文件
                                    match std::fs::write(path, content) {
                                        Ok(_) => {
                                            app.chat_history.add_message(crate::core::message::Message {
                                                role: crate::core::message::Role::System,
                                                content: format!("✅ 文件已创建: {}", path),
                                            });
                                        }
                                        Err(e) => {
                                            app.chat_history.add_message(crate::core::message::Message {
                                                role: crate::core::message::Role::System,
                                                content: format!("❌ 创建文件失败: {}", e),
                                            });
                                        }
                                    }
                                }
                                crate::ai::code_modification::CodeModificationOp::Modify { path, search: _, replace } => {
                                    // 修改文件
                                    match std::fs::read_to_string(path) {
                                        Ok(content) => {
                                            let new_content = content.replace(&content, replace);
                                            match std::fs::write(path, new_content) {
                                                Ok(_) => {
                                                    app.chat_history.add_message(crate::core::message::Message {
                                                        role: crate::core::message::Role::System,
                                                        content: format!("✅ 文件已修改: {}", path),
                                                    });
                                                }
                                                Err(e) => {
                                                    app.chat_history.add_message(crate::core::message::Message {
                                                        role: crate::core::message::Role::System,
                                                        content: format!("❌ 修改文件失败: {}", e),
                                                    });
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            app.chat_history.add_message(crate::core::message::Message {
                                                role: crate::core::message::Role::System,
                                                content: format!("❌ 读取文件失败: {}", e),
                                            });
                                        }
                                    }
                                }
                                crate::ai::code_modification::CodeModificationOp::Delete { path } => {
                                    // 删除文件
                                    match std::fs::remove_file(path) {
                                        Ok(_) => {
                                            app.chat_history.add_message(crate::core::message::Message {
                                                role: crate::core::message::Role::System,
                                                content: format!("✅ 文件已删除: {}", path),
                                            });
                                        }
                                        Err(e) => {
                                            app.chat_history.add_message(crate::core::message::Message {
                                                role: crate::core::message::Role::System,
                                                content: format!("❌ 删除文件失败: {}", e),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // 取消修改
                        app.chat_history.add_message(crate::core::message::Message {
                            role: crate::core::message::Role::System,
                            content: "✅ 修改已取消".to_string(),
                        });
                    }
                    
                    // 清空待确认的修改
                    app.pending_modifications.clear();
                    app.modification_confirmation_pending = false;
                    return AppAction::None;
                }
                KeyCode::Esc => {
                    // 取消修改
                    app.pending_modifications.clear();
                    app.modification_confirmation_pending = false;
                    return AppAction::None;
                }
                _ => return AppAction::None,
            }
        }

        // 次优先级：处理文件命令确认对话
        if app.file_command_handler.has_pending_confirmation() {
            match key.code {
                KeyCode::Up => {
                    app.file_command_handler.move_confirmation_up();
                    return AppAction::None;
                }
                KeyCode::Down => {
                    app.file_command_handler.move_confirmation_down();
                    return AppAction::None;
                }
                KeyCode::Enter => {
                    // 执行确认选择
                    let _choice = app.file_command_handler.get_confirmation_choice();
                    let _cmd = crate::commands::FileCommand::ConfirmModify;
                    // 这里会在后续的命令处理中执行
                    return AppAction::SubmitChat;
                }
                KeyCode::Esc => {
                    // 取消确认
                    let _cmd = crate::commands::FileCommand::CancelModify;
                    let _ = app.file_command_handler.execute(_cmd);
                    return AppAction::None;
                }
                _ => return AppAction::None,
            }
        }

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