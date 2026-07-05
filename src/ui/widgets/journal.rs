use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::ui::theme::Theme;

/// Trade Journal — history, stats, filters.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Length(1), Constraint::Min(3)])
        .split(area);

    render_stats(frame, chunks[0], state, theme);
    render_filter_row(frame, chunks[1], state, theme);
    render_history(frame, chunks[2], state, theme);
}

fn render_stats(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let r = &state.risk_state;
    let wr = if r.trades_today > 0 { r.wins_today as f64 / r.trades_today as f64 * 100.0 } else { 0.0 };
    let trade_count = state.trade_history.len() + r.trades_today as usize;
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(" Journal", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(format!(" Total Trades: {}", trade_count)),
            Span::raw(format!(" │ Win Rate: {:.0}%", wr)),
            Span::raw(format!(" │ Today: {}W/{}L", r.wins_today, r.losses_today)),
            Span::raw(format!(" │ PnL: {:+.2} SOL", state.daily_pnl)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(" Session "),
            Span::styled("[Post-Mortem]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" [Export CSV]"),
        ]),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}

fn render_filter_row(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let text = Line::from(vec![
        Span::styled(" All", Style::default().fg(theme.accent)),
        Span::raw(" │ Wins │ Losses │ Filter by token... "),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}

fn render_history(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let text = if state.trade_history.is_empty() && state.risk_state.trades_today == 0 {
        Text::from(vec![
            Line::from(Span::styled(" No trades yet. Paper trade from the Trade tab to build your journal.", Style::default().fg(theme.muted))),
        ])
    } else {
        let mut lines = vec![Line::from(vec![
            Span::styled(" Date    ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" Token      ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" Side  ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" Entry      ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" Exit       ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" PnL         ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" Mode    ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" α", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ])];
        lines.push(Line::from(Span::styled(
            " ─────────────────────────────────────────────────────────────────",
            Style::default().fg(theme.muted),
        )));

        for pos in state.trade_history.iter().take(15) {
            let pnl = pos.pnl_sol.unwrap_or(0.0);
            let pc = if pnl >= 0.0 { theme.success } else { theme.danger };
            let pnl_s = format!("{:+.4}", pnl);
            lines.push(Line::from(vec![
                Span::raw(format!(" {:>8}", pos.opened_at.format("%Y-%m-%d"))),
                Span::raw(format!(" {:>10}", pos.token_symbol)),
                Span::raw(format!(" {:>5}", format!("{:?}", pos.side))),
                Span::raw(format!(" ${:.6}", pos.entry_price)),
                Span::styled(format!(" ${:.6}", pos.exit_price.unwrap_or(0.0)), Style::default().fg(theme.muted)),
                Span::styled(format!(" {:>8}", pnl_s), Style::default().fg(pc)),
                Span::raw(format!(" {:>7}", pos.mode.as_str())),
                Span::styled(format!(" {:.0}", pos.alpha_score), Style::default().fg(theme.accent)),
            ]));
        }
        Text::from(lines)
    };
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}
