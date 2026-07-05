use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::sync::Arc;
use std::time::Duration;

use quickscope::{
    app::{self, AppState},
    data::{
        models::{AppCommand, AppEvent, DataEvent},
        orchestrator::DataOrchestrator,
    },
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

    // Load Alph AI cookie from environment (GMGN auth handled by gmgn-cli)
    let alph_dex_cookie = std::env::var("ALPH_DEX_COOKIE").unwrap_or_default();

    // Build the data orchestrator (shared across async tasks)
    let orchestrator = Arc::new(DataOrchestrator::new(alph_dex_cookie));

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut state = AppState::new();

    // Channel: background tasks → main loop
    let (data_tx, mut data_rx) = tokio::sync::mpsc::unbounded_channel::<DataEvent>();

    // Main event loop
    loop {
        if !state.running {
            break;
        }

        // Render
        terminal.draw(|frame| {
            ui::render_ui(frame, &state);
        })?;

        // Drain incoming data events (non-blocking)
        while let Ok(event) = data_rx.try_recv() {
            app::update(&mut state, AppEvent::Data(event));
        }

        // Poll for crossterm input
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    let cmds = app::update(&mut state, AppEvent::Key(key));
                    dispatch_commands(cmds, &orchestrator, &data_tx);
                }
                Event::Mouse(mouse) => {
                    let _ = app::update(&mut state, AppEvent::Mouse(mouse));
                }
                Event::Resize(w, h) => {
                    app::update(&mut state, AppEvent::Resize(w, h));
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

/// Execute AppCommands by spawning async background tasks.
/// Each task calls the orchestrator and pushes DataEvents back through the channel.
fn dispatch_commands(
    commands: Vec<AppCommand>,
    orchestrator: &Arc<DataOrchestrator>,
    data_tx: &tokio::sync::mpsc::UnboundedSender<DataEvent>,
) {
    for cmd in commands {
        let orch = Arc::clone(orchestrator);
        let tx = data_tx.clone();

        tokio::spawn(async move {
            match cmd {
                AppCommand::FetchTrending => {
                    match orch.fetch_trending().await {
                        Ok(tokens) => {
                            let count = tokens.len();
                            let _ = tx.send(DataEvent::TrendingUpdated(tokens));
                            tracing::info!("Fetched {} trending tokens", count);
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "fetch_trending".into(),
                                e.to_string(),
                            ));
                            tracing::error!("Failed to fetch trending: {}", e);
                        }
                    }
                }

                AppCommand::FetchTokenDetail(address) => {
                    match orch.fetch_token_detail(&address).await {
                        Ok(detail) => {
                            let symbol = detail.token.symbol.clone();
                            let _ = tx.send(DataEvent::TokenLoaded(detail));
                            tracing::info!("Fetched token detail for {}", symbol);
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "fetch_token_detail".into(),
                                e.to_string(),
                            ));
                            tracing::error!("Failed to fetch token detail: {}", e);
                        }
                    }
                }

                AppCommand::FetchKline(address, resolution, from, to) => {
                    match orch.fetch_kline(&address, &resolution, from, to).await {
                        Ok(candles) => {
                            let _ = tx.send(DataEvent::KlineUpdated(address, candles));
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "fetch_kline".into(),
                                e.to_string(),
                            ));
                            tracing::error!("Failed to fetch kline: {}", e);
                        }
                    }
                }

                AppCommand::ShowNotification(msg) => {
                    let _ = tx.send(DataEvent::ConnectionError("notification".into(), msg));
                }

                AppCommand::ShowModal(_msg) => {
                    // Modal is handled synchronously in AppState
                }

                AppCommand::SwitchTab(_) => {
                    // Handled synchronously in update()
                }

                AppCommand::EmergencyExitAll => {
                    tracing::warn!("Emergency exit all (paper positions)");
                    // TODO: iterate open positions and sell them
                }

                AppCommand::AddToWatchlist(_addr) => {
                    // TODO: storage-backed
                }

                AppCommand::RemoveFromWatchlist(_addr) => {
                    // TODO: storage-backed
                }

                AppCommand::ToggleKillSwitch => {
                    // TODO: toggle in storage
                }

                AppCommand::RunPostMortem(_start, _end) => {
                    // TODO: LLM post-mortem
                }

                AppCommand::RunAutoTune => {
                    // TODO: auto-tune
                }

                AppCommand::SaveAlphaConfig(_config) => {
                    // TODO: persist config
                }

                AppCommand::PaperBuy { .. } => {
                    // TODO: execute via TradeEngine
                }

                AppCommand::PaperSell { .. } => {
                    // TODO: execute via TradeEngine
                }
            }
        });
    }
}