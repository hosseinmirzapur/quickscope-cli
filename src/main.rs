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
use quickscope::data::{
    models::{AppCommand, AppEvent, DataEvent},
    orchestrator::DataOrchestrator,
};
use quickscope::storage::DbManager;
use quickscope::trade::TradeEngine;
use quickscope::ui;

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

    // ── Initialize services ─────────────────────────────────────
    let alph_dex_cookie = std::env::var("ALPH_DEX_COOKIE").unwrap_or_default();
    let orchestrator = Arc::new(DataOrchestrator::new(alph_dex_cookie));

    // Database path: use env var or default
    let db_path = std::env::var("QUICKSCOPE_DB_PATH").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/.config/quickscope/data.db", home)
    });
    let db = DbManager::new(&db_path).await?;
    let pool = db.pool.clone();

    // Load initial state from DB
    let mut state = AppState::new();
    if let Ok(config) = quickscope::storage::config::load_alpha_config(&pool).await {
        state.alpha_config = config;
    }
    if let Ok(_positions) = quickscope::storage::positions::get_open_positions(&pool).await {
        // Note: positions are PositionRows, not PaperPositions — mapping happens in Wave C
        state.open_positions.clear();
    }
    if let Ok(wl) = quickscope::storage::journal::get_watchlist(&pool).await {
        state.watchlist = wl;
    }
    if let Ok((balance, _)) = quickscope::storage::journal::get_portfolio(&pool).await {
        state.balance_sol = balance;
    }

    // Check for missing API keys on startup — show modal if critical ones are missing
    state.check_api_keys();

    let trade_engine = Arc::new(tokio::sync::Mutex::new(TradeEngine::new(db)));

    // Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let redraw = Arc::new(Notify::new());
    let (data_tx, mut data_rx) = tokio::sync::mpsc::unbounded_channel::<DataEvent>();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<Event>();

    // ── Background: event reader ─────────────────────────────────
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

    // ── Background: auto-refresh loop (10s interval) ─────────────
    {
        let orch = Arc::clone(&orchestrator);
        let tx = data_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                if let Ok(tokens) = orch.fetch_trending().await {
                    let _ = tx.send(DataEvent::TrendingUpdated(tokens));
                }
                if let Ok(trades) = orch.fetch_smart_money_trades(20).await {
                    let _ = tx.send(DataEvent::SmartMoneyActivity(trades));
                }
            }
        });
    }

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
                        dispatch_commands(cmds, &orchestrator, &trade_engine, &pool, &data_tx, &redraw).await;
                    }
                    Event::Mouse(mouse) => {
                        let cmds = app::update(&mut state, AppEvent::Mouse(mouse));
                        dispatch_commands(cmds, &orchestrator, &trade_engine, &pool, &data_tx, &redraw).await;
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

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

// ── Command dispatcher ──────────────────────────────────────────

async fn dispatch_commands(
    commands: Vec<AppCommand>,
    orchestrator: &Arc<DataOrchestrator>,
    trade_engine: &Arc<tokio::sync::Mutex<TradeEngine>>,
    pool: &sqlx::SqlitePool,
    data_tx: &tokio::sync::mpsc::UnboundedSender<DataEvent>,
    redraw: &Arc<Notify>,
) {
    for cmd in commands {
        let orch = Arc::clone(orchestrator);
        let engine = Arc::clone(trade_engine);
        let db_pool = pool.clone();
        let tx = data_tx.clone();
        let notify = redraw.clone();

        tokio::spawn(async move {
            let result = match cmd {
                // ── Fetch commands ───────────────────────────────
                AppCommand::FetchTrending => match orch.fetch_trending().await {
                    Ok(t) => {
                        let _ = tx.send(DataEvent::TrendingUpdated(t));
                        Ok(())
                    }
                    Err(e) => {
                        let _ =
                            tx.send(DataEvent::ConnectionError("trending".into(), e.to_string()));
                        Err(())
                    }
                },
                AppCommand::FetchTokenDetail(addr) => {
                    let a = addr.clone();
                    match orch.fetch_token_detail(&a).await {
                        Ok(d) => {
                            let _ = tx.send(DataEvent::TokenLoaded(Box::new(d)));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "token_detail".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }
                AppCommand::FetchKline(addr, res, from, to) => {
                    match orch.fetch_kline(&addr, &res, from, to).await {
                        Ok(c) => {
                            let _ = tx.send(DataEvent::KlineUpdated(addr, c));
                            Ok(())
                        }
                        Err(e) => {
                            let _ =
                                tx.send(DataEvent::ConnectionError("kline".into(), e.to_string()));
                            Err(())
                        }
                    }
                }
                AppCommand::FetchSmartMoney => match orch.fetch_smart_money_trades(20).await {
                    Ok(t) => {
                        let _ = tx.send(DataEvent::SmartMoneyActivity(t));
                        Ok(())
                    }
                    Err(e) => {
                        let _ = tx.send(DataEvent::ConnectionError(
                            "smart_money".into(),
                            e.to_string(),
                        ));
                        Err(())
                    }
                },
                AppCommand::FetchSignals => match orch.fetch_signals_gmgn().await {
                    Ok(signals) => {
                        for sig in signals {
                            let _ = tx.send(DataEvent::SignalReceived(sig));
                        }
                        Ok(())
                    }
                    Err(e) => {
                        let _ =
                            tx.send(DataEvent::ConnectionError("signals".into(), e.to_string()));
                        Err(())
                    }
                },
                AppCommand::FetchTrenches(tt) => match orch.fetch_trenches(&tt).await {
                    Ok(t) => {
                        let _ = tx.send(DataEvent::TrenchesUpdated(t));
                        Ok(())
                    }
                    Err(e) => {
                        let _ =
                            tx.send(DataEvent::ConnectionError("trenches".into(), e.to_string()));
                        Err(())
                    }
                },

                // ── Strategy data loading ──────────────────────────
                AppCommand::FetchAutoTuneHistory => {
                    match quickscope::storage::journal::get_recent_tuning_runs(&db_pool, 10).await {
                        Ok(runs) => {
                            let _ = tx.send(DataEvent::AutoTuneHistoryLoaded(runs));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "auto_tune_history".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }
                AppCommand::FetchPostMortemHistory => {
                    match quickscope::storage::journal::get_recent_post_mortems(&db_pool, 5).await {
                        Ok(mortems) => {
                            let _ = tx.send(DataEvent::PostMortemHistoryLoaded(mortems));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "post_mortem_history".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }

                // ── Trade commands ───────────────────────────────
                AppCommand::PaperBuy {
                    token_address,
                    amount_sol,
                    mode: _,
                    tp_percent,
                    sl_percent,
                } => {
                    let mut eng = engine.lock().await;
                    // Need token detail for trade — fetch it
                    match orch.fetch_token_detail(&token_address).await {
                        Ok(detail) => {
                            let config = quickscope::storage::config::load_alpha_config(&db_pool)
                                .await
                                .unwrap_or_default();
                            let report = quickscope::alpha::analyze_token(&detail, &config);
                            let (_tp, _sl) = (
                                tp_percent.unwrap_or_else(|| {
                                    quickscope::alpha::exit_params_for_mode(&report.mode).0
                                }),
                                sl_percent.unwrap_or_else(|| {
                                    quickscope::alpha::exit_params_for_mode(&report.mode).1
                                }),
                            );
                            match eng
                                .paper_buy(
                                    &detail,
                                    &config,
                                    amount_sol,
                                    detail.token.price_usd,
                                    150.0,
                                )
                                .await
                            {
                                Ok(result) => {
                                    let _ = tx.send(DataEvent::ConnectionError(
                                        "trade".into(),
                                        format!(
                                            "Paper buy executed: {} SOL → {} tokens at ${:.6}",
                                            amount_sol,
                                            result.tokens_received,
                                            result.effective_price
                                        ),
                                    ));
                                    Ok(())
                                }
                                Err(e) => {
                                    let _ = tx.send(DataEvent::ConnectionError(
                                        "paper_buy".into(),
                                        e.to_string(),
                                    ));
                                    Err(())
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "paper_buy".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }

                AppCommand::PaperSell {
                    position_id,
                    sell_percent,
                } => {
                    let mut eng = engine.lock().await;
                    match eng.paper_sell(&position_id, 0.0, sell_percent).await {
                        Ok(result) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "trade".into(),
                                format!(
                                    "Paper sell executed: {:.2}% → PnL {:.4} SOL ({:+.2}%)",
                                    sell_percent, result.pnl_sol, result.pnl_percent
                                ),
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "paper_sell".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }

                AppCommand::EmergencyExitAll => {
                    match quickscope::storage::positions::get_open_positions(&db_pool).await {
                        Ok(positions) => {
                            let mut eng = engine.lock().await;
                            for pos in &positions {
                                if let Err(e) = eng.paper_sell(&pos.id, 0.0, 100.0).await {
                                    let _ = tx.send(DataEvent::ConnectionError(
                                        "emergency_exit".into(),
                                        e.to_string(),
                                    ));
                                }
                            }
                            let _ = tx.send(DataEvent::ConnectionError(
                                "trade".into(),
                                format!("Emergency exit: {} positions closed", positions.len()),
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "emergency_exit".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }

                // ── Watchlist ────────────────────────────────────
                AppCommand::AddToWatchlist(addr) => {
                    let symbol = addr.chars().take(8).collect::<String>();
                    match quickscope::storage::journal::add_to_watchlist(&db_pool, &addr, &symbol)
                        .await
                    {
                        Ok(_) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "watchlist".into(),
                                format!("Added {} to watchlist", symbol),
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "watchlist".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }
                AppCommand::RemoveFromWatchlist(addr) => {
                    match quickscope::storage::journal::remove_from_watchlist(&db_pool, &addr).await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "watchlist".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }

                // ── Risk ─────────────────────────────────────────
                AppCommand::ToggleKillSwitch => {
                    let active = quickscope::storage::journal::get_today_risk(&db_pool, "today")
                        .await
                        .ok()
                        .flatten()
                        .map(|r| !r.kill_switch_active_bool())
                        .unwrap_or(true);
                    let _ = tx.send(DataEvent::ConnectionError(
                        "risk".into(),
                        if active {
                            "Kill switch ACTIVATED".into()
                        } else {
                            "Kill switch DEACTIVATED".into()
                        },
                    ));
                    Ok(())
                }

                // ── Learning ──────────────────────────────────────
                AppCommand::RunPostMortem(start, end) => {
                    let provider = quickscope::learning::LlmProvider::Stub {
                        model: "stub".to_string(),
                        response: "Post-mortem analysis:\n- Good trades: 0\n- Bad trades: 0\n- Suggestions: Set up an LLM API key for real analysis.".to_string(),
                    };
                    match quickscope::learning::journal::run_post_mortem(
                        &db_pool, &provider, &start, &end,
                    )
                    .await
                    {
                        Ok(resp) => {
                            let _ = tx.send(DataEvent::ConnectionError("post_mortem".into(), resp));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "post_mortem".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }
                AppCommand::RunAutoTune => {
                    match quickscope::learning::tuner::run_auto_tune(&db_pool).await {
                        Ok(Some(result)) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "auto_tune".into(),
                                format!(
                                    "Auto-tune complete: {}W/{}L, {} discriminations",
                                    result.wins,
                                    result.losses,
                                    result.discriminations.len()
                                ),
                            ));
                            Ok(())
                        }
                        Ok(None) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "auto_tune".into(),
                                "Auto-tune skipped: need 10+ wins AND 10+ losses".into(),
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "auto_tune".into(),
                                e.to_string(),
                            ));
                            Err(())
                        }
                    }
                }
                AppCommand::SaveAlphaConfig(config) => {
                    match quickscope::storage::config::save_alpha_config(&db_pool, &config).await {
                        Ok(_) => {
                            let _ = tx.send(DataEvent::ConnectionError(
                                "config".into(),
                                "Alpha config saved".into(),
                            ));
                            Ok(())
                        }
                        Err(e) => {
                            let _ =
                                tx.send(DataEvent::ConnectionError("config".into(), e.to_string()));
                            Err(())
                        }
                    }
                }

                // ── UI commands (handled synchronously) ──────────
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
