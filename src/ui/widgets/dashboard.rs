use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::ui::theme::Theme;

/// Render the Dashboard tab content.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_portfolio_panel(frame, chunks[0], state, theme);
    render_trending_panel(frame, chunks[1], state, theme);
}

fn render_portfolio_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let pnl_color = if state.daily_pnl >= 0.0 {
        theme.success
    } else {
        theme.danger
    };

    let text = Text::from(vec![
        Line::from(vec![
            Span::styled("Portfolio", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Balance: "),
            Span::styled(format!("{:.2} SOL", state.balance_sol), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Daily PnL: "),
            Span::styled(format!("{:+.2} SOL", state.daily_pnl), Style::default().fg(pnl_color)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(format!("Open positions: {}", state.open_positions.len())),
        ]),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_trending_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let items: Vec<ListItem> = if state.trending.is_empty() {
        vec![ListItem::new(Line::from(
            Span::styled("No trending data — press 'r' to refresh", Style::default().fg(theme.muted)),
        ))]
    } else {
        state.trending.iter().enumerate().map(|(i, token)| {
            let style = if i == state.list_cursor {
                Style::default().bg(theme.highlight).fg(theme.accent)
            } else {
                Style::default().fg(theme.fg)
            };

            let change_str = token.change_1h
                .map(|c| format!("{:+.1}%", c))
                .unwrap_or_default();

            let change_color = if token.change_1h.unwrap_or(0.0) >= 0.0 {
                theme.success
            } else {
                theme.danger
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>6} ", token.symbol), style.add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.8} ", token.price_usd), style),
                Span::styled(format!("MC:${:.0} ", token.market_cap), Style::default().fg(theme.muted)),
                Span::styled(change_str, Style::default().fg(change_color)),
            ]))
        }).collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Trending (GMGN 1h) "));

    frame.render_widget(list, area);
}