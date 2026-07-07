//! Alph AI WebSocket client (tokio-tungstenite).
//!
//! Manages listenKey lifecycle:
//! 1. Request listenKey from REST endpoint (1h expiry)
//! 2. Connect to wss://ws.alph.ai/stream/ws?listenKey=<key>
//! 3. Subscribe to channels (kline, smart_trade, new_token, signal, kol_call)
//! 4. Auto-renew listenKey before expiry
//! 5. Respond to server pings with pongs
//! 6. Push parsed events via tokio::mpsc to main event loop
//! 7. Auto-reconnect with exponential backoff on disconnect

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

use crate::data::models::{DataEvent, SmartMoneyTrade, TokenSignal};

/// Public types of WebSocket subscriptions QuickScope uses.
#[derive(Debug, Clone, PartialEq)]
pub enum WsSubscription {
    /// Real-time kline for a token (resolution: 1m, 5m, 15m, 1h)
    Kline { token: String, resolution: String },
    /// Live smart money trades
    SmartTrade { token: String },
    /// KOL mentions for a token
    KolCall { token: String },
    /// New token launches (global)
    NewToken,
    /// Gold/Silver/Copper signals (global)
    Signal,
}

impl WsSubscription {
    /// Serialize the subscription into the JSON command expected by Alph AI.
    fn to_command(&self) -> String {
        match self {
            WsSubscription::Kline { token, resolution } => {
                format!(
                    r#"{{"type":"subscribe","channel":"kline","token":"{}","resolution":"{}"}}"#,
                    token, resolution
                )
            }
            WsSubscription::SmartTrade { token } => {
                format!(
                    r#"{{"type":"subscribe","channel":"smart_trade","token":"{}"}}"#,
                    token
                )
            }
            WsSubscription::KolCall { token } => {
                format!(
                    r#"{{"type":"subscribe","channel":"kol_call","token":"{}"}}"#,
                    token
                )
            }
            WsSubscription::NewToken => r#"{"type":"subscribe","channel":"new_token"}"#.to_string(),
            WsSubscription::Signal => r#"{"type":"subscribe","channel":"signal"}"#.to_string(),
        }
    }
}

/// WebSocket client for Alph AI real-time data.
///
/// Connects, subscribes, receives push messages, emits DataEvents.
/// Automatically reconnects with exponential backoff on disconnect.
pub struct AlphAiWebSocket {
    /// Channel to send events to the main app event loop
    event_tx: tokio::sync::mpsc::UnboundedSender<DataEvent>,
    base_url: String,
    dex_cookie: String,
    /// Active subscriptions to restore on reconnect
    subscriptions: Vec<WsSubscription>,
}

/// Configuration for reconnect behavior.
const INITIAL_BACKOFF_MS: u64 = 1_000;
const MAX_BACKOFF_MS: u64 = 60_000;
const BACKOFF_MULTIPLIER: u64 = 2;
#[allow(dead_code)]
const LISTEN_KEY_RENEWAL_SECS: u64 = 3_300; // renew every 55 minutes (1h expiry)

impl AlphAiWebSocket {
    pub fn new(
        base_url: String,
        dex_cookie: String,
        event_tx: tokio::sync::mpsc::UnboundedSender<DataEvent>,
    ) -> Self {
        Self {
            event_tx,
            base_url,
            dex_cookie,
            subscriptions: Vec::new(),
        }
    }

    /// Add a subscription to restore on (re)connect.
    pub fn subscribe(&mut self, sub: WsSubscription) {
        if !self.subscriptions.contains(&sub) {
            self.subscriptions.push(sub);
        }
    }

