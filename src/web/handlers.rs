//! REST API handlers for all endpoints.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::data::models::*;

use super::state::WebState;

// ── Request types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BuyRequest {
    pub token_address: String,
    pub amount_sol: f64,
    pub mode: String,
    #[serde(default)]
    pub tp_percent: Option<f64>,
    #[serde(default)]
    pub sl_percent: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct SellRequest {
    pub position_id: String,
    #[serde(default = "default_sell_percent")]
    pub sell_percent: f64,
}

fn default_sell_percent() -> f64 {
    100.0
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StrategyUpdate {
    pub daily_loss_cap: Option<f64>,
    pub per_trade_risk: Option<f64>,
    pub mode_thresholds: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SettingsUpdate {
    pub theme: Option<String>,
    pub log_level: Option<String>,
}

// ── Response helper ────────────────────────────────────────────────────

type ApiResult<T> = Result<Json<T>, (StatusCode, String)>;

fn internal_err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn bad_request(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, e.to_string())
}

// ── Handlers ───────────────────────────────────────────────────────────

pub async fn get_trending(State(state): State<WebState>) -> ApiResult<serde_json::Value> {
    match state.core.fetch_trending().await {
        Ok(tokens) => Ok(Json(json!({ "tokens": tokens }))),
        Err(e) => Err(internal_err(e)),
    }
}

pub async fn get_trenches(State(state): State<WebState>) -> ApiResult<serde_json::Value> {
    match state.core.fetch_trenches("1h").await {
        Ok(tokens) => Ok(Json(json!({ "tokens": tokens }))),
        Err(e) => Err(internal_err(e)),
    }
}

pub async fn get_watchlist(State(state): State<WebState>) -> ApiResult<serde_json::Value> {
    match state.core.get_watchlist().await {
        Ok(rows) => Ok(Json(json!({ "watchlist": rows }))),
        Err(e) => Err(internal_err(e)),
    }
}

pub async fn analyze_token(
    Path(address): Path<String>,
    State(state): State<WebState>,
) -> ApiResult<serde_json::Value> {
    let config = state.core.get_alpha_config().await.map_err(internal_err)?;
    match state.core.analyze_token(&address, &config).await {
        Ok((detail, report)) => Ok(Json(json!({
            "detail": detail,
            "report": report,
        }))),
        Err(e) => Err(internal_err(e)),
    }
}

pub async fn get_positions(State(state): State<WebState>) -> ApiResult<serde_json::Value> {
    match state.core.get_open_positions().await {
        Ok(positions) => Ok(Json(json!({ "positions": positions }))),
        Err(e) => Err(internal_err(e)),
    }
}

pub async fn buy_paper(
    State(state): State<WebState>,
    Json(payload): Json<BuyRequest>,
) -> ApiResult<serde_json::Value> {
    let mode = match payload.mode.to_uppercase().as_str() {
        "EXPLODE" => TradeMode::Explode,
        "ALPHA" => TradeMode::Alpha,
        "SCALP" => TradeMode::Scalp,
        _ => TradeMode::Fallback,
    };

    match state.core.paper_buy(&payload.token_address, payload.amount_sol, mode, payload.tp_percent, payload.sl_percent).await {
        Ok(result) => {
            let _ = state.broadcast.send(DataEvent::ConnectionError(
                "trade".into(),
                format!("Paper buy: {} SOL", payload.amount_sol),
            ));
            Ok(Json(json!({
                "success": true,
                "tokens_received": result.tokens_received,
                "effective_price": result.effective_price
            })))
        }
        Err(e) => Err(bad_request(e)),
    }
}

pub async fn sell_paper(
    State(state): State<WebState>,
    Json(payload): Json<SellRequest>,
) -> ApiResult<serde_json::Value> {
    match state.core.paper_sell(&payload.position_id, 0.0, payload.sell_percent).await {
        Ok(result) => {
            let _ = state.broadcast.send(DataEvent::ConnectionError(
                "trade".into(),
                format!("Paper sell: {:.2}%", payload.sell_percent),
            ));
            Ok(Json(json!({
                "success": true,
                "pnl_sol": result.pnl_sol,
                "pnl_percent": result.pnl_percent
            })))
        }
        Err(e) => Err(bad_request(e)),
    }
}

pub async fn get_journal(State(state): State<WebState>) -> ApiResult<serde_json::Value> {
    match state.core.get_closed_positions(None, None).await {
        Ok(entries) => Ok(Json(json!({ "journal": entries }))),
        Err(e) => Err(internal_err(e)),
    }
}

pub async fn get_strategy(State(state): State<WebState>) -> ApiResult<serde_json::Value> {
    let config = state.core.get_alpha_config().await.map_err(internal_err)?;
    Ok(Json(json!({
        "strategy": config,
        "daily_loss_cap": null,
        "per_trade_risk": null
    })))
}

pub async fn update_strategy(
    State(_state): State<WebState>,
    Json(_payload): Json<StrategyUpdate>,
) -> ApiResult<serde_json::Value> {
    // Currently only alpha config is persisted; risk settings to be added
    Ok(Json(json!({ "success": true })))
}

pub async fn get_settings(State(state): State<WebState>) -> ApiResult<serde_json::Value> {
    Ok(Json(json!({
        "theme": "dark",
        "log_level": state.core.config.log_level,
        "api_keys": {
            "alph_dex": if state.core.config.has_critical_keys() { "configured" } else { "missing" },
            "openai": if state.core.config.openai_api_key.as_ref().is_some_and(|k| !k.is_empty()) { "configured" } else { "missing" },
            "anthropic": if state.core.config.anthropic_api_key.as_ref().is_some_and(|k| !k.is_empty()) { "configured" } else { "missing" },
            "ollama": if state.core.config.ollama_base_url.as_ref().is_some_and(|k| !k.is_empty()) { "configured" } else { "missing" },
        }
    })))
}

pub async fn update_settings(
    State(_state): State<WebState>,
    Json(_payload): Json<SettingsUpdate>,
) -> ApiResult<serde_json::Value> {
    Ok(Json(json!({ "success": true })))
}
