use crate::app::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Alpha Analyzer — deep-dive on a single token.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    if state.selected_token.is_none() {
        let text = Text::from(vec![
            Line::from(Span::styled(
                " Alpha Analyzer",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Select a token from Scanner or Dashboard to analyze here.",
                Style::default().fg(theme.muted),
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
        return;
    }

    let detail = state.selected_token.as_ref().unwrap();
    let t = &detail.token;
    let sec = &detail.security;
    let ps = &detail.price_stats;
    let dev = &detail.dev_info;

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Left: price stats + sparkline
    let mut left = vec![
        Line::from(vec![Span::styled(
            format!(" {} ({})", t.symbol, t.name),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(Span::raw(format!(
            " Address: {}..{}",
            &t.address[..8],
            &t.address[t.address.len() - 4..]
        ))),
        Line::from(""),
        Line::from(Span::styled(
            " Price Stats",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )),
    ];
    for (label, val) in [
        ("1m", ps.change_1m),
        ("5m", ps.change_5m),
        ("1h", ps.change_1h),
    ] {
        if let Some(v) = val {
            let c = if v >= 0.0 {
                theme.success
            } else {
                theme.danger
            };
            left.push(Line::from(vec![
                Span::raw(format!(" {}: ", label)),
                Span::styled(format!("{:+.2}%", v), Style::default().fg(c)),
            ]));
        }
    }
    left.push(Line::from(""));
    left.push(Line::from(Span::styled(
        format!(" MC: ${:.0}", t.market_cap),
        Style::default().fg(theme.fg),
    )));
    left.push(Line::from(Span::styled(
        format!(" Liq: ${:.0}", t.liquidity_usd),
        Style::default().fg(theme.fg),
    )));
    left.push(Line::from(Span::styled(
        format!(" Holders: {}", t.holder_count),
        Style::default().fg(theme.fg),
    )));
    left.push(Line::from(Span::styled(
        format!(" Price: ${:.8}", t.price_usd),
        Style::default().fg(theme.fg),
    )));
    frame.render_widget(
        Paragraph::new(Text::from(left)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        ),
        top[0],
    );

    // Middle: Alpha Score + Rug
    let score = state.alpha_report.as_ref();
    let mut mid = vec![Line::from(Span::styled(
        " Alpha Analysis",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))];
    if let Some(r) = score {
        mid.push(Line::from(""));
        mid.push(Line::from(vec![
            Span::styled(" Alpha Score: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{:.1}", r.alpha_score),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        mid.push(Line::from(vec![Span::styled(
            format!(" Mode: {}", r.mode.as_str()),
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )]));
        mid.push(Line::from(""));
        mid.push(Line::from(Span::styled(
            " Rug Report:",
            Style::default().fg(theme.danger),
        )));
        mid.push(Line::from(Span::styled(
            format!("  Severity: {:?}", r.rug_report.severity),
            Style::default().fg(theme.danger),
        )));
        mid.push(Line::from(Span::styled(
            format!("  {}", r.rug_report.verdict),
            Style::default().fg(theme.muted),
        )));
        if let Some(ref n) = r.narrative {
            mid.push(Line::from(""));
            mid.push(Line::from(Span::styled(
                format!(" Narrative: {}", n),
                Style::default().fg(theme.accent),
            )));
        }
    } else {
        mid.push(Line::from(""));
        mid.push(Line::from(Span::styled(
            " Press Enter on a token to see its Alpha Score.",
            Style::default().fg(theme.muted),
        )));
    }
    frame.render_widget(
        Paragraph::new(Text::from(mid)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        ),
        top[1],
    );

    // Right: Security audit
    let right = vec![
        Line::from(Span::styled(
            " Security Audit",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw(" Rug Ratio: "),
            Span::styled(
                format!("{:.1}%", sec.rug_ratio * 100.0),
                Style::default().fg(if sec.rug_ratio > 0.3 {
                    theme.danger
                } else {
                    theme.success
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Wash Trade: "),
            Span::styled(
                if sec.is_wash_trading { "YES" } else { "NO" },
                Style::default().fg(if sec.is_wash_trading {
                    theme.danger
                } else {
                    theme.success
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Mint Renounced: "),
            Span::styled(
                if sec.renounced_mint { "YES" } else { "NO" },
                Style::default().fg(if sec.renounced_mint {
                    theme.success
                } else {
                    theme.danger
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Freeze Renounced: "),
            Span::styled(
                if sec.renounced_freeze { "YES" } else { "NO" },
                Style::default().fg(if sec.renounced_freeze {
                    theme.success
                } else {
                    theme.danger
                }),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!(" Top 10 Holders: {:.1}%", sec.top_10_holder_rate * 100.0),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!(" Dev Hold: {:.1}%", sec.dev_team_hold_rate * 100.0),
            Style::default().fg(theme.fg),
        )),
        Line::from(Span::styled(
            format!(" Creator: {:?}", sec.creator_status),
            Style::default().fg(theme.fg),
        )),
        Line::from(if dev.cto_flag {
            Span::styled(" CTO Detected", Style::default().fg(theme.warning))
        } else {
            Span::raw("")
        }),
    ];
    frame.render_widget(
        Paragraph::new(Text::from(right)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        ),
        top[2],
    );
}