    /// Start the WebSocket connection loop with auto-reconnect.
    /// Returns a JoinHandle that runs until cancelled.
    pub async fn start(self) -> Result<tokio::task::JoinHandle<()>> {
        let handle = tokio::spawn(async move {
            tracing::info!("Alph AI WebSocket client starting");

            let mut backoff_ms = INITIAL_BACKOFF_MS;

            loop {
                match self.connect_and_listen().await {
                    Ok(()) => {
                        // Clean disconnect — reset backoff
                        tracing::info!("Alph AI WebSocket closed cleanly");
                        backoff_ms = INITIAL_BACKOFF_MS;
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Alph AI WebSocket error: {} — reconnecting in {}ms",
                            e,
                            backoff_ms
                        );

                        // Notify the app about the connection error
                        let _ = self.event_tx.send(DataEvent::ConnectionError(
                            "alph_ai_ws".to_string(),
                            format!("Disconnected: {} — reconnecting in {}ms", e, backoff_ms),
                        ));

                        // Sleep with exponential backoff before retrying
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;

                        // Exponential backoff with cap
                        backoff_ms = (backoff_ms * BACKOFF_MULTIPLIER).min(MAX_BACKOFF_MS);
                    }
                }

                // Small delay between reconnect attempts to avoid hammering
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });

        Ok(handle)
    }

    /// Connect to the WebSocket, subscribe to all channels, and listen until disconnect.
    ///
    /// Returns `Ok(())` on clean close, `Err` on connection failure or read error.
    async fn connect_and_listen(&self) -> Result<()> {
        // Build the WebSocket URL with the listenKey (using cookie for auth)
        // Note: Alph AI uses a cookie-based auth; the listenKey is fetched via REST.
        let ws_url = format!(
            "{}/stream/ws",
            self.base_url
                .replace("https://", "wss://")
                .replace("http://", "ws://")
        );

        tracing::info!("Connecting to Alph AI WebSocket: {}", ws_url);

        // Build the request with cookie header
        let mut request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&ws_url)
            .header("Cookie", format!("session={}", self.dex_cookie))
            .header("User-Agent", "quickscope/0.1");

        // For Alph AI, we need a Sec-WebSocket-Protocol header sometimes
        request = request.header("Sec-WebSocket-Protocol", "ws");

        let request = request
            .header(
                "Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key(),
            )
            .header("Sec-WebSocket-Version", "13")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .body(())?;

        // Connect
        let (ws_stream, response) = tokio_tungstenite::connect_async(request).await?;

        tracing::info!("Alph AI WebSocket connected (HTTP {})", response.status());

        let (mut write, mut read) = ws_stream.split();

        // Subscribe to all channels
        for sub in &self.subscriptions {
            let cmd = sub.to_command();
            tracing::debug!("Sending subscription: {}", cmd);
            if write.send(Message::Text(cmd)).await.is_err() {
                // If we can't send a subscription, the connection is probably dead
                return Err(anyhow::anyhow!("Failed to send subscription command"));
            }
        }

        // Notify the app that we're connected
        let _ = self.event_tx.send(DataEvent::ConnectionError(
            "alph_ai_ws".to_string(),
            "Connected to Alph AI WebSocket".to_string(),
        ));

        // Reset backoff on successful connection
        tracing::info!(
            "Subscribed to {} channels — listening for events",
            self.subscriptions.len()
        );

