//! App core — state management, event handling, input dispatch.

pub mod input;
pub mod state;

pub use state::AppState;

use crate::data::models::*;
use input::{handle_key, handle_mouse, handle_resize};

/// Main update function — the "T" in TEA.
/// Takes current state + an event, returns commands for side effects.
pub fn update(state: &mut AppState, event: AppEvent) -> Vec<AppCommand> {
    match event {
        AppEvent::Key(key) => handle_key(key, state),
        AppEvent::Mouse(mouse) => handle_mouse(mouse, state),
        AppEvent::Resize(w, h) => {
            handle_resize(w, h, state);
            vec![]
        }
        AppEvent::Tick => {
            // Periodic tick — toast decay
            state.tick_toasts();
            vec![]
        }
        AppEvent::Data(data_event) => handle_data_event(state, *data_event),
    }
}

/// Handle async data events pushed from background tasks.
fn handle_data_event(state: &mut AppState, event: DataEvent) -> Vec<AppCommand> {
    match event {
        DataEvent::TrendingUpdated(tokens) => {
            state.loading_trending = false;
            state.trending = tokens;
            state.set_status(&format!("Loaded {} trending tokens", state.trending.len()));
        }
        DataEvent::TokenLoaded(detail) => {
            state.loading_token_detail = false;
            let symbol = detail.token.symbol.clone();
            state.selected_token = Some(*detail);
            state.set_status(&format!("Loaded {}", symbol));
            // Auto-switch to Analyzer tab
            if state.active_tab != TabIndex::Analyzer {
                state.switch_tab(TabIndex::Analyzer);
            }
        }
        DataEvent::KlineUpdated(_addr, candles) => {
            state.set_status(&format!("Kline: {} candles loaded", candles.len()));
        }
        DataEvent::SmartMoneyActivity(trades) => {
            state.smart_money_feed = trades;
        }
        DataEvent::SignalReceived(signal) => {
            state.notify(&format!(
                "🔔 Signal: {} ({})",
                signal.token_symbol,
                match signal.confidence {
                    SignalConfidence::Gold => "GOLD",
                    SignalConfidence::Silver => "SILVER",
                    _ => "COPPER",
                }
            ));
            state.signals.push(signal);
        }
        DataEvent::TwitterMention(_addr, tweet) => {
            state.tweets.push(tweet);
        }
        DataEvent::RateLimitHit(endpoint, reset_at) => {
            state.notify(&format!(
                "⚠️  Rate limit: {} (reset: {:?})",
                endpoint, reset_at
            ));
        }
        DataEvent::ConnectionError(endpoint, error) => {
            // Show errors visibly to the user — prefix with severity
            if error.contains("Connected") || error.contains("Loaded") || error.contains("complete")
            {
                state.notify(&format!("✓ {}", error));
            } else if endpoint == "alph_ai_ws" {
                // WebSocket status updates (info-level, don't spam)
                state.set_status(&error);
            } else {
                state.set_status(&format!("{}: {}", endpoint, error));
                state.notify(&format!("⚠ {}", error));
            }
        }
        DataEvent::PriceUpdated(_addr, price) => {
            // Update PnL for positions holding this token
            state.set_status(&format!("Price updated: ${:.6}", price));
        }
        DataEvent::TrenchesUpdated(tokens) => {
            let count = tokens.len();
            state.trenches = tokens;
            state.set_status(&format!("Loaded {} trenches", count));
        }
        DataEvent::WatchlistUpdated(tokens) => {
            state.trending = tokens; // For now, just replace trending with watchlist
            state.set_status("Watchlist loaded");
        }
        DataEvent::AutoTuneHistoryLoaded(runs) => {
            state.tuning_history = runs;
            state.set_status("Auto-tune history loaded");
        }
        DataEvent::PostMortemHistoryLoaded(mortems) => {
            state.post_mortems = mortems;
            state.set_status("Post-mortem history loaded");
        }
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = AppState::new();
        assert_eq!(state.active_tab, TabIndex::Dashboard);
        assert!(state.running);
        assert_eq!(state.balance_sol, 50.0);
    }

    #[test]
    fn test_tab_switching() {
        let mut state = AppState::new();
        state.switch_tab(TabIndex::Scanner);
        assert_eq!(state.active_tab, TabIndex::Scanner);
    }

    #[test]
    fn test_quit_on_q() {
        let mut state = AppState::new();
        let cmds = update(
            &mut state,
            AppEvent::Key(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('q'),
                crossterm::event::KeyModifiers::NONE,
            )),
        );
        assert!(!state.running);
        assert!(cmds.is_empty());
    }
}
