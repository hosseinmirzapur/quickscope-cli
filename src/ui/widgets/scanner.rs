use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::data::models::TrendingToken;
use crate::ui::theme::Theme;
use crate::ui::format::{format_marketcap, marketcap_color, format_volume};

/// Token Scanner with filter bar, full list, and detail panel.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // filter bar
            Constraint::Min(5),     // token list
            Constraint::Length(7),  // detail panel (when selected)
        ])
        .split(area);

    render_filter_bar(frame, chunks[0], state, theme);
    render_token_list(frame, chunks[1], state, theme);
    if state.selected_token.is_some() {
        render_detail_panel(frame, chunks[2], state, theme);
    }
}

fn render_filter_bar(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let text = Line::from(vec![
        Span::styled(" [Trending] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(" │ "),
        Span::styled("Trenches", Style::default().fg(theme.muted)),
        Span::raw(" │ "),
        Span::styled("Watchlist", Style::default().fg(theme.muted)),
        Span::raw(" │ "),
        Span::styled("AI-Rec", Style::default().fg(theme.muted)),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_token_list(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let visible_rows = area.height.saturating_sub(2) as usize; // minus borders
    let filtered = state.filtered_trending();
    let offset = state.scroll_offset.min(filtered.len().saturating_sub(1));
    let visible_tokens: Vec<&&TrendingToken> = filtered
        .iter()
        .skip(offset)
        .take(visible_rows)
        .collect();

    let items: Vec<ListItem> = if visible_tokens.is_empty() {
        vec![ListItem::new(Line::from(
            if state.input_active && !state.input_buffer.is_empty() {
                Span::styled(" No matches — try a different search", Style::default().fg(theme.muted))
            } else {
                Span::styled(" Press 'r' to load trending tokens", Style::default().fg(theme.muted))
            },
        ))]
    } else {
        visible_tokens.iter().enumerate().map(|(rel_i, t)| {
            let abs_i = offset + rel_i;
            let sel = abs_i == state.list_cursor;
            let s = if sel { Style::default().bg(theme.highlight).fg(theme.accent) } else { Style::default().fg(theme.fg) };
            let chg = t.change_1h.unwrap_or(0.0);
            let cc = if chg >= 0.0 { theme.success } else { theme.danger };
            let mc_color = marketcap_color(t.market_cap, theme);
            let vol = t.volume_1h.unwrap_or(0.0);
            let sm = t.smart_degen_count.unwrap_or(0);

            ListItem::new(Line::from(vec![
                if sel { Span::styled(" ▶", Style::default().fg(theme.accent)) } else { Span::raw("  ") },
                Span::raw(" "),
                Span::styled(format!("{:>8}", t.symbol), s.add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(
                    format_marketcap(t.market_cap),
                    Style::default().fg(mc_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(format_volume(vol), Style::default().fg(theme.muted)),
                Span::raw(" "),
                Span::styled(format!("{:+.1}%", chg), Style::default().fg(cc).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(
                    format!("${:.8}", t.price_usd),
                    Style::default().fg(theme.muted),
                ),
                if sm > 0 {
                    Span::styled(format!(" SM:{}", sm), Style::default().fg(theme.accent))
                } else {
                    Span::raw("")
                },
            ]))
        }).collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(format!(" Trending — {} tokens ", state.trending.len())));
    frame.render_widget(list, area);
}

fn render_detail_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    if let Some(ref detail) = state.selected_token {
        let t = &detail.token;
        let sec = &detail.security;
        let mc_color = marketcap_color(t.market_cap, theme);
        let text = Line::from(vec![
            Span::styled(format!(" {}", t.symbol), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" | Price: ${:.8}", t.price_usd)),
            Span::styled(format!(" | MC: {}", format_marketcap(t.market_cap)), Style::default().fg(mc_color)),
            Span::raw(format!(" | Liq: ${:.0}", t.liquidity_usd)),
            Span::raw(format!(" | Holders: {}", t.holder_count)),
            Span::raw(format!(" | Rug: {:.0}%", sec.rug_ratio * 100.0)),
            Span::raw(" | "),
            Span::styled(
                if sec.renounced_mint { "✅ Mint Renounced" } else { "❌ Mint Not Renounced" },
                Style::default().fg(if sec.renounced_mint { theme.success } else { theme.danger }),
            ),
            Span::raw(" | "),
            Span::styled("[Analyze]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" [Buy] [Watch]"),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent))
            .title(format!(" {} ", t.symbol));
        frame.render_widget(Paragraph::new(text).block(block), area);
    }
}
