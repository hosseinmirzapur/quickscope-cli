use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::Duration;

use quickscope::{
    app::{self, AppState},
    data::models::{AppEvent, DataEvent},
    ui,
};

#[derive(Parser)]
#[command(name = "quickscope", about = "Solana memecoin alpha hunting TUI")]
struct Cli {
    #[arg(short, long, env = "QUICKSCOPE_CONFIG")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let _cli = Cli::parse();

    dotenvy::dotenv().ok();

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut state = AppState::new();

    // Channel for background data events
    let (data_tx, mut data_rx) = tokio::sync::mpsc::unbounded_channel::<DataEvent>();

    // Spawn demo bg task
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let _ = data_tx.send(DataEvent::TrendingUpdated(vec![]));
    });

    // Main event loop
    loop {
        if !state.running {
            break;
        }

        // Render
        terminal.draw(|frame| {
            ui::render_ui(frame, &state);
        })?;

        // Check for data events (non-blocking)
        while let Ok(event) = data_rx.try_recv() {
            let _ = app::update(&mut state, AppEvent::Data(event));
        }

        // Poll for crossterm input (with timeout for smooth rendering)
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    let _ = app::update(&mut state, AppEvent::Key(key));
                }
                Event::Mouse(mouse) => {
                    let _ = app::update(&mut state, AppEvent::Mouse(mouse));
                }
                Event::Resize(w, h) => {
                    let _ = app::update(&mut state, AppEvent::Resize(w, h));
                }
                _ => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    Ok(())
}