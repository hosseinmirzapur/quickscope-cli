use std::collections::HashMap;
use crate::data::models::*;
use crate::storage::journal::WatchlistRow;

/// Central application state — holds everything the UI needs to render.
pub struct AppState {
    /// Currently active tab
    pub active_tab: TabIndex,
    /// Whether the app is running
    pub running: bool,
    /// Terminal size
    pub terminal_size: (u16, u16),

    // Portfolio
    pub balance_sol: f64,
    pub daily_pnl: f64,

    // Data caches (populated by background fetchers)
    pub trending: Vec<TrendingToken>,
    pub open_positions: Vec<PaperPosition>,
    pub smart_money_feed: Vec<SmartMoneyTrade>,
    pub signals: Vec<TokenSignal>,
    pub tweets: Vec<Tweet>,
    pub trenches: Vec<TrenchToken>,

    // Storage-backed state
    pub trade_history: Vec<PaperPosition>,
    pub watchlist: Vec<WatchlistRow>,
    pub alpha_config: AlphaConfig,
    pub risk_state: RiskState,
    pub kline_cache: HashMap<String, Vec<KlineCandle>>,

    // Selected / focused items
    pub selected_token: Option<TokenDetail>,
    pub selected_position_id: Option<String>,
    pub alpha_report: Option<AlphaReport>,

    // Notification queue
    pub notifications: Vec<String>,

    // UI state
    pub theme_preset: ThemePreset,
    pub show_modal: bool,
    pub modal_message: String,
    pub notification: Option<String>,
    pub status_message: String,

    // Input state
    pub input_buffer: String,
    pub input_active: bool,
    pub scroll_offset: usize,
    pub list_cursor: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_tab: TabIndex::Dashboard,
            running: true,
            terminal_size: (80, 24),
            balance_sol: 50.0,
            daily_pnl: 0.0,
            trending: Vec::new(),
            open_positions: Vec::new(),
            smart_money_feed: Vec::new(),
            signals: Vec::new(),
            tweets: Vec::new(),
            trenches: Vec::new(),
            trade_history: Vec::new(),
            watchlist: Vec::new(),
            alpha_config: AlphaConfig::default(),
            risk_state: RiskState::default(),
            kline_cache: HashMap::new(),
            selected_token: None,
            selected_position_id: None,
            alpha_report: None,
            notifications: Vec::new(),
            theme_preset: ThemePreset::Dark,
            show_modal: false,
            modal_message: String::new(),
            notification: None,
            status_message: "Ready — press ? for help".to_string(),
            input_buffer: String::new(),
            input_active: false,
            scroll_offset: 0,
            list_cursor: 0,
        }
    }

    /// Switch to a tab
    pub fn switch_tab(&mut self, index: TabIndex) {
        self.active_tab = index;
        self.list_cursor = 0;
        self.scroll_offset = 0;
    }

    /// Move list cursor up/down
    pub fn move_cursor(&mut self, delta: isize) {
        let new = self.list_cursor as isize + delta;
        if new >= 0 {
            self.list_cursor = new as usize;
        }
    }

    /// Show a status message
    pub fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
    }

    /// Queue a notification (appears as toast overlay)
    pub fn notify(&mut self, msg: &str) {
        self.notifications.push(msg.to_string());
        self.notification = Some(msg.to_string());
    }

    /// Show a modal dialog
    pub fn show_modal(&mut self, msg: &str) {
        self.show_modal = true;
        self.modal_message = msg.to_string();
    }

    /// Dismiss modal
    pub fn dismiss_modal(&mut self) {
        self.show_modal = false;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}