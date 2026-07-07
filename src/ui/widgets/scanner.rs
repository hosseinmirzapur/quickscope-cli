use crate::app::state::TokenListMode;
use crate::app::AppState;
use crate::ui::format::{format_marketcap, format_volume, marketcap_color};
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Token Scanner with mode selector, full list, and detail panel.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // mode selector + filter bar
            Constraint::Min(5),    // token list
            Constraint::Length(7), // detail panel (when selected)
        ])
        .split(area);

    render_mode_selector(frame, chunks[0], state, theme);
    match state.list_mode {
        TokenListMode::Trenches => render_trench_list(frame, chunks[1], state, theme),
        _ => render_token_list(frame, chunks[1], state, theme),
    }
    if state.selected_token.is_some() {
        render_detail_panel(frame, chunks[2], state, theme);
    }
}

/// Render mode selector bar — clickable labels switch between data sources.
fn render_mode_selector(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let modes = [
        ("Trending", TokenListMode::Trending),
        ("Trenches", TokenListMode::Trenches),
        ("Watchlist", TokenListMode::Watchlist),
        ("AI-Rec", TokenListMode::AiRec),
    ];

    let mut spans: Vec<Span> = Vec::new();
    for (i, (label, mode)) in modes.iter().enumerate() {
        let is_active = *mode == state.list_mode;
        let fg = if is_active { theme.accent } else { theme.muted };
        let indicator = if is_active { "▸" } else { " " };
        spans.push(Span::styled(
            format!("{} {} ", indicator, label),
            Style::default().fg(fg).add_modifier(if is_active {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
        ));
        if i < modes.len() - 1 {
            spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
        }
    }

    // Count and keyboard hint
    let count = state.current_list().len();
    if state.list_mode == TokenListMode::Trenches {
        spans.push(Span::styled(
            format!(" ({})", state.trenches.len()),
            Style::default().fg(theme.muted),
        ));
    } else {
        spans.push(Span::styled(
            format!(" ({})", count),
            Style::default().fg(theme.muted),
        ));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));
    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

/// Render token list for Trending / Watchlist / AI-Rec modes.
fn render_token_list(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let visible_rows = area.height.saturating_sub(2) as usize;
    let tokens = state.current_list();
    let max_offset = tokens.len().saturating_sub(visible_rows).saturating_sub(1);
    let offset = if tokens.is_empty() {
        0
    } else {
        state.scroll_offset.min(max_offset)
    };
    let visible_tokens: Vec<&&crate::data::models::TrendingToken> =
        tokens.iter().skip(offset).take(visible_rows).collect();

    let items: Vec<ListItem> = if state.loading_trending {
        vec![ListItem::new(Line::from(Span::styled(
            " ◌ Fetching data...",
            Style::default().fg(theme.warning),
        )))]
    } else if visible_tokens.is_empty() {
        let msg = match state.list_mode {
            TokenListMode::Watchlist => " Watchlist is empty — press Space to add tokens",
            TokenListMode::AiRec => " No AI recommendations yet — wait for signals",
            _ => {
                if state.input_active && !state.input_buffer.is_empty() {
                    " No matches — try a different search"
                } else {
                    " Press 'r' to load trending tokens"
                }
            }
        };
        vec![ListItem::new(Line::from(Span::styled(
            msg,
            Style::default().fg(theme.muted),
        )))]
    } else {
        visible_tokens
            .iter()
            .enumerate()
            .map(|(rel_i, t)| {
                let abs_i = offset + rel_i;
                let sel = abs_i == state.list_cursor;
                let s = if sel {
                    Style::default().bg(theme.highlight).fg(theme.accent)
                } else {
                    Style::default().fg(theme.fg)
                };
                let chg = t.change_1h.unwrap_or(0.0);
                let cc = if chg >= 0.0 {
                    theme.success
                } else {
                    theme.danger
                };
                let mc_color = marketcap_color(t.market_cap, theme);
                let vol = t.volume_1h.unwrap_or(0.0);
                let sm = t.smart_degen_count.unwrap_or(0);

                ListItem::new(Line::from(vec![
                    if sel {
                        Span::styled(" ▶", Style::default().fg(theme.accent))
                    } else {
                        Span::raw("  ")
                    },
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
                    Span::styled(
                        format!("{:+.1}%", chg),
                        Style::default().fg(cc).add_modifier(Modifier::BOLD),
                    ),
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
            })
            .collect()
    };

    let title_prefix = match state.list_mode {
        TokenListMode::Watchlist => "⭐ Watchlist",
        TokenListMode::AiRec => "🤖 AI-Rec",
        _ => "📊 Trending",
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(format!(" {} — {} tokens ", title_prefix, tokens.len())),
    );
    frame.render_widget(list, area);
}

/// Render trenches (newly launched tokens).
fn render_trench_list(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let visible_rows = area.height.saturating_sub(2) as usize;
    let tokens = &state.trenches;
    let max_offset = tokens.len().saturating_sub(visible_rows).saturating_sub(1);
    let offset = if tokens.is_empty() {
        0
    } else {
        state.scroll_offset.min(max_offset)
    };
    let visible: Vec<&crate::data::models::TrenchToken> =
        tokens.iter().skip(offset).take(visible_rows).collect();

    let items: Vec<ListItem> = if visible.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            " No trenches loaded — press 'r' to fetch",
            Style::default().fg(theme.muted),
        )))]
    } else {
        visible
            .iter()
            .enumerate()
            .map(|(rel_i, t)| {
                let abs_i = offset + rel_i;
                let sel = abs_i == state.list_cursor;
                let s = if sel {
                    Style::default().bg(theme.highlight).fg(theme.accent)
                } else {
                    Style::default().fg(theme.fg)
                };
                let mc_color = marketcap_color(t.market_cap, theme);

                ListItem::new(Line::from(vec![
                    if sel {
                        Span::styled(" ▶", Style::default().fg(theme.accent))
                    } else {
                        Span::raw("  ")
                    },
                    Span::raw(" "),
                    Span::styled(format!("{:>8}", t.symbol), s.add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(
                        format_marketcap(t.market_cap),
                        Style::default().fg(mc_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:>3}m", t.age_minutes),
                        Style::default().fg(theme.muted),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("${:.8}", t.price_usd),
                        Style::default().fg(theme.muted),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        t.platform.chars().take(10).collect::<String>(),
                        Style::default().fg(theme.accent_dim),
                    ),
                    if t.smart_holding > 0 {
                        Span::styled(
                            format!(" SM:{}", t.smart_holding),
                            Style::default().fg(theme.accent),
                        )
                    } else {
                        Span::raw("")
                    },
                ]))
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(format!(" 🆕 Trenches — {} tokens ", tokens.len())),
    );
    frame.render_widget(list, area);
}

fn render_detail_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    if let Some(ref detail) = state.selected_token {
        let t = &detail.token;
        let sec = &detail.security;
        let mc_color = marketcap_color(t.market_cap, theme);
        let text = Line::from(vec![
            Span::styled(
                format!(" {}", t.symbol),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" | Price: ${:.8}", t.price_usd)),
            Span::styled(
                format!(" | MC: {}", format_marketcap(t.market_cap)),
                Style::default().fg(mc_color),
            ),
            Span::raw(format!(" | Liq: ${:.0}", t.liquidity_usd)),
            Span::raw(format!(" | Holders: {}", t.holder_count)),
            Span::raw(format!(" | Rug: {:.0}%", sec.rug_ratio * 100.0)),
            Span::raw(" | "),
            Span::styled(
                if sec.renounced_mint {
                    "✅ Mint Renounced"
                } else {
                    "❌ Mint Not Renounced"
                },
                Style::default().fg(if sec.renounced_mint {
                    theme.success
                } else {
                    theme.danger
                }),
            ),
            Span::raw(" | "),
            Span::styled(
                "[Analyze]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" [Buy] [Watch]"),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent))
            .title(format!(" {} ", t.symbol));
        frame.render_widget(Paragraph::new(text).block(block), area);
    }
}
