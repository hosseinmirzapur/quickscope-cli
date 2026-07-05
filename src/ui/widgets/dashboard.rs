use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::data::models::TrendingToken;
use crate::ui::theme::Theme;
use crate::ui::format::{format_marketcap, marketcap_color, format_volume};

/// Render the Dashboard tab: portfolio left, trending right-top, smart money right-bottom.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(area);

    render_portfolio_panel(frame, chunks[0], state, theme);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[1]);

    render_trending_panel(frame, right[0], state, theme);
    render_smart_money_panel(frame, right[1], state, theme);
}

fn render_portfolio_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let pnl_color = if state.daily_pnl >= 0.0 { theme.success } else { theme.danger };
    let win_rate = if state.risk_state.trades_today > 0 {
        state.risk_state.wins_today as f64 / state.risk_state.trades_today as f64 * 100.0
    } else { 0.0 };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(" Portfolio", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Balance:  "),
            Span::styled(format!("{:.2} SOL", state.balance_sol), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Daily PnL: "),
            Span::styled(format!("{:+.2} SOL", state.daily_pnl), Style::default().fg(pnl_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(format!("Positions: {}", state.open_positions.len())),
        ]),
        Line::from(vec![
            Span::raw(format!("Trades: {}W/{}L", state.risk_state.wins_today, state.risk_state.losses_today)),
            Span::raw(format!(" ({:.0}%)", win_rate)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Kill Switch: "),
            Span::styled(
                if state.risk_state.kill_switch_active { "ACTIVE" } else { "OFF" },
                Style::default().fg(if state.risk_state.kill_switch_active { theme.danger } else { theme.muted }),
            ),
        ]),
    ];

    // Current open positions with live PnL
    if !state.open_positions.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(" Positions:", Style::default().fg(theme.muted))));
        for pos in state.open_positions.iter().take(5) {
            let pnl = pos.pnl_sol.unwrap_or(0.0);
            let c = if pnl >= 0.0 { theme.success } else { theme.danger };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&pos.token_symbol, Style::default().fg(theme.fg)),
                Span::raw(" "),
                Span::styled(format!("{:+.2} SOL", pnl), Style::default().fg(c)),
            ]));
        }
    }

    let text = Text::from(lines);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_trending_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let visible_rows = area.height.saturating_sub(2) as usize; // minus borders
    let filtered = state.filtered_trending();
    let offset = state.scroll_offset.min(filtered.len().saturating_sub(1));
    let visible_tokens: Vec<&&TrendingToken> = filtered
        .iter()
        .skip(offset)
        .take(visible_rows)
        .collect();

    let items: Vec<ListItem> = if state.loading_trending {
        vec![ListItem::new(Line::from(
            Span::styled(" ◌ Fetching trending tokens...", Style::default().fg(theme.warning)),
        ))]
    } else if visible_tokens.is_empty() {
        vec![ListItem::new(Line::from(
            if state.input_active && !state.input_buffer.is_empty() {
                Span::styled(" No matches — try a different search", Style::default().fg(theme.muted))
            } else {
                Span::styled(" No trending data — press 'r'", Style::default().fg(theme.muted))
            },
        ))]
    } else {
        visible_tokens.iter().enumerate().map(|(rel_i, token)| {
            let abs_i = offset + rel_i;
            let selected = abs_i == state.list_cursor;
            let style = if selected {
                Style::default().bg(theme.highlight).fg(theme.accent)
            } else {
                Style::default().fg(theme.fg)
            };

            let change = token.change_1h.unwrap_or(0.0);
            let change_color = if change >= 0.0 { theme.success } else { theme.danger };
            let mc_color = marketcap_color(token.market_cap, theme);
            let vol = token.volume_1h.unwrap_or(0.0);
            let sm = token.smart_degen_count.unwrap_or(0);
            let is_hot = token.hot_level.unwrap_or(0) >= 3;
            let vol_1h_change = token.change_1h.unwrap_or(0.0);

            // Primary: Symbol | Marketcap (color-coded) | Vol 1h | Change 1h
            // Secondary: Price (muted) | Smart Money | 🔥 if hot + gain
            let mut cols = vec![
                Span::styled(format!(" {:>8}", token.symbol), style.add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(
                    format_marketcap(token.market_cap),
                    Style::default().fg(mc_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format_volume(vol),
                    Style::default().fg(theme.muted),
                ),
                Span::raw(" "),
                Span::styled(format!("{:+.1}%", change), Style::default().fg(change_color).add_modifier(Modifier::BOLD)),
            ];

            // Price (muted, secondary)
            cols.push(Span::raw(" "));
            cols.push(Span::styled(
                format!("${:.8}", token.price_usd),
                Style::default().fg(theme.muted),
            ));

            // Smart money count
            if sm > 0 {
                cols.push(Span::raw(" "));
                cols.push(Span::styled(format!("SM:{}", sm), Style::default().fg(theme.accent)));
            }

            // 🔥 Fire indicator if hot AND positive change
            if is_hot && vol_1h_change > 0.0 {
                cols.push(Span::raw(" "));
                cols.push(Span::styled("🔥", Style::default().fg(theme.warning)));
            }

            ListItem::new(Line::from(cols))
        }).collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Trending (1h) "));
    frame.render_widget(list, area);
}

fn render_smart_money_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let items: Vec<ListItem> = if state.smart_money_feed.is_empty() {
        vec![ListItem::new(Line::from(
            Span::styled(" No smart money activity yet", Style::default().fg(theme.muted)),
        ))]
    } else {
        state.smart_money_feed.iter().take(8).map(|trade| {
            let is_buy = matches!(trade.side, crate::data::models::TradeSide::Buy);
            let (arrow, color) = if is_buy {
                ("↑", theme.success)
            } else {
                ("↓", theme.danger)
            };
            ListItem::new(Line::from(vec![
                Span::styled(arrow, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(&trade.token_symbol, Style::default().fg(theme.accent)),
                Span::raw(format!(" ${:.0}", trade.amount_usd)),
            ]))
        }).collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Smart Money / KOL "));
    frame.render_widget(list, area);
}
