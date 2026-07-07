use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Token ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price_usd: f64,
    pub market_cap: f64,
    pub liquidity_usd: f64,
    pub circulating_supply: f64,
    pub holder_count: u64,
    pub created_at: DateTime<Utc>,
    pub open_timestamp: i64,
    pub logo_url: Option<String>,
    pub launchpad_platform: Option<String>,
    pub is_on_curve: bool,
}

// ── Token Detail (enriched) ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenDetail {
    #[serde(flatten)]
    pub token: Token,
    pub security: TokenSecurity,
    pub dev_info: DevInfo,
    pub social_links: Option<SocialLinks>,
    pub wallet_tags: WalletTags,
    pub pool_info: Option<PoolInfo>,
    pub price_stats: PriceStats,
}

// ── Security ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenSecurity {
    pub rug_ratio: f64,
    pub is_wash_trading: bool,
    pub open_source: bool,
    pub renounced_mint: bool,
    pub renounced_freeze: bool,
    pub is_honeypot: bool,
    pub buy_tax: f64,
    pub sell_tax: f64,
    pub top_10_holder_rate: f64,
    pub dev_team_hold_rate: f64,
    pub creator_hold_rate: f64,
    pub creator_status: CreatorStatus,
    pub suspected_insider_hold_rate: f64,
    pub burn_status: String,
    pub sniper_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CreatorStatus {
    #[default]
    Unknown,
    CreatorHold,
    CreatorClose,
}

// ── Dev Info ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevInfo {
    pub creator_address: String,
    pub creator_token_balance: f64,
    pub creator_status: CreatorStatus,
    pub creator_prev_tokens: u64,
    pub creator_ath_mc: Option<f64>,
    pub creator_ath_token: Option<String>,
    pub cto_flag: bool,
    pub dexscr_ad: bool,
    pub dexscr_boost: bool,
    pub dexscr_trending_bar: bool,
}

// ── Social Links ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SocialLinks {
    pub twitter_username: Option<String>,
    pub website: Option<String>,
    pub telegram: Option<String>,
    pub discord: Option<String>,
    pub description: Option<String>,
}

impl SocialLinks {
    /// True if at least one social link is present
    pub fn has_any(&self) -> bool {
        self.twitter_username.is_some()
            || self.website.is_some()
            || self.telegram.is_some()
            || self.discord.is_some()
    }
}

// ── Wallet Tags ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletTags {
    pub smart_wallets: u64,
    pub renowned_wallets: u64,
    pub sniper_wallets: u64,
    pub rat_trader_wallets: u64,
    pub bundler_wallets: u64,
    pub whale_wallets: u64,
    pub fresh_wallets: u64,
}

// ── Pool Info ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolInfo {
    pub pool_address: String,
    pub exchange: String,
    pub liquidity_usd: f64,
    pub base_reserve: f64,
    pub quote_reserve: f64,
    pub fee_ratio: f64,
}

// ── Price Stats ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceStats {
    pub price_1m: Option<f64>,
    pub price_5m: Option<f64>,
    pub price_1h: Option<f64>,
    pub price_6h: Option<f64>,
    pub price_24h: Option<f64>,
    pub volume_1h: Option<f64>,
    pub volume_24h: Option<f64>,
    pub buys_1h: Option<u64>,
    pub sells_1h: Option<u64>,
    pub swaps_1h: Option<u64>,
    pub hot_level: Option<u64>,
    pub change_1m: Option<f64>,
    pub change_5m: Option<f64>,
    pub change_1h: Option<f64>,
}

// ── Kline / OHLCV ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KlineCandle {
    pub time: i64,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume_usd: f64,
    pub amount: f64,
    pub buys: u64,
    pub sells: u64,
}

// ── Trending Token (lightweight, from lists) ───────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrendingToken {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub market_cap: f64,
    pub liquidity_usd: f64,
    pub volume_5m: Option<f64>,
    pub volume_1h: Option<f64>,
    pub volume_24h: Option<f64>,
    pub change_5m: Option<f64>,
    pub change_1h: Option<f64>,
    pub change_24h: Option<f64>,
    pub hot_level: Option<u64>,
    pub smart_degen_count: Option<u64>,
    pub renowned_count: Option<u64>,
    pub holder_count: Option<u64>,
    pub swaps_5m: Option<u64>,
    pub swaps_1h: Option<u64>,
    pub is_on_curve: Option<bool>,
    pub launchpad_platform: Option<String>,
    pub rug_ratio: Option<f64>,
    pub dexscr_boost: Option<bool>,
}

