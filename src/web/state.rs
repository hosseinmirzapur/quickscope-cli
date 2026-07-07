//! Shared web state for the Axum server.

use std::sync::Arc;
use tokio::sync::broadcast;

use crate::core::AppCore;
use crate::data::models::DataEvent;

#[derive(Clone)]
pub struct WebState {
    pub core: Arc<AppCore>,
    pub broadcast: broadcast::Sender<DataEvent>,
}

impl WebState {
    pub fn new(core: Arc<AppCore>) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            core,
            broadcast: tx,
        }
    }
}
