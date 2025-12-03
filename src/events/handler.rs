use crate::app::{App, AppAction, ModificationChoice};
use crate::ai::code_modification::{CodeModificationOp, CodeMatcher};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use crate::ui::pixel_layout_v2::extract_text_from_chat_area;

fn estimate_chat_lines(app: &App) -> usize {
    let mut total = 0;
    if app.chat_history.is_empty() && !app.is_streaming {
        return 20; // Estimate for welcome message
    }
    
    for msg in app.chat_history.get_messages() {
        // 3 lines overhead (header, footer, separator) + content lines
        total += 3 + msg.content.lines().count();
    }
    
    if app.is_streaming {
        if let Ok(response) = app.streaming_response.try_lock() {
             total += 5 + response.content.lines().count();
        } else {
             total += 10;
        }
    }
    
    total
}

pub struct EventHandler;

impl EventHandler {
    pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent, terminal_size: (u16, u16)) -> AppAction {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // 左键按下 - 开始选择
                app.selection_start = Some((mouse.column, mouse.row));
                app.selection_end = None;
                app.selected_text.clear();
                AppAction::None
            }
            MouseEventKind::Up(MouseButton::Left) => {
                // 左键释放 - 结束选择并复制到剪贴板
                if app.selection_start.is_some() {
                    app.selection_end = Some((mouse.column, mouse.row));

                    // 提取选中的文本
                    if let Ok(selected_text) = extract_text_from_chat_area(
                        app,
                        mouse.column,
                        mouse.row,
                        terminal_size.0,
                        terminal_size.1
                    ) {
                        if !selected_text.is_empty() {
                            app.selected_text = selected_text;

                            // 自动复制到剪贴板
                            if let Err(e) = Self::copy_to_clipboard(&app.selected_text) {
                                eprintln!("Failed to copy to clipboard: {}", e);
                            }
                        }
                    }
                }
                AppAction::None
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                // 拖动 - 更新选择范围
                if app.selection_start.is_some() {
                    app.selection_end = Some((mouse.column, mouse.row));
                }
                AppAction::None
            }
            MouseEventKind::ScrollUp => {
                // 鼠标滚轮向上 - 向上滚动聊天历史（看更早的消息）
                let max_scroll = estimate_chat_lines(app);
                if app.chat_scroll_offset < max_scroll {
                    app.chat_scroll_offset += 3;
                }
                AppAction::None
            }
            MouseEventKind::ScrollDown => {
                // 鼠标滚轮向下 - 向下滚动聊天历史（看更新的消息）
                if app.chat_scroll_offset > 0 {
                    app.chat_scroll_offset = app.chat_scroll_offset.saturating_sub(3);
                }
                AppAction::None
            }
            _ => AppAction::None,
        }
    }

    /// 复制文本到系统剪贴板
    fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text.to_string())?;
        Ok(())
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
                                CodeModificationOp::Modify { path, search, replace } => {
                                    // 修改文件 - 使用 CodeMatcher 进行模糊匹配
                                    match CodeMatcher::find_and_replace(&path, &search, &replace) {
                                        Ok(diff) => {
                                            match std::fs::write(path, diff.new_content) {
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
                                                content: format!("❌ 代码匹配失败: {}", e),
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
                    app.scroll_to_bottom();
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
                    app.scroll_to_bottom();
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
                    app.scroll_to_bottom();
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
                                                let new_content = content.replace(&content, &replace);
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
                    app.scroll_to_bottom(); // 滚动到底部显示最新消息
                    return AppAction::None;
                }
                _ => return AppAction::None,
            }
        }

        // 新的高优先级：处理文件名建议对话框
        if app.filename_suggestion.is_visible() {
            match key.code {
                KeyCode::Up => {
                    app.filename_suggestion.select_previous();
                    return AppAction::None;
                }
                KeyCode::Down => {
                    app.filename_suggestion.select_next();
                    return AppAction::None;
                }
                KeyCode::Enter => {
                    // 用户确认选择，创建文件
                    if let Some(filename) = app.filename_suggestion.get_selected() {
                        let code_content = app.filename_suggestion.get_code_content().to_string();

                        // 隐藏对话框
                        app.filename_suggestion.hide();

                        // 使用文件处理器创建文件
                        let result = app.file_command_handler.file_handler().write_file(&filename, &code_content);

                        // 显示结果
                        app.chat_history.add_message(crate::core::message::Message {
                            role: crate::core::message::Role::System,
                            content: result.message.clone(),
                        });

                        // 如果有备份信息，显示它
                        if let Some(backup_path) = result.backup_path {
                            app.chat_history.add_message(crate::core::message::Message {
                                role: crate::core::message::Role::System,
                                content: format!("💾 备份已创建: {}", backup_path.display()),
                            });
                        }

                        app.scroll_to_bottom();
                    }
                    return AppAction::None;
                }
                KeyCode::Esc => {
                    // 取消选择
                    app.filename_suggestion.hide();
                    app.chat_history.add_message(crate::core::message::Message {
                        role: crate::core::message::Role::System,
                        content: "❌ 已取消文件创建".to_string(),
                    });
                    app.scroll_to_bottom();
                    return AppAction::None;
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+C 也取消
                    app.filename_suggestion.hide();
                    return AppAction::None;
                }
                _ => return AppAction::None, // 在对话框显示时，其他按键无效
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
                        app.scroll_to_bottom();
                    }
                    AppAction::None
                } else {
                    AppAction::Quit
                }
            }
            KeyCode::Enter => {
                // Enter - 如果有提及建议被选中，则插入；否则提交聊天
                if app.mention_suggestions.visible {
                    if let Some(selected) = app.file_search.get_selected() {
                        // 替换 @ 后的内容为选中的文件路径
                        let at_pos = app.input_text.rfind('@').unwrap_or(0);
                        app.input_text.truncate(at_pos);
                        // 保留 @ 符号，添加文件路径和空格
                        app.input_text.push_str(&selected);
                        app.input_text.push(' '); // 添加空格，这样后续输入不会立即触发搜索
                        app.input_cursor = app.input_text.len(); // Move cursor to end
                        app.mention_suggestions.close();
                        app.file_search.clear();
                    }
                    AppAction::None
                } else {
                    AppAction::SubmitChat
                }
            }
            KeyCode::Backspace => {
                if app.input_cursor > 0 {
                    // 删除光标前的字符
                    let delete_char_index = app.input_cursor.saturating_sub(1);
                    
                    // 获取要删除的字符的字节位置
                    if let Some(byte_pos) = app.input_text
                        .char_indices()
                        .map(|(i, _)| i)
                        .nth(delete_char_index)
                    {
                        // 找到下一个字符的字节位置（用于确定删除范围）
                        let next_byte_pos = app.input_text
                            .char_indices()
                            .map(|(i, _)| i)
                            .nth(delete_char_index + 1)
                            .unwrap_or(app.input_text.len());
                        
                        // 删除该字符
                        app.input_text.drain(byte_pos..next_byte_pos);
                        app.input_cursor = delete_char_index;
                    }
                }
                
                // 自动调整输入框滚动位置（退格后）
                let total_lines = app.input_text.lines().count();
                let visible_lines = 3; // 输入框可见行数
                if total_lines > visible_lines {
                    app.input_scroll_offset = total_lines.saturating_sub(visible_lines);
                } else {
                    app.input_scroll_offset = 0;
                }
                
                // 如果提及建议可见，更新或关闭
                if app.mention_suggestions.visible {
                    if app.input_text.contains('@') {
                        // 使用文件搜索引擎更新
                        app.file_search.update_query(app.input_text.clone());
                        app.mention_suggestions.suggestions = app.file_search.results.clone();
                        app.mention_suggestions.selected_index = app.file_search.selected_index;
                        app.mention_suggestions.visible = !app.file_search.results.is_empty();
                    } else {
                        app.mention_suggestions.close();
                        app.file_search.clear();
                    }
                } else {
                    app.command_hints.update_input(&app.input_text);
                }
                
                AppAction::None
            }
            KeyCode::Up => {
                // 上键 - 如果提及建议可见，则导航；否则滚动聊天历史（看更早的消息）
                if app.mention_suggestions.visible {
                    app.file_search.select_previous();
                    app.mention_suggestions.selected_index = app.file_search.selected_index;
                } else if key.modifiers == KeyModifiers::CONTROL {
                    // Ctrl+Up: 向上滚动输入框
                    if app.input_scroll_offset > 0 {
                        app.input_scroll_offset -= 1;
                    }
                } else {
                    // 向上滚动：增加偏移量以查看更早的消息
                    let max_scroll = estimate_chat_lines(app);
                    if app.chat_scroll_offset < max_scroll {
                        app.chat_scroll_offset += 1;
                    }
                }
                AppAction::None
            }
            KeyCode::Down => {
                // 下键 - 如果提及建议可见，则导航；否则滚动聊天历史（看更新的消息）
                if app.mention_suggestions.visible {
                    app.file_search.select_next();
                    app.mention_suggestions.selected_index = app.file_search.selected_index;
                } else if key.modifiers == KeyModifiers::CONTROL {
                    // Ctrl+Down: 向下滚动输入框
                    let total_lines = app.input_text.lines().count();
                    let visible_lines = 3; // 输入框可见行数
                    let max_scroll = total_lines.saturating_sub(visible_lines);
                    if app.input_scroll_offset < max_scroll {
                        app.input_scroll_offset += 1;
                    }
                } else {
                    // 向下滚动：减少偏移量以查看更新的消息
                    if app.chat_scroll_offset > 0 {
                        app.chat_scroll_offset -= 1;
                    }
                }
                AppAction::None
            }
            KeyCode::PageUp => {
                // 向上翻页
                let max_scroll = estimate_chat_lines(app);
                if app.chat_scroll_offset < max_scroll {
                    app.chat_scroll_offset = app.chat_scroll_offset.saturating_add(10).min(max_scroll);
                }
                AppAction::None
            }
            KeyCode::Left => {
                // 使用字符索引移动光标
                app.input_cursor = app.input_cursor.saturating_sub(1);
                AppAction::None
            }
            KeyCode::Right => {
                // 使用字符索引移动光标
                let char_count = app.input_text.chars().count();
                app.input_cursor = (app.input_cursor + 1).min(char_count);
                AppAction::None
            }
            KeyCode::Char(c) if key.kind == KeyEventKind::Press => {
                // 只在按键按下时处理（过滤 IME 组合事件）
                // 将字符索引转换为字节索引，然后插入字符
                let char_count = app.input_text.chars().count();
                let byte_index = app.input_text
                    .char_indices()
                    .map(|(i, _)| i)
                    .nth(app.input_cursor.min(char_count))
                    .unwrap_or(app.input_text.len());
                
                app.input_text.insert(byte_index, c);
                app.input_cursor = (app.input_cursor + 1).min(char_count + 1);

                // 自动调整输入框滚动位置
                let total_lines = app.input_text.lines().count();
                let visible_lines = 3; // 输入框可见行数
                if total_lines > visible_lines {
                    app.input_scroll_offset = total_lines.saturating_sub(visible_lines);
                } else {
                    app.input_scroll_offset = 0;
                }

                // 检查最后一个 '@' 之后是否有空格
                if let Some(at_pos) = app.input_text.rfind('@') {
                    let after_at = &app.input_text[at_pos + 1..];
                    if after_at.contains(' ') {
                        // 如果@之后有空格，说明用户已经选完了，关闭建议
                        app.mention_suggestions.close();
                        app.file_search.clear();
                    } else {
                        // @之后没有空格，是正在输入，触发搜索
                        if !app.mention_suggestions.visible {
                            app.mention_suggestions.activate('@');
                        }
                        app.file_search.update_query(app.input_text.clone());
                        app.mention_suggestions.suggestions = app.file_search.results.clone();
                        app.mention_suggestions.selected_index = app.file_search.selected_index;
                        app.mention_suggestions.visible = !app.file_search.results.is_empty();
                    }
                } else {
                    // 没有@符号，处理普通命令提示
                    app.mention_suggestions.close();
                    app.file_search.clear();
                    app.command_hints.update_input(&app.input_text);
                }

                AppAction::None
            }
            _ => AppAction::None,
        }
    }
}