// ── Smart Money / KOL Trade ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmartMoneyTrade {
    pub tx_hash: String,
    pub maker: String,
    pub side: TradeSide,
    pub token_address: String,
    pub token_symbol: String,
    pub amount_usd: f64,
    pub token_amount: f64,
    pub price_usd: f64,
    pub price_change: f64,
    pub is_open_or_close: bool,
    pub timestamp: i64,
    pub maker_tags: Vec<String>,
    pub maker_twitter: Option<String>,
    pub launchpad: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum TradeSide {
    #[default]
    Sell,
    Buy,
}

// ── Token Signal ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenSignal {
    pub token_address: String,
    pub token_symbol: String,
    pub signal_type: SignalType,
    pub confidence: SignalConfidence,
    pub trigger_at: i64,
    pub amount_usd: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum SignalType {
    #[default]
    PriceSpike,
    SmartMoneyBuy,
    LargeBuy,
    DexAd,
    KolMention,
    Cto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum SignalConfidence {
    #[default]
    Copper,
    Silver,
    Gold,
}

impl SignalConfidence {
    /// Numeric score for alpha filter: Gold=1.0, Silver=0.6, Copper=0.3
    pub fn score(&self) -> f64 {
        match self {
            SignalConfidence::Gold => 1.0,
            SignalConfidence::Silver => 0.6,
            SignalConfidence::Copper => 0.3,
        }
    }
}

// ── Twitter / Social ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tweet {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub text: String,
    pub tweet_type: TweetType,
    pub created_at: DateTime<Utc>,
    pub likes: u64,
    pub retweets: u64,
    pub replies: u64,
    pub extracted_token_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum TweetType {
    #[default]
    Send,
    Retweeted,
    RepliedTo,
    Quoted,
}

// ── Wallet (from portfolio/tracking) ──────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletHolding {
    pub token_address: String,
    pub token_symbol: String,
    pub balance: f64,
    pub usd_value: f64,
    pub cost: f64,
    pub realized_profit: f64,
    pub unrealized_profit: f64,
    pub total_profit: f64,
    pub profit_change: f64,
    pub buy_tx_count: u64,
    pub sell_tx_count: u64,
}

// ── Trench (new token) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrenchToken {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub market_cap: f64,
    pub liquidity_usd: f64,
    pub age_minutes: u64,
    pub platform: String,
    pub holder_count: u64,
    pub dev_hold_rate: f64,
    pub smart_holding: u64,
    pub kol_calls: u64,
    pub bonding_curve: bool,
    pub social: Option<SocialLinks>,
}

// ── DEX Screener Boost ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DexScreenerPair {
    pub address: String,
    pub token_address: String,
    pub token_symbol: String,
    pub price_usd: f64,
    pub liquidity_usd: f64,
    pub fdv: f64,
    pub volume_24h: f64,
    pub change_24h: f64,
    pub boost_count: u64,
}

// ── Trade / Position Types ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum PositionStatus {
    #[default]
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum TradeMode {
    #[default]
    Fallback,
    Scalp,
    Alpha,
    Explode,
}

