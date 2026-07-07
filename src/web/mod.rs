//! Web server module for QuickScope.
//! REST API + WebSocket for the Leptos frontend.

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod handlers;
mod state;
mod ws;

pub use state::WebState;

pub fn create_router(core: Arc<crate::core::AppCore>) -> Router {
    let state = WebState::new(core);

    Router::new()
        .route("/api/tokens/trending", get(handlers::get_trending))
        .route("/api/tokens/trenches", get(handlers::get_trenches))
        .route("/api/tokens/watchlist", get(handlers::get_watchlist))
        .route("/api/tokens/analyze/{address}", get(handlers::analyze_token))
        .route("/api/positions", get(handlers::get_positions))
        .route("/api/trade/buy", post(handlers::buy_paper))
        .route("/api/trade/sell", post(handlers::sell_paper))
        .route("/api/journal", get(handlers::get_journal))
        .route("/api/strategy", get(handlers::get_strategy).put(handlers::update_strategy))
        .route("/api/settings", get(handlers::get_settings).put(handlers::update_settings))
        .route("/ws", get(ws::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
