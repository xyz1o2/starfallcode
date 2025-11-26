use crate::app::{App, AppAction, ModificationChoice};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub struct EventHandler;

impl EventHandler {
    pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) -> AppAction {
        match mouse.kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                // 左键按下 - 开始选择
                app.selection_start = Some((mouse.column, mouse.row));
                app.selection_end = None;
                app.selected_text.clear();
                AppAction::None
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                // 左键释放 - 结束选择
                app.selection_end = Some((mouse.column, mouse.row));
                AppAction::None
            }
            MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                // 拖动 - 更新选择范围
                app.selection_end = Some((mouse.column, mouse.row));
                AppAction::None
            }
            _ => AppAction::None,
        }
    }
    
    pub fn handle_chat_event(app: &mut App, key: KeyEvent) -> AppAction {
        // 最高优先级：处理 AI 代码修改确认对话
        if app.modification_confirmation_pending && !app.pending_modifications.is_empty() {
            match key.code {
                KeyCode::Up => {
                    // 上键 - 向上循环切换
                    app.modification_choice = match app.modification_choice {
                        ModificationChoice::Confirm => ModificationChoice::Abandon,
                        ModificationChoice::Cancel => ModificationChoice::Confirm,
                        ModificationChoice::Abandon => ModificationChoice::Cancel,
                    };
                    return AppAction::None;
                }
                KeyCode::Down => {
                    // 下键 - 向下循环切换
                    app.modification_choice = match app.modification_choice {
                        ModificationChoice::Confirm => ModificationChoice::Cancel,
                        ModificationChoice::Cancel => ModificationChoice::Abandon,
                        ModificationChoice::Abandon => ModificationChoice::Confirm,
                    };
                    return AppAction::None;
                }
                KeyCode::Char('1') => {
                    // 数字 1 - 确认
                    app.modification_choice = ModificationChoice::Confirm;
                    // 立即执行
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
                KeyCode::Char('2') | KeyCode::Char('n') | KeyCode::Char('N') => {
                    // 数字 2 或 N 键 - 取消
                    app.chat_history.add_message(crate::core::message::Message {
                        role: crate::core::message::Role::System,
                        content: "✅ 修改已取消".to_string(),
                    });
                    
                    // 清空待确认的修改
                    app.pending_modifications.clear();
                    app.modification_confirmation_pending = false;
                    return AppAction::None;
                }
                KeyCode::Char('3') => {
                    // 数字 3 - 放弃
                    app.modification_choice = ModificationChoice::Abandon;
                    // 立即执行
                    app.chat_history.add_message(crate::core::message::Message {
                        role: crate::core::message::Role::System,
                        content: "✅ 修改已放弃".to_string(),
                    });
                    app.pending_modifications.clear();
                    app.modification_confirmation_pending = false;
                    return AppAction::None;
                }
                KeyCode::Esc => {
                    // Esc - 放弃
                    app.chat_history.add_message(crate::core::message::Message {
                        role: crate::core::message::Role::System,
                        content: "✅ 修改已放弃".to_string(),
                    });
                    app.pending_modifications.clear();
                    app.modification_confirmation_pending = false;
                    return AppAction::None;
                }
                KeyCode::Enter => {
                    // Enter - 执行当前选择
                    match app.modification_choice {
                        ModificationChoice::Confirm => {
                            // 执行修改
                            for (op, _diff) in &app.pending_modifications {
                                match op {
                                    crate::ai::code_modification::CodeModificationOp::Create { path, content } => {
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
                        }
                        ModificationChoice::Cancel | ModificationChoice::Abandon => {
                            // 取消或放弃修改
                            app.chat_history.add_message(crate::core::message::Message {
                                role: crate::core::message::Role::System,
                                content: "✅ 修改已取消".to_string(),
                            });
                        }
                    }
                    
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
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                // Ctrl+C - 如果有选中文本则复制，否则退出
                if !app.selected_text.is_empty() {
                    // 复制到剪贴板
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(app.selected_text.clone());
                        app.chat_history.add_message(crate::core::message::Message {
                            role: crate::core::message::Role::System,
                            content: "✅ 已复制到剪贴板".to_string(),
                        });
                    }
                    AppAction::None
                } else {
                    AppAction::Quit
                }
            }
            KeyCode::Enter => AppAction::SubmitChat,
            KeyCode::Backspace => {
                app.input_text.pop();
                app.command_hints.update_input(&app.input_text);
                AppAction::None
            }
            KeyCode::Up => {
                // 上键 - 滚动聊天历史向上
                if app.chat_scroll_offset < app.chat_history.get_messages().len().saturating_sub(1) {
                    app.chat_scroll_offset += 1;
                }
                AppAction::None
            }
            KeyCode::Down => {
                // 下键 - 滚动聊天历史向下
                if app.chat_scroll_offset > 0 {
                    app.chat_scroll_offset -= 1;
                }
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