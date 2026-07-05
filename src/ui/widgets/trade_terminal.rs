use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::ui::theme::Theme;

/// Trade Terminal — order form + active positions + risk bar.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    render_order_form(frame, chunks[0], state, theme);
    render_positions(frame, chunks[1], state, theme);
    render_risk_bar(frame, chunks[2], state, theme);
}

fn render_order_form(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let mut lines = vec![
        Line::from(vec![
            Span::styled(" Order Form", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("[BUY]", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            if let Some(ref t) = state.selected_token {
                Span::styled(format!(" {}", t.token.symbol), Style::default().fg(theme.fg))
            } else {
                Span::styled(" Select a token first", Style::default().fg(theme.muted))
            },
        ]),
        Line::from(""),
        Line::from(Span::raw(" Amount:  [____0.5 SOL____]   Slippage: [__3%__]")),
        Line::from(Span::raw(" TP:      [___100%____]   SL:      [__60%__]")),
        Line::from(""),
        Line::from(Span::styled(" Modes: [EXPLODE] [ALPHA] [SCALP] [FALLBACK]", Style::default().fg(theme.muted))),
    ];
    if let Some(ref token) = state.selected_token {
        lines.push(Line::from(Span::styled(
            format!(" Price: ${:.8} | Liq: ${:.0}", token.token.price_usd, token.token.liquidity_usd),
            Style::default().fg(theme.accent),
        )));
    }
    frame.render_widget(Paragraph::new(Text::from(lines)).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}

fn render_positions(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let items: Vec<ListItem> = if state.open_positions.is_empty() {
        vec![ListItem::new(Line::from(
            Span::styled(" No open positions", Style::default().fg(theme.muted)),
        ))]
    } else {
        state.open_positions.iter().map(|pos| {
            let sel = Some(pos.id.as_str()) == state.selected_position_id.as_deref();
            let s = if sel { Style::default().bg(theme.highlight).fg(theme.accent) } else { Style::default().fg(theme.fg) };
            let pnl = pos.pnl_sol.unwrap_or(0.0);
            let pc = if pnl >= 0.0 { theme.success } else { theme.danger };
            let pnl_str = format!("{:+.4}", pnl);
            ListItem::new(Line::from(vec![
                if sel { Span::styled(" ▶", Style::default().fg(theme.accent)) } else { Span::raw("  ") },
                Span::styled(format!(" {}", pos.token_symbol), s.add_modifier(Modifier::BOLD)),
                Span::raw(format!(" @ ${:.6}", pos.entry_price)),
                Span::raw(format!(" {} SOL", pos.amount_sol)),
                Span::styled(format!(" {} ", pos.mode.as_str()), Style::default().fg(theme.warning)),
                Span::styled(format!(" PnL:{}", pnl_str), Style::default().fg(pc)),
                if let Some(tp) = pos.tp_percent { Span::raw(format!(" TP:{:.0}%", tp)) } else { Span::raw("") },
                if let Some(sl) = pos.sl_percent { Span::raw(format!(" SL:{:.0}%", sl)) } else { Span::raw("") },
            ]))
        }).collect()
    };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(format!(" Active Positions ({}) ", state.open_positions.len())));
    frame.render_widget(list, area);
}

fn render_risk_bar(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let r = &state.risk_state;
    let ks_color = if r.kill_switch_active { theme.danger } else { theme.success };
    let ks_text = if r.kill_switch_active { "⚠ KILL SWITCH ACTIVE" } else { "Kill Switch: OFF" };

    let text = Line::from(vec![
        Span::styled(" Risk: ", Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        Span::styled(format!("Daily: {:+.2}/{} SOL", r.daily_realized_pnl, r.daily_loss_cap_sol), Style::default().fg(theme.fg)),
        Span::raw(" │ "),
        Span::styled(format!("Trades: {} ({}W/{}L)", r.trades_today, r.wins_today, r.losses_today), Style::default().fg(theme.fg)),
        Span::raw(" │ "),
        Span::styled(ks_text, Style::default().fg(ks_color)),
        Span::raw(" │ "),
        Span::styled(format!("Max: {}/{} positions", state.open_positions.len(), r.max_open_positions), Style::default().fg(theme.fg)),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));
    frame.render_widget(Paragraph::new(text).block(block), area);
}
