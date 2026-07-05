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
    pub active_toast: Option<String>,
    pub toast_remaining_ms: u32,

    // UI state
    pub theme_preset: ThemePreset,
    pub show_modal: bool,
    pub modal_message: String,
    pub show_command_palette: bool,
    pub palette_filter: String,
    pub palette_cursor: usize,
    pub sidebar_collapsed: bool,
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
            active_toast: None,
            toast_remaining_ms: 0,
            theme_preset: ThemePreset::Dark,
            show_modal: false,
            modal_message: String::new(),
            show_command_palette: false,
            palette_filter: String::new(),
            palette_cursor: 0,
            sidebar_collapsed: false,
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

    /// Move list cursor up/down with bounds clamping
    pub fn move_cursor(&mut self, delta: isize) {
        let max_len = self.trending.len().max(1).saturating_sub(1);
        let new = self.list_cursor as isize + delta;
        if new >= 0 {
            self.list_cursor = (new as usize).min(max_len);
        } else {
            self.list_cursor = 0;
        }
        // Auto-scroll: adjust scroll_offset so cursor is visible
        let visible = self.terminal_size.1.saturating_sub(6) as usize; // rough visible rows
        if self.list_cursor < self.scroll_offset {
            self.scroll_offset = self.list_cursor;
        } else if self.list_cursor >= self.scroll_offset + visible {
            self.scroll_offset = self.list_cursor.saturating_sub(visible) + 1;
        }
    }

    /// Show a status message
    pub fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
    }

    /// Queue a notification (appears as toast overlay)
    pub fn notify(&mut self, msg: &str) {
        self.notifications.push(msg.to_string());
        self.active_toast = Some(msg.to_string());
        self.toast_remaining_ms = 4000;
        self.notification = Some(msg.to_string());
    }

    /// Tick toasts down each frame
    pub fn tick_toasts(&mut self) {
        if self.active_toast.is_some() {
            if self.toast_remaining_ms > 33 {
                self.toast_remaining_ms = self.toast_remaining_ms.saturating_sub(33);
            } else {
                self.active_toast = None;
                self.toast_remaining_ms = 0;
            }
        }
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

    /// Return trending tokens filtered by the current search input buffer.
    /// If no search is active, returns the full list.
    pub fn filtered_trending(&self) -> Vec<&TrendingToken> {
        if self.input_active && !self.input_buffer.is_empty() {
            let q = self.input_buffer.to_lowercase();
            self.trending.iter()
                .filter(|t| t.symbol.to_lowercase().contains(&q) || t.name.to_lowercase().contains(&q))
                .collect()
        } else {
            self.trending.iter().collect()
        }
    }

    /// Toggle the command palette.
    pub fn toggle_command_palette(&mut self) {
        self.show_command_palette = !self.show_command_palette;
        if self.show_command_palette {
            self.palette_filter.clear();
            self.palette_cursor = 0;
        }
    }

    /// Toggle sidebar collapsed state.
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
    }

    /// Get the current sidebar width based on collapsed state.
    pub fn sidebar_width(&self) -> u16 {
        if self.sidebar_collapsed { 3 } else { 16 }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}