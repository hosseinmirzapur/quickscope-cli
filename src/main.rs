use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

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

    let alph_dex_cookie = std::env::var("ALPH_DEX_COOKIE").unwrap_or_default();
    let orchestrator = Arc::new(DataOrchestrator::new(alph_dex_cookie));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut state = AppState::new();
    let redraw = Arc::new(Notify::new());

    // Channels
    let (data_tx, mut data_rx) = tokio::sync::mpsc::unbounded_channel::<DataEvent>();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<Event>();

    // ── Background: crossterm event reader ──────────────────────
    tokio::spawn(async move {
        loop {
            if crossterm::event::poll(Duration::from_millis(16)).unwrap_or(false) {
                if let Ok(event) = crossterm::event::read() {
                    if event_tx.send(event).is_err() {
                        break;
                    }
                }
            }
            tokio::task::yield_now().await;
        }
    });

    // ── Main loop ───────────────────────────────────────────────
    loop {
        if !state.running {
            break;
        }

        terminal.draw(|frame| {
            ui::render_ui(frame, &state);
        })?;

        tokio::select! {
            biased;

            Some(event) = event_rx.recv() => {
                match event {
                    Event::Key(key) => {
                        let cmds = app::update(&mut state, AppEvent::Key(key));
                        dispatch_commands(cmds, &orchestrator, &data_tx, &redraw);
                    }
                    Event::Mouse(mouse) => {
                        let cmds = app::update(&mut state, AppEvent::Mouse(mouse));
                        dispatch_commands(cmds, &orchestrator, &data_tx, &redraw);
                    }
                    Event::Resize(w, h) => {
                        app::update(&mut state, AppEvent::Resize(w, h));
                    }
                    _ => {}
                }
            }

            _ = redraw.notified() => {
                // Data arrived — drain it and re-render
            }

            _ = tokio::time::sleep(Duration::from_millis(32)) => {}
        }

        // Drain any pending data events before next frame
        while let Ok(event) = data_rx.try_recv() {
            app::update(&mut state, AppEvent::Data(event));
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

// ── Command dispatcher ──────────────────────────────────────────

fn dispatch_commands(
    commands: Vec<AppCommand>,
    orchestrator: &Arc<DataOrchestrator>,
    data_tx: &tokio::sync::mpsc::UnboundedSender<DataEvent>,
    redraw: &Arc<Notify>,
) {
    for cmd in commands {
        let orch = Arc::clone(orchestrator);
        let tx = data_tx.clone();
        let notify = redraw.clone();

        tokio::spawn(async move {
            let result = match cmd {
                AppCommand::FetchTrending => {
                    match orch.fetch_trending().await {
                        Ok(tokens) => { let _ = tx.send(DataEvent::TrendingUpdated(tokens)); Ok(()) }
                        Err(e) => { let _ = tx.send(DataEvent::ConnectionError("fetch_trending".into(), e.to_string())); Err(()) }
                    }
                }

                AppCommand::FetchTokenDetail(address) => {
                    match orch.fetch_token_detail(&address).await {
                        Ok(detail) => { let _ = tx.send(DataEvent::TokenLoaded(detail)); Ok(()) }
                        Err(e) => { let _ = tx.send(DataEvent::ConnectionError("fetch_token_detail".into(), e.to_string())); Err(()) }
                    }
                }

                AppCommand::FetchKline(address, resolution, from, to) => {
                    match orch.fetch_kline(&address, &resolution, from, to).await {
                        Ok(candles) => { let _ = tx.send(DataEvent::KlineUpdated(address, candles)); Ok(()) }
                        Err(e) => { let _ = tx.send(DataEvent::ConnectionError("fetch_kline".into(), e.to_string())); Err(()) }
                    }
                }

                AppCommand::FetchSmartMoney => {
                    match orch.fetch_smart_money_trades(20).await {
                        Ok(trades) => { let _ = tx.send(DataEvent::SmartMoneyActivity(trades)); Ok(()) }
                        Err(e) => { let _ = tx.send(DataEvent::ConnectionError("fetch_smart_money".into(), e.to_string())); Err(()) }
                    }
                }

                AppCommand::FetchSignals => {
                    match orch.fetch_signals_gmgn().await {
                        Ok(signals) => { for sig in signals { let _ = tx.send(DataEvent::SignalReceived(sig)); } Ok(()) }
                        Err(e) => { let _ = tx.send(DataEvent::ConnectionError("fetch_signals".into(), e.to_string())); Err(()) }
                    }
                }

                AppCommand::FetchTrenches(token_type) => {
                    match orch.fetch_trenches(&token_type).await {
                        Ok(tokens) => { let _ = tx.send(DataEvent::TrenchesUpdated(tokens)); Ok(()) }
                        Err(e) => { let _ = tx.send(DataEvent::ConnectionError("fetch_trenches".into(), e.to_string())); Err(()) }
                    }
                }

                _ => {
                    tracing::warn!("{:?} not yet wired — coming in Wave B", cmd);
                    Ok(())
                }
            };
            if result.is_ok() {
                notify.notify_one();
            }
        });
    }
}