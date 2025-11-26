mod app;
mod ui;
mod core;
mod ai;
mod events;
mod utils;

use crate::app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
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
    loop {
        terminal.draw(|f| ui::render_modern_ui(f, app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = crossterm::event::read()? {
                let should_continue = crate::events::handler::EventHandler::handle_chat_event(app, key);
                if !should_continue {
                    return Ok(());
                }
            }
        }
    }
}