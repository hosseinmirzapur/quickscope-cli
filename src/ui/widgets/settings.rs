use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::ui::theme::Theme;

/// Settings — sectioned configuration view.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Length(7), Constraint::Length(7), Constraint::Min(3)])
        .split(area);

    render_api_block(frame, sections[0], state, theme);
    render_trading_block(frame, sections[1], state, theme);
    render_theme_block(frame, sections[2], state, theme);
    render_learning_block(frame, sections[3], state, theme);
}

fn render_api_block(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let text = Text::from(vec![
        Line::from(Span::styled(" API Keys", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(" GMGN: ✅ Connected (via gmgn-cli)", Style::default().fg(theme.success))),
        Line::from(Span::styled(" Alph AI: ⚠️  Cookie set in .env", Style::default().fg(theme.warning))),
        Line::from(Span::styled(" OpenAI: ❌ Not configured (for LLM post-mortem)", Style::default().fg(theme.muted))),
        Line::from(""),
        Line::from(Span::styled(" [Check Connections]", Style::default().fg(theme.accent))),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}

fn render_trading_block(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let r = &state.risk_state;
    let text = Text::from(vec![
        Line::from(Span::styled(" Paper Trading", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::raw(format!(" Balance: {:.2} SOL  |  Loss Cap: {:.1} SOL", state.balance_sol, r.daily_loss_cap_sol))),
        Line::from(Span::raw(format!(" Per Trade Max: {:.1} SOL  |  Max Positions: {}", r.per_trade_max_sol, r.max_open_positions))),
        Line::from(Span::raw(format!(" Slippage: 3%  |  Same Token Limit: {}", r.max_same_token))),
        Line::from(""),
        Line::from(Span::styled(" [Reset Portfolio]  [Set Defaults]", Style::default().fg(theme.accent))),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}

fn render_theme_block(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let current = match state.theme_preset {
        crate::data::models::ThemePreset::Dark => "Dark (active)",
        crate::data::models::ThemePreset::Degen => "Degen (active)",
    };
    let text = Text::from(vec![
        Line::from(Span::styled(" Display & Theme", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::raw(format!(" Current: {}", current))),
        Line::from(Span::styled(" [Dark Mode]  [Degen Mode]", Style::default().fg(theme.accent))),
        Line::from(""),
        Line::from(Span::raw(" Refresh: 30s  |  Mouse: ON  |  Animations: ON")),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}

fn render_learning_block(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let text = Text::from(vec![
        Line::from(Span::styled(" Learning Engine", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::raw(" Auto-Tune: Every 20 trades  |  Min Sample: 10W/10L")),
        Line::from(Span::raw(" LLM Provider: OpenAI (default)  |  Anthropic  |  Ollama")),
        Line::from(""),
        Line::from(Span::styled(" [Configure LLM Keys]", Style::default().fg(theme.accent))),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}