impl TradeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TradeMode::Explode => "EXPLODE",
            TradeMode::Alpha => "ALPHA",
            TradeMode::Scalp => "SCALP",
            TradeMode::Fallback => "FALLBACK",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrailingConfig {
    pub activation_percent: f64,
    pub callback_percent: f64,
    pub peak_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaperPosition {
    pub id: String,
    pub token_address: String,
    pub token_symbol: String,
    pub side: TradeSide,
    pub entry_price: f64,
    pub amount_sol: f64,
    pub amount_tokens: f64,
    pub slippage: f64,
    pub mode: TradeMode,
    pub tp_percent: Option<f64>,
    pub sl_percent: Option<f64>,
    pub trailing_tp: Option<TrailingConfig>,
    pub trailing_sl: Option<TrailingConfig>,
    pub status: PositionStatus,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub exit_price: Option<f64>,
    pub pnl_sol: Option<f64>,
    pub pnl_percent: Option<f64>,
    pub feature_vector_json: String,
    pub alpha_score: f64,
    pub rug_report_json: String,
}

// ── Alpha Filter Types ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureVector {
    // Market Momentum
    pub volume_1m: Option<f64>,
    pub volume_5m: Option<f64>,
    pub volume_1h: Option<f64>,
    pub swaps_1h: Option<u64>,
    pub price_change_1m: Option<f64>,
    pub price_change_1h: Option<f64>,
    pub hot_level: Option<u64>,
    // Liquidity
    pub liquidity_usd: f64,
    pub market_cap: f64,
    pub pool_exchange: String,
    pub is_on_curve: bool,
    // Security / Rug
    pub rug_ratio: f64,
    pub is_wash_trading: bool,
    pub open_source: bool,
    pub renounced_mint: bool,
    pub renounced_freeze: bool,
    // Holder
    pub holder_count: u64,
    pub top_10_holder_rate: f64,
    pub dev_team_hold_rate: f64,
    pub creator_hold_rate: f64,
    pub suspected_insider_hold_rate: f64,
    pub fresh_wallet_rate: f64,
    // Wallet Signals
    pub smart_degen_count: u64,
    pub renowned_count: u64,
    pub sniper_count: u64,
    pub bundler_rate: f64,
    pub rat_trader_rate: f64,
    // Dev Profile
    pub creator_status: String,
    pub creator_prev_tokens: u64,
    pub creator_ath_mc: Option<f64>,
    pub cto_flag: bool,
    pub dexscr_ad: bool,
    pub dexscr_boost: bool,
    // Social / Meta
    pub has_social_links: bool,
    pub dexscr_trending_bar: bool,
    pub launchpad_platform: Option<String>,
    // Twitter (Alph AI)
    pub twitter_mentions_1h: Option<u64>,
    pub twitter_sentiment: Option<f64>,
    pub twitter_follower_count: Option<u64>,
    pub twitter_ca_extracted: bool,
    pub signal_confidence: Option<f64>,
    pub smart_wallet_pnl: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlphaReport {
    pub token_address: String,
    pub token_symbol: String,
    pub alpha_score: f64,
    pub scores: CategoryScores,
    pub hard_filter_result: HardFilterResult,
    pub mode: TradeMode,
    pub sizing: SizingBounds,
    pub feature_vector: FeatureVector,
    pub rug_report: RugReport,
    pub narrative: Option<String>,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryScores {
    pub momentum: f64,
    pub safety: f64,
    pub holder_quality: f64,
    pub liquidity: f64,
    pub dev_trust: f64,
    pub social: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardFilterResult {
    pub passed: bool,
    pub failures: Vec<FilterFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterFailure {
    pub name: String,
    pub value: f64,
    pub threshold: f64,
    pub direction: FilterDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterDirection {
    Exceeded,
    Below,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SizingBounds {
    pub min_sol: f64,
    pub max_sol: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RugReport {
    pub severity: RugSeverity,
    pub flags: Vec<RugFlag>,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum RugSeverity {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RugFlag {
    pub name: String,
    pub severity: RugSeverity,
    pub detail: String,
    pub value: f64,
    pub threshold: f64,
}

// ── Alpha Config (mutable weights + thresholds) ──────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaConfig {
    // Weights (must sum to ~1.0)
    pub w_momentum: f64,
    pub w_safety: f64,
    pub w_holder: f64,
    pub w_liquidity: f64,
    pub w_dev: f64,
    pub w_social: f64,
    // Hard filter thresholds
    pub hf_rug_ratio_max: f64,
    pub hf_dev_hold_max: f64,
    pub hf_wash_trading: bool,
    pub hf_renounced_mint: bool,
    pub hf_liquidity_min_usd: f64,
}

impl Default for AlphaConfig {
    fn default() -> Self {
        Self {
            w_momentum: 0.25,
            w_safety: 0.15,
            w_holder: 0.20,
            w_liquidity: 0.18,
            w_dev: 0.07,
            w_social: 0.15,
            hf_rug_ratio_max: 0.30,
            hf_dev_hold_max: 0.15,
            hf_wash_trading: true,
            hf_renounced_mint: true,
            hf_liquidity_min_usd: 5000.0,
        }
    }
}

// ── Risk Types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskState {
    pub daily_loss_cap_sol: f64,
    pub per_trade_max_sol: f64,
    pub per_trade_max_pct: f64,
    pub daily_realized_pnl: f64,
    pub trades_today: u32,
    pub wins_today: u32,
    pub losses_today: u32,
    pub kill_switch_active: bool,
    pub max_open_positions: u8,
    pub max_same_token: u8,
}

impl Default for RiskState {
    fn default() -> Self {
        Self {
            daily_loss_cap_sol: 5.0,
            per_trade_max_sol: 2.5,
            per_trade_max_pct: 5.0,
            daily_realized_pnl: 0.0,
            trades_today: 0,
            wins_today: 0,
            losses_today: 0,
            kill_switch_active: false,
            max_open_positions: 5,
            max_same_token: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PreTradeCheckResult {
    Approved,
    Rejected(String),
    Warning(String),
}

// ── Daily Stats ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyStats {
    pub date: String,
    pub starting_balance: f64,
    pub daily_realized_pnl: f64,
    pub trades_today: u32,
    pub wins_today: u32,
    pub losses_today: u32,
    pub kill_switch_active: bool,
    pub override_count: u32,
}

// ── Portfolio ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub balance_sol: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Portfolio {
    fn default() -> Self {
        Self {
            balance_sol: 50.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// ── Data Event (async → UI thread) ───────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DataEvent {
    TrendingUpdated(Vec<TrendingToken>),
    TokenLoaded(Box<TokenDetail>),
    KlineUpdated(String, Vec<KlineCandle>),
    SmartMoneyActivity(Vec<SmartMoneyTrade>),
    SignalReceived(TokenSignal),
    TwitterMention(String, Tweet),
    RateLimitHit(String, Option<i64>),
    ConnectionError(String, String),
    PriceUpdated(String, f64),
    TrenchesUpdated(Vec<TrenchToken>),
    WatchlistUpdated(Vec<TrendingToken>),
    AutoTuneHistoryLoaded(Vec<crate::storage::journal::TuningHistoryRow>),
    PostMortemHistoryLoaded(Vec<crate::storage::journal::PostMortemRow>),
}

// ── App Event (crossterm → update loop) ─────────────────────────

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    Tick,
    Data(Box<DataEvent>),
}

// ── App Command (update → side effects) ─────────────────────────

#[derive(Debug)]
pub enum AppCommand {
    FetchTrending,
    FetchTokenDetail(String),
    FetchKline(String, String, i64, i64),
    PaperBuy {
        token_address: String,
        amount_sol: f64,
        mode: TradeMode,
        tp_percent: Option<f64>,
        sl_percent: Option<f64>,
    },
    PaperSell {
        position_id: String,
        sell_percent: f64,
    },
    AddToWatchlist(String),
    RemoveFromWatchlist(String),
    ShowNotification(String),
    ShowModal(String),
    SwitchTab(usize),
    EmergencyExitAll,
    ToggleKillSwitch,
    RunPostMortem(String, String),
    RunAutoTune,
    SaveAlphaConfig(AlphaConfig),
    /// New commands for Wave A/B
    FetchSmartMoney,
    FetchSignals,
    FetchTrenches(String),
    /// Load strategy data from DB
    FetchAutoTuneHistory,
    FetchPostMortemHistory,
}

// ── Tab Index ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TabIndex {
    #[default]
    Dashboard = 0,
    Scanner = 1,
    Analyzer = 2,
    TradeTerminal = 3,
    Journal = 4,
    Strategy = 5,
    Settings = 6,
}

impl TabIndex {
    pub const COUNT: usize = 7;

    pub fn from_usize(i: usize) -> Self {
        match i {
            0 => TabIndex::Dashboard,
            1 => TabIndex::Scanner,
            2 => TabIndex::Analyzer,
            3 => TabIndex::TradeTerminal,
            4 => TabIndex::Journal,
            5 => TabIndex::Strategy,
            6 => TabIndex::Settings,
            _ => TabIndex::Dashboard,
        }
    }

    pub fn as_usize(&self) -> usize {
        *self as usize
    }

    pub fn label(&self) -> &'static str {
        match self {
            TabIndex::Dashboard => "Dashboard",
            TabIndex::Scanner => "Scanner",
            TabIndex::Analyzer => "Analyzer",
            TabIndex::TradeTerminal => "Trade",
            TabIndex::Journal => "Journal",
            TabIndex::Strategy => "Strategy",
            TabIndex::Settings => "Settings",
        }
    }

    pub fn next(&self) -> Self {
        Self::from_usize((self.as_usize() + 1) % Self::COUNT)
    }

    pub fn prev(&self) -> Self {
        Self::from_usize((self.as_usize() + Self::COUNT - 1) % Self::COUNT)
    }
}

// ── Theme ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum ThemePreset {
    #[default]
    Dark,
    Degen,
    Terminal,
    Cyberpunk,
}
