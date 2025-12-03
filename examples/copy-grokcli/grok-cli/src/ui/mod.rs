use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    Terminal as RatatuiTerminal,
    widgets::{Block, Borders, Paragraph, List, ListItem},
    layout::{Layout, Direction, Constraint},
    style::{Style, Color},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent::GrokAgent;
use crate::types::{ChatEntry, ChatEntryType};

pub struct ChatState {
    chat_history: Vec<ChatEntry>,
    input: String,
    scroll: u16,
}

pub async fn run_app(mut agent: GrokAgent, initial_message: String) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = RatatuiTerminal::new(backend)?;

    let mut chat_state = ChatState {
        chat_history: vec![],
        input: String::new(),
        scroll: 0,
    };

    // If there's an initial message, process it first
    if !initial_message.trim().is_empty() {
        chat_state.chat_history.push(ChatEntry {
            entry_type: ChatEntryType::User,
            content: initial_message.clone(),
            timestamp: chrono::Utc::now(),
            tool_calls: None,
            tool_call: None,
            tool_result: None,
            is_streaming: None,
        });

        match agent.process_user_message(&initial_message).await {
            Ok(entries) => {
                chat_state.chat_history.extend(entries);
            }
            Err(e) => {
                chat_state.chat_history.push(ChatEntry {
                    entry_type: ChatEntryType::Assistant,
                    content: format!("Error: {}", e),
                    timestamp: chrono::Utc::now(),
                    tool_calls: None,
                    tool_call: None,
                    tool_result: None,
                    is_streaming: None,
                });
            }
        }
    }

    // Run the main UI loop
    let result = run_ui_loop(&mut terminal, &mut agent, &mut chat_state).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_ui_loop(
    terminal: &mut RatatuiTerminal<CrosstermBackend<std::io::Stdout>>,
    agent: &mut GrokAgent,
    state: &mut ChatState,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Draw UI
        terminal.draw(|f| {
            let size = f.size();

            // Create vertical layout: header, chat area, input
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(10),   // Chat history
                    Constraint::Length(3), // Input
                ])
                .split(size);

            // Header
            let header_block = Block::default()
                .borders(Borders::BOTTOM)
                .title("🤖 Grok CLI - Conversational AI Assistant");
            f.render_widget(header_block, chunks[0]);

            // Chat history
            let chat_items: Vec<ListItem> = state.chat_history.iter()
                .map(|entry| {
                    let content = match &entry.entry_type {
                        ChatEntryType::User => format!("👤 You: {}", entry.content),
                        ChatEntryType::Assistant => format!("🤖 Grok: {}", entry.content),
                        ChatEntryType::ToolResult => format!("🔧 Tool Result: {}", entry.content),
                        ChatEntryType::ToolCall => format!("🔧 Tool Call: {}", entry.content),
                    };

                    ListItem::new(content)
                        .style(match &entry.entry_type {
                            ChatEntryType::User => Style::default().fg(Color::Green),
                            ChatEntryType::Assistant => Style::default().fg(Color::Cyan),
                            ChatEntryType::ToolResult => Style::default().fg(Color::Yellow),
                            ChatEntryType::ToolCall => Style::default().fg(Color::Magenta),
                        })
                })
                .collect();

            let chat_list = List::new(chat_items)
                .block(Block::default().borders(Borders::TOP | Borders::BOTTOM));
            f.render_widget(chat_list, chunks[1]);

            // Input area
            let input_text = format!("> {}_", state.input);
            let input_paragraph = Paragraph::new(input_text)
                .block(Block::default().borders(Borders::TOP).title("Input"));
            f.render_widget(input_paragraph, chunks[2]);
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                // Only process Press events, ignore Release and Repeat
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' => {
                            return Ok(());
                        },
                        KeyCode::Char(c) => {
                            state.input.push(c);
                        },
                        KeyCode::Backspace => {
                            state.input.pop();
                        },
                        KeyCode::Enter => {
                            if !state.input.trim().is_empty() {
                                // Add user message to chat
                                let user_input = state.input.clone();
                                state.chat_history.push(ChatEntry {
                                    entry_type: ChatEntryType::User,
                                    content: user_input.clone(),
                                    timestamp: chrono::Utc::now(),
                                    tool_calls: None,
                                    tool_call: None,
                                    tool_result: None,
                                    is_streaming: None,
                                });

                                // Process with agent
                                let agent_response = agent.process_user_message(&user_input).await;
                                state.input.clear();

                                match agent_response {
                                    Ok(entries) => {
                                        state.chat_history.extend(entries);
                                    }
                                    Err(e) => {
                                        state.chat_history.push(ChatEntry {
                                            entry_type: ChatEntryType::Assistant,
                                            content: format!("Error: {}", e),
                                            timestamp: chrono::Utc::now(),
                                            tool_calls: None,
                                            tool_call: None,
                                            tool_result: None,
                                            is_streaming: None,
                                        });
                                    }
                                }
                            }
                        },
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }
}