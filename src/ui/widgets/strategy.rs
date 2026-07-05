use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::ui::theme::Theme;

/// Strategy & Learning — alpha config, auto-tune history, LLM post-mortem.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_alpha_config(frame, chunks[0], state, theme);
    render_llm_panel(frame, chunks[1], state, theme);
}

fn render_alpha_config(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let ac = &state.alpha_config;
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(" Alpha Weights", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::raw(format!(" Momentum:  {:.0}%", ac.w_momentum * 100.0))),
        Line::from(Span::raw(format!(" Safety:    {:.0}%", ac.w_safety * 100.0))),
        Line::from(Span::raw(format!(" Holder:    {:.0}%", ac.w_holder * 100.0))),
        Line::from(Span::raw(format!(" Liquidity: {:.0}%", ac.w_liquidity * 100.0))),
        Line::from(Span::raw(format!(" Dev:       {:.0}%", ac.w_dev * 100.0))),
        Line::from(Span::raw(format!(" Social:    {:.0}%", ac.w_social * 100.0))),
        Line::from(""),
        Line::from(Span::styled(" Hard Filters", Style::default().fg(theme.fg).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw(format!(" Max Rug Ratio: {:.0}%", ac.hf_rug_ratio_max * 100.0))),
        Line::from(Span::raw(format!(" Max Dev Hold: {:.0}%", ac.hf_dev_hold_max * 100.0))),
        Line::from(Span::raw(format!(" Min Liquidity: ${:.0}", ac.hf_liquidity_min_usd))),
        Line::from(Span::raw(format!(" Wash Trade Filter: {}", if ac.hf_wash_trading { "ON" } else { "OFF" }))),
        Line::from(Span::raw(format!(" Renounced Mint Filter: {}", if ac.hf_renounced_mint { "ON" } else { "OFF" }))),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [Save Config]  ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("[Run Auto-Tune]", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
        ]),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}

fn render_llm_panel(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(" LLM Post-Mortem", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled(" Period: [7d] [30d] [All]", Style::default().fg(theme.muted))),
        Line::from(Span::styled(" Provider: [OpenAI] [Anthropic] [Ollama]", Style::default().fg(theme.muted))),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [Run Post-Mortem]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled(" Results will appear here after analysis.", Style::default().fg(theme.muted))),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}