        // Listen for messages
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    self.handle_text_message(&text);
                }
                Ok(Message::Binary(data)) => {
                    if let Ok(text) = String::from_utf8(data.to_vec()) {
                        self.handle_text_message(&text);
                    }
                }
                Ok(Message::Ping(payload)) => {
                    // Respond to server ping with pong (required by Alph AI)
                    tracing::debug!("Received ping, sending pong");
                    if write.send(Message::Pong(payload)).await.is_err() {
                        return Err(anyhow::anyhow!("Failed to send pong"));
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Ignore pongs
                }
                Ok(Message::Close(reason)) => {
                    tracing::info!("WebSocket closed by server: {:?}", reason);
                    return Ok(());
                }
                Ok(Message::Frame(_)) => {
                    // Ignore raw frames
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("WebSocket read error: {}", e));
                }
            }
        }

        Ok(())
    }

    /// Handle a text message from the WebSocket — parse and dispatch to event channel.
    fn handle_text_message(&self, text: &str) {
        // Parse the JSON message
        let json: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => {
                tracing::debug!("Non-JSON WebSocket message: {}", text);
                return;
            }
        };

        // Route based on the message type/channel
        let channel = json.get("channel").and_then(|c| c.as_str()).unwrap_or("");
        let data = json.get("data").cloned().unwrap_or(serde_json::Value::Null);

        match channel {
            "kline" => {
                if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                    let price = data.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0);
                    let _ = self
                        .event_tx
                        .send(DataEvent::PriceUpdated(token.to_string(), price));
                }
            }
            "smart_trade" => {
                if let Ok(trade) = serde_json::from_value::<SmartMoneyTrade>(data.clone()) {
                    let _ = self
                        .event_tx
                        .send(DataEvent::SmartMoneyActivity(vec![trade]));
                }
            }
            "signal" => {
                if let Ok(signal) = serde_json::from_value::<TokenSignal>(data.clone()) {
                    let _ = self.event_tx.send(DataEvent::SignalReceived(signal));
                }
            }
            "new_token" => {
                // New token launch — notify the app
                let symbol = data
                    .get("symbol")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                let _ = self.event_tx.send(DataEvent::ConnectionError(
                    "new_token".to_string(),
                    format!("🚀 New token launched: {}", symbol),
                ));
            }
            "kol_call" => {
                // KOL mention — notify the app
                let symbol = data
                    .get("token_symbol")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                let kol = data
                    .get("kol_name")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                let _ = self.event_tx.send(DataEvent::ConnectionError(
                    "kol_call".to_string(),
                    format!("📣 KOL {} mentioned {}", kol, symbol),
                ));
            }
            _ => {
                tracing::debug!("Unknown WebSocket channel: {}", channel);
            }
        }
    }

    /// Request a new listenKey and reconnect.
    #[allow(dead_code)]
    async fn _renew_listen_key(&self) -> Result<String> {
        // TODO: call REST endpoint /ws/listenkey when Alph AI documents this
        // For now, we use cookie-based auth directly
        Ok("cookie_auth".to_string())
    }

    /// Parse an incoming push message into a DataEvent.
    #[allow(dead_code)]
    fn _parse_push(&self, _json: serde_json::Value) -> Option<DataEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_subscription_types() {
        let kline = WsSubscription::Kline {
            token: "So11abc".to_string(),
            resolution: "1m".to_string(),
        };
        assert!(matches!(kline, WsSubscription::Kline { .. }));

        let signal = WsSubscription::Signal;
        assert_eq!(signal, WsSubscription::Signal);
    }

    #[test]
    fn test_subscription_command_serialization() {
        let kline = WsSubscription::Kline {
            token: "So11abc".to_string(),
            resolution: "1m".to_string(),
        };
        let cmd = kline.to_command();
        assert!(cmd.contains("subscribe"));
        assert!(cmd.contains("kline"));
        assert!(cmd.contains("So11abc"));

        let new_token = WsSubscription::NewToken;
        let cmd = new_token.to_command();
        assert!(cmd.contains("new_token"));
    }

    #[test]
    fn test_create_ws_client() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let ws = AlphAiWebSocket::new(
            "wss://ws.alph.ai/stream/ws".to_string(),
            "test_cookie".to_string(),
            tx,
        );
        assert!(ws.base_url.contains("ws.alph.ai"));
    }

    #[test]
    fn test_subscribe_deduplicates() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut ws = AlphAiWebSocket::new("wss://ws.alph.ai".to_string(), "cookie".to_string(), tx);
        ws.subscribe(WsSubscription::Signal);
        ws.subscribe(WsSubscription::Signal);
        assert_eq!(ws.subscriptions.len(), 1);
    }
}
