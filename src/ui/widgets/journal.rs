use crate::app::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Trade Journal — history, stats, filters.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Min(3),
        ])
        .split(area);

    render_stats(frame, chunks[0], state, theme);
    render_filter_row(frame, chunks[1], state, theme);
    render_history(frame, chunks[2], state, theme);
}

fn render_stats(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let r = &state.risk_state;
    let wr = if r.trades_today > 0 {
        r.wins_today as f64 / r.trades_today as f64 * 100.0
    } else {
        0.0
    };
    let trade_count = state.trade_history.len();

    // Compute total realized PnL from history
    let total_pnl: f64 = state
        .trade_history
        .iter()
        .map(|p| p.pnl_sol.unwrap_or(0.0))
        .sum();
    let pnl_color = if total_pnl >= 0.0 {
        theme.success
    } else {
        theme.danger
    };

    // Compute best/worst trade
    let best = state
        .trade_history
        .iter()
        .map(|p| p.pnl_sol.unwrap_or(0.0))
        .fold(f64::NEG_INFINITY, f64::max);
    let worst = state
        .trade_history
        .iter()
        .map(|p| p.pnl_sol.unwrap_or(0.0))
        .fold(f64::INFINITY, f64::min);

    let mut lines = vec![
        Line::from(vec![Span::styled(
            " 📊 Trade Journal",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Total Trades: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{}", trade_count),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(" Win Rate: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:.0}%", wr),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(" Today: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{}W/{}L", r.wins_today, r.losses_today),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Total PnL: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:+.4} SOL", total_pnl),
                Style::default().fg(pnl_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(" Best: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:+.4}", best.max(0.0)),
                Style::default().fg(theme.success),
            ),
            Span::raw("   "),
            Span::styled(" Worst: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:+.4}", worst.min(0.0)),
                Style::default().fg(theme.danger),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Session ", Style::default().fg(theme.muted)),
            Span::styled(
                " [Post-Mortem] ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[Export CSV] ", Style::default().fg(theme.muted)),
            Span::styled("[Auto-Tune]", Style::default().fg(theme.warning)),
        ]),
    ];

    if trade_count == 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " 💡 No trades yet. Switch to Trade tab and paper trade to build your journal.",
            Style::default().fg(theme.muted),
        )));
    }

    frame.render_widget(
        Paragraph::new(Text::from(lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        ),
        area,
    );
}

fn render_filter_row(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let text = Line::from(vec![
        Span::styled(
            " All",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(theme.muted)),
        Span::styled("Wins", Style::default().fg(theme.muted)),
        Span::styled(" │ ", Style::default().fg(theme.muted)),
        Span::styled("Losses", Style::default().fg(theme.muted)),
        Span::styled(" │ ", Style::default().fg(theme.muted)),
        Span::styled("Filter by token...", Style::default().fg(theme.muted)),
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

fn render_history(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let text = if state.trade_history.is_empty() {
        Text::from(vec![Line::from(Span::styled(
            " No trades yet. Paper trade from the Trade tab to build your journal.",
            Style::default().fg(theme.muted),
        ))])
    } else {
        let mut lines = vec![Line::from(vec![
            Span::styled(
                " Date       ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Token       ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Side  ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Entry      ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Exit       ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " PnL         ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Mode    ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " α",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];
        lines.push(Line::from(Span::styled(
            " ─────────────────────────────────────────────────────────────────",
            Style::default().fg(theme.muted),
        )));

        for pos in state.trade_history.iter().take(30) {
            let pnl = pos.pnl_sol.unwrap_or(0.0);
            let pc = if pnl >= 0.0 {
                theme.success
            } else {
                theme.danger
            };
            let pnl_s = format!("{:+.4}", pnl);
            lines.push(Line::from(vec![
                Span::raw(format!(" {:>10}", pos.opened_at.format("%Y-%m-%d"))),
                Span::styled(
                    format!(" {:>10}", pos.token_symbol),
                    Style::default().fg(theme.fg),
                ),
                Span::raw(format!(" {:>5}", format!("{:?}", pos.side))),
                Span::raw(format!(" ${:.6}", pos.entry_price)),
                Span::styled(
                    format!(" ${:.6}", pos.exit_price.unwrap_or(0.0)),
                    Style::default().fg(theme.muted),
                ),
                Span::styled(
                    format!(" {:>10}", pnl_s),
                    Style::default().fg(pc).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" {:>7}", pos.mode.as_str())),
                Span::styled(
                    format!(" {:.1}", pos.alpha_score),
                    Style::default().fg(theme.accent),
                ),
            ]));
        }
        Text::from(lines)
    };
    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Trade History "),
        ),
        area,
    );
}
