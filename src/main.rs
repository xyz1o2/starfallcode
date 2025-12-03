mod app;
mod ui;
mod core;
mod ai;
mod events;
mod utils;
mod prompts;
mod commands;
mod tools;
mod fs;

use crate::app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app instance
    let mut app = App::new();

    // Set project root to current directory
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    app.file_search.set_root(current_dir);
    eprintln!("📁 Project root: {}", app.file_search.root_path.display());

    // Build file search cache at startup (like Gemini CLI's list_directory)
    // This ensures fast file lookups when user types @
    eprintln!("📁 Building file cache...");
    app.file_search.build_cache();
    eprintln!("✓ File cache built ({} files)", app.file_search.cache.len());

    // Initialize AI client from environment configuration
    match crate::ai::config::LLMConfig::from_env() {
        Ok(config) => {
            app.init_ai_client_with_config(config);
            eprintln!("✓ LLM client initialized successfully");
        }
        Err(e) => {
            eprintln!("⚠ Warning: Failed to load LLM configuration: {}", e);
            eprintln!("  Please check your .env file or environment variables");
            eprintln!("  See ENV_CONFIG.md for configuration instructions");
        }
    }

    // Initialize project context (optional)
    // app.init_project_context(".");

    // Run the application
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
        use crossterm::event::EventStream;
    use futures_util::StreamExt;
    use std::time::Duration;

    let mut reader = EventStream::new();
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            // 渲染 UI
            _ = interval.tick() => {
                terminal.draw(|f| {
                    app.render(f);
                })?;
            }

            // 处理终端事件 - 键盘和鼠标
            Some(Ok(event)) = reader.next() => {
                match event {
                    crossterm::event::Event::Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                            let action = crate::events::handler::EventHandler::handle_chat_event(app, key);
                            match action {
                                crate::app::AppAction::SubmitChat => {
                                    app.handle_chat_submit().await;
                                }
                                crate::app::AppAction::Quit => {
                                    return Ok(());
                                }
                                crate::app::AppAction::None => {}
                            }
                        }
                    }
                    crossterm::event::Event::Mouse(mouse) => {
                        let _action = crate::events::handler::EventHandler::handle_mouse_event(app, mouse);
                    }
                    _ => {}
                }
            }

            // 处理异步 LLM 响应
            maybe_stream_event = async {
                if let Some(handler) = app.stream_handler.as_mut() {
                    handler.get_receiver().lock().await.recv().await
                } else {
                    // 如果没有流处理器，我们可以使用 pending() 来避免这个分支被立即选择
                    std::future::pending().await
                }
            } => {
                if let Some(stream_event) = maybe_stream_event {
                    match stream_event {
                        crate::ai::streaming::StreamEvent::Token(t) => {
                            {
                                // 将 token 追加到聊天历史中最后一条 AI 消息
                                if let Some(last_msg) = app.chat_history.get_messages_mut().back_mut() {
                                    if let crate::core::message::Role::Assistant = last_msg.role {
                                        last_msg.content.push_str(&t);
                                    }
                                }
                            }
                            // 同步到 streaming_response（保持现有渲染逻辑兼容）
                            let mut streaming_response = app.streaming_response.lock().unwrap();
                            streaming_response.append(&t);
                            drop(streaming_response); // 释放锁
                            
                            // 保持自动滚动到底部
                            app.scroll_to_bottom();
                            
                            // 立即触发重新渲染以显示新的 token
                            terminal.draw(|f| app.render(f)).ok();
                        }
                        crate::ai::streaming::StreamEvent::Done => {
                            app.finalize_streaming_response().await;
                            // 最终渲染
                            terminal.draw(|f| app.render(f)).ok();
                        }
                        crate::ai::streaming::StreamEvent::Error(e) => {
                            eprintln!("Streaming Error: {}", e);
                            app.finalize_streaming_response().await;
                            terminal.draw(|f| app.render(f)).ok();
                        }
                    }
                }
            }
        }
    }
}