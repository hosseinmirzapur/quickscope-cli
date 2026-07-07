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

use quickscope::app;
use quickscope::app::AppState;
use quickscope::core::{AppConfig, AppCore};
use quickscope::data::models::{AppCommand, AppEvent, DataEvent};
use quickscope::ui;

#[derive(Parser)]
#[command(name = "quickscope", about = "Solana memecoin alpha hunting TUI & Web")]
struct Cli {
    #[arg(short, long, env = "QUICKSCOPE_CONFIG")]
    config: Option<String>,
    #[arg(long, default_value_t = false)]
    web: bool,
    #[arg(long, default_value_t = 3000)]
    port: u16,
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let cli = Cli::parse();
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env();
    let core = AppCore::new(config).await?;

    if cli.web {
        return run_web_server(core, cli.host, cli.port).await;
    }

    run_tui(core).await
}

async fn run_tui(core: AppCore) -> Result<()> {

    // ── Load initial state ──────────────────────────────────────────
    let mut state = AppState::new();
    if let Ok(ref config) = core.get_alpha_config().await {
        state.alpha_config = config.clone();
    }
    if let Ok(pos) = core.get_open_positions().await {
        state.open_positions.clear();
        drop(pos);
    }
    if let Ok(wl) = core.get_watchlist().await {
        state.watchlist = wl;
    }
    if let Ok((balance, _)) = core.get_portfolio().await {
        state.balance_sol = balance;
    }

    state.check_api_keys();

    // ── Terminal setup ──────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let redraw = Arc::new(Notify::new());
    let (data_tx, mut data_rx) = tokio::sync::mpsc::unbounded_channel::<DataEvent>();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<Event>();

    // ── Background: event reader ────────────────────────────────────
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

    // ── Background: auto-refresh loop (10s) ────────────────────────
    {
        let core_clone = core.clone();
        let tx = data_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                if let Ok(tokens) = core_clone.fetch_trending().await {
                    let _ = tx.send(DataEvent::TrendingUpdated(tokens));
                }
                if let Ok(trades) = core_clone.fetch_smart_money(20).await {
                    let _ = tx.send(DataEvent::SmartMoneyActivity(trades));
                }
            }
        });
    }

    // ── Main loop ───────────────────────────────────────────────────
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
                        dispatch_commands(cmds, &core, &data_tx, &redraw).await;
                    }
                    Event::Mouse(mouse) => {
                        let cmds = app::update(&mut state, AppEvent::Mouse(mouse));
                        dispatch_commands(cmds, &core, &data_tx, &redraw).await;
                    }
                    Event::Resize(w, h) => { app::update(&mut state, AppEvent::Resize(w, h)); }
                    _ => {}
                }
            }
            _ = redraw.notified() => {}
            _ = tokio::time::sleep(Duration::from_millis(32)) => {}
        }

        while let Ok(event) = data_rx.try_recv() {
            app::update(&mut state, AppEvent::Data(Box::new(event)));
        }
    }

    // ── Cleanup ─────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_commands(
    commands: Vec<AppCommand>,
    core: &AppCore,
    data_tx: &tokio::sync::mpsc::UnboundedSender<DataEvent>,
    redraw: &Arc<Notify>,
) {
    for cmd in commands {
        let core_clone = core.clone();
        let tx = data_tx.clone();
        let notify = redraw.clone();

        tokio::spawn(async move {
            let result = match cmd {
                // ── Fetch commands ──────────────────────────────────────
                AppCommand::FetchTrending => match core_clone.fetch_trending().await {
                    Ok(t) => {
                        let _ = tx.send(DataEvent::TrendingUpdated(t));
                        Ok(())
                    }
                    Err(e) => {
                        let _ = tx.send(DataEvent::ConnectionError("trending".into(), e.to_string()));
                        Err(())
                    }
                },
                AppCommand::FetchTokenDetail(addr) => {
                    match core_clone.fetch_token_detail(&addr).await {
                        Ok(d) => {
                            let _ = tx.send(DataEvent::TokenLoaded(Box::new(d)));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("token_detail".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::FetchKline(addr, res, from, to) => {
                    match core_clone.fetch_kline(&addr, &res, from, to).await {
                        Ok(c) => {
                            let _ = tx.send(DataEvent::KlineUpdated(addr, c));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("kline".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::FetchSmartMoney => {
                    match core_clone.fetch_smart_money(20).await {
                        Ok(t) => {
                            let _ = tx.send(DataEvent::SmartMoneyActivity(t));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("smart_money".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::FetchSignals => {
                    match core_clone.fetch_signals().await {
                        Ok(signals) => {
                            for sig in signals {
                                let _ = tx.send(DataEvent::SignalReceived(sig));
                            }
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("signals".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::FetchTrenches(time_token) => {
                    match core_clone.fetch_trenches(&time_token).await {
                        Ok(t) => {
                            let _ = tx.send(DataEvent::TrenchesUpdated(t));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("trenches".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::FetchAutoTuneHistory => {
                    match core_clone.get_tuning_history(10).await {
                        Ok(runs) => {
                            let _ = tx.send(DataEvent::AutoTuneHistoryLoaded(runs));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("auto_tune_history".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::FetchPostMortemHistory => {
                    match core_clone.get_post_mortems(5).await {
                        Ok(mortems) => {
                            let _ = tx.send(DataEvent::PostMortemHistoryLoaded(mortems));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("post_mortem_history".into(), e.to_string()));
                            Err(())
                        }
                    }
                }

                // ── Trade commands ─────────────────────────────────────
                AppCommand::PaperBuy { token_address, amount_sol, mode, tp_percent, sl_percent } => {
                    match core_clone.paper_buy(&token_address, amount_sol, mode, tp_percent, sl_percent).await {
                        Ok(result) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "trade".into(),
                                format!("Paper buy executed: {} SOL → {} tokens", amount_sol, result.tokens_received),
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("paper_buy".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::PaperSell { position_id, sell_percent } => {
                    match core_clone.paper_sell(&position_id, 0.0, sell_percent).await {
                        Ok(result) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "trade".into(),
                                format!("Paper sell: {:.2}% → PnL {:.4} SOL", sell_percent, result.pnl_sol),
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("paper_sell".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::EmergencyExitAll => {
                    match core_clone.emergency_exit_all().await {
                        Ok(count) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "trade".into(),
                                format!("Emergency exit: {} positions closed", count),
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("emergency_exit".into(), e.to_string()));
                            Err(())
                        }
                    }
                }

                // ── Watchlist ──────────────────────────────────────────
                AppCommand::AddToWatchlist(addr) => {
                    let symbol = addr.chars().take(8).collect::<String>();
                    match core_clone.add_to_watchlist(&addr, &symbol).await {
                        Ok(_) => {
                            let _ = tx.send(DataEvent::ConnectionError("watchlist".into(),
                                format!("Added {} to watchlist", symbol)
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("watchlist".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::RemoveFromWatchlist(addr) => {
                    match core_clone.remove_from_watchlist(&addr).await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("watchlist".into(), e.to_string()));
                            Err(())
                        }
                    }
                }

                // ── Risk ───────────────────────────────────────────────
                AppCommand::ToggleKillSwitch => {
                    match core_clone.toggle_kill_switch().await {
                        Ok(active) => {
                            let _ = tx.send(DataEvent::ConnectionError("risk".into(),
                                if active { "Kill switch ACTIVATED".into() } else { "Kill switch DEACTIVATED".into() }
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("risk".into(), e.to_string()));
                            Err(())
                        }
                    }
                }

                // ── Learning ──────────────────────────────────────────
                AppCommand::RunPostMortem(start, end) => {
                    match core_clone.run_post_mortem(&start, &end).await {
                        Ok(resp) => {
                            let _ = tx.send(DataEvent::ConnectionError("post_mortem".into(), resp));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("post_mortem".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::RunAutoTune => {
                    match core_clone.run_auto_tune().await {
                        Ok(Some(result)) => {
                            let _ = tx.send(DataEvent::ConnectionError("auto_tune".into(),
                                format!("Auto-tune: {}W/{}L, {} discriminations", result.wins, result.losses, result.discriminations.len())
                            ));
                            Ok(())
                        }
                        Ok(None) => {
                            let _ = tx.send(DataEvent::ConnectionError("auto_tune".into(),
                                "Auto-tune skipped: need 10+ wins AND 10+ losses".into()
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("auto_tune".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::SaveAlphaConfig(cfg) => {
                    match core_clone.save_alpha_config(&cfg).await {
                        Ok(_) => {
                            let _ = tx.send(DataEvent::ConnectionError("config".into(), "Alpha config saved".into()));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError("config".into(), e.to_string()));
                            Err(())
                        }
                    }
                }

                // ── UI commands (handled synchronously) ──────────────
                AppCommand::ShowNotification(msg) => {
                    let _ = tx.send(DataEvent::ConnectionError("notification".into(), msg));
                    Ok(())
                }
                AppCommand::ShowModal(_) | AppCommand::SwitchTab(_) => Ok(()),
            };

            if result.is_ok() {
                notify.notify_one();
            }
        });
    }
}

async fn run_web_server(core: AppCore, host: String, port: u16) -> Result<()> {
    use axum::serve;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    use quickscope::web::create_router;

    let core = Arc::new(core);
    let app = create_router(core);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    println!("🌐 QuickScope Web UI available at http://{}", addr);

    let listener = TcpListener::bind(&addr).await?;
    serve(listener, app).await?;

    Ok(())
}
