//! API client for QuickScope backend.

use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde_json::Value;

const BASE_URL: &str = "http://127.0.0.1:3000/api";

async fn get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", BASE_URL, path);
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<T>()
        .await
        .map_err(|e| e.to_string())
}

async fn post<T: DeserializeOwned>(path: &str, body: &impl serde::Serialize) -> Result<T, String> {
    let url = format!("{}{}", BASE_URL, path);
    let json = serde_json::to_string(body).map_err(|e| e.to_string())?;
    Request::post(&url)
        .header("Content-Type", "application/json")
        .body(&json)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<T>()
        .await
        .map_err(|e| e.to_string())
}

// ── Token endpoints ─────────────────────────────────────────────

pub async fn fetch_trending() -> Result<Value, String> {
    get("/tokens/trending").await
}

pub async fn fetch_trenches() -> Result<Value, String> {
    get("/tokens/trenches").await
}

pub async fn fetch_watchlist() -> Result<Value, String> {
    get("/tokens/watchlist").await
}

pub async fn analyze_token(address: &str) -> Result<Value, String> {
    get(&format!("/tokens/analyze/{}", address)).await
}

// ── Position endpoints ──────────────────────────────────────────

pub async fn fetch_positions() -> Result<Value, String> {
    get("/positions").await
}

pub async fn buy_paper(
    token_address: &str,
    amount_sol: f64,
    mode: &str,
    tp_percent: Option<f64>,
    sl_percent: Option<f64>,
) -> Result<Value, String> {
    let body = serde_json::json!({
        "token_address": token_address,
        "amount_sol": amount_sol,
        "mode": mode,
        "tp_percent": tp_percent,
        "sl_percent": sl_percent,
    });
    post("/trade/buy", &body).await
}

pub async fn sell_paper(position_id: &str, sell_percent: f64) -> Result<Value, String> {
    let body = serde_json::json!({
        "position_id": position_id,
        "sell_percent": sell_percent,
    });
    post("/trade/sell", &body).await
}

// ── Journal ───────────────────────────────────────────────────────

pub async fn fetch_journal() -> Result<Value, String> {
    get("/journal").await
}

// ── Strategy ──────────────────────────────────────────────────────

pub async fn fetch_strategy() -> Result<Value, String> {
    get("/strategy").await
}

// ── Settings ──────────────────────────────────────────────────────

pub async fn fetch_settings() -> Result<Value, String> {
    get("/settings").await
}
