use crate::app::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Settings — sectioned configuration view.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Min(3),
        ])
        .split(area);

    render_api_block(frame, sections[0], state, theme);
    render_trading_block(frame, sections[1], state, theme);
    render_theme_block(frame, sections[2], state, theme);
    render_learning_block(frame, sections[3], state, theme);
}

fn render_api_block(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    // Check actual env vars
    let gmgn_key = std::env::var("GMGN_API_KEY").unwrap_or_default();
    let alph_cookie = std::env::var("ALPH_DEX_COOKIE").unwrap_or_default();
    let openai_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let ollama_url = std::env::var("OLLAMA_BASE_URL").unwrap_or_default();

    let gmgn_status = if !gmgn_key.is_empty() {
        "✅ Configured"
    } else {
        "❌ Missing"
    };
    let alph_status = if !alph_cookie.is_empty() {
        "✅ Cookie set"
    } else {
        "❌ Missing"
    };
    let openai_status = if !openai_key.is_empty() {
        "✅ Configured"
    } else {
        "❌ Not set"
    };
    let anthropic_status = if !anthropic_key.is_empty() {
        "✅ Configured"
    } else {
        "❌ Not set"
    };
    let ollama_status = if !ollama_url.is_empty() {
        "✅ Configured"
    } else {
        "❌ Not set"
    };

    let text = Text::from(vec![
        Line::from(Span::styled(
            " 🔑 API Keys & Connections",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw(" GMGN (gmgn-cli):     "),
            Span::styled(
                gmgn_status,
                Style::default().fg(if gmgn_key.is_empty() {
                    theme.danger
                } else {
                    theme.success
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Alph AI (cookie):    "),
            Span::styled(
                alph_status,
                Style::default().fg(if alph_cookie.is_empty() {
                    theme.danger
                } else {
                    theme.success
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw(" OpenAI (LLM):        "),
            Span::styled(
                openai_status,
                Style::default().fg(if openai_key.is_empty() {
                    theme.muted
                } else {
                    theme.success
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Anthropic (LLM):     "),
            Span::styled(
                anthropic_status,
                Style::default().fg(if anthropic_key.is_empty() {
                    theme.muted
                } else {
                    theme.success
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Ollama (local LLM):  "),
            Span::styled(
                ollama_status,
                Style::default().fg(if ollama_url.is_empty() {
                    theme.muted
                } else {
                    theme.success
                }),
            ),
        ]),
    ]);
    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        ),
        area,
    );
}

fn render_trading_block(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let r = &state.risk_state;
    let text = Text::from(vec![
        Line::from(Span::styled(
            " Paper Trading",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw(format!(
            " Balance: {:.2} SOL  |  Loss Cap: {:.1} SOL",
            state.balance_sol, r.daily_loss_cap_sol
        ))),
        Line::from(Span::raw(format!(
            " Per Trade Max: {:.1} SOL  |  Max Positions: {}",
            r.per_trade_max_sol, r.max_open_positions
        ))),
        Line::from(Span::raw(format!(
            " Slippage: 3%  |  Same Token Limit: {}",
            r.max_same_token
        ))),
        Line::from(""),
        Line::from(Span::styled(
            " [Reset Portfolio]  [Set Defaults]",
            Style::default().fg(theme.accent),
        )),
    ]);
    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        ),
        area,
    );
}

fn render_theme_block(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let current = match state.theme_preset {
        crate::data::models::ThemePreset::Dark => "Dark (GitHub-inspired)",
        crate::data::models::ThemePreset::Terminal => "Terminal (Bloomberg-style)",
        crate::data::models::ThemePreset::Degen => "Degen (Neon green)",
        crate::data::models::ThemePreset::Cyberpunk => "Cyberpunk (Pink/Cyan)",
    };
    let text = Text::from(vec![
        Line::from(Span::styled(
            " Display & Theme",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw(format!(" Current: {}", current))),
        Line::from(Span::styled(
            " [Dark]  [Terminal]  [Degen]  [Cyberpunk]",
            Style::default().fg(theme.accent),
        )),
        Line::from(""),
        Line::from(Span::raw(" Refresh: 10s  |  Mouse: ON  |  Animations: ON")),
    ]);
    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        ),
        area,
    );
}

fn render_learning_block(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let text = Text::from(vec![
        Line::from(Span::styled(
            " Learning Engine",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw(
            " Auto-Tune: Every 20 trades  |  Min Sample: 10W/10L",
        )),
        Line::from(Span::raw(
            " LLM Provider: OpenAI (default)  |  Anthropic  |  Ollama",
        )),
        Line::from(""),
        Line::from(Span::styled(
            " [Configure LLM Keys]",
            Style::default().fg(theme.accent),
        )),
    ]);
    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        ),
        area,
    );
}
