use crate::app::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Strategy & Learning — alpha config, auto-tune history, LLM post-mortem.
pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_alpha_config(frame, chunks[0], state, theme);
    render_strategy_panel(frame, chunks[1], state, theme);
}

fn render_alpha_config(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let ac = &state.alpha_config;
    let text = Text::from(vec![
        Line::from(vec![Span::styled(
            " 🧠 Alpha Weights",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(Span::raw(format!(
            " Momentum:  {:.0}%",
            ac.w_momentum * 100.0
        ))),
        Line::from(Span::raw(format!(
            " Safety:    {:.0}%",
            ac.w_safety * 100.0
        ))),
        Line::from(Span::raw(format!(
            " Holder:    {:.0}%",
            ac.w_holder * 100.0
        ))),
        Line::from(Span::raw(format!(
            " Liquidity: {:.0}%",
            ac.w_liquidity * 100.0
        ))),
        Line::from(Span::raw(format!(" Dev:       {:.0}%", ac.w_dev * 100.0))),
        Line::from(Span::raw(format!(
            " Social:    {:.0}%",
            ac.w_social * 100.0
        ))),
        Line::from(""),
        Line::from(Span::styled(
            " Hard Filters",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw(format!(
            " Max Rug Ratio: {:.0}%",
            ac.hf_rug_ratio_max * 100.0
        ))),
        Line::from(Span::raw(format!(
            " Max Dev Hold: {:.0}%",
            ac.hf_dev_hold_max * 100.0
        ))),
        Line::from(Span::raw(format!(
            " Min Liquidity: ${:.0}",
            ac.hf_liquidity_min_usd
        ))),
        Line::from(Span::raw(format!(
            " Wash Trade Filter: {}",
            if ac.hf_wash_trading { "ON" } else { "OFF" }
        ))),
        Line::from(Span::raw(format!(
            " Renounced Mint Filter: {}",
            if ac.hf_renounced_mint { "ON" } else { "OFF" }
        ))),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [Save Config]  ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "[Run Auto-Tune]",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" ⚙ Configuration "),
        ),
        area,
    );
}

fn render_strategy_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_auto_tune_history(frame, chunks[0], state, theme);
    render_post_mortem_history(frame, chunks[1], state, theme);
}

fn render_auto_tune_history(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let mut lines = vec![
        Line::from(vec![Span::styled(
            " ⚡ Auto-Tune History",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    if state.tuning_history.is_empty() {
        lines.push(Line::from(Span::styled(
            " No auto-tune runs yet. Run from command palette.",
            Style::default().fg(theme.muted),
        )));
    } else {
        // Header
        lines.push(Line::from(vec![
            Span::styled(
                " Date       ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Sample ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " W/L ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Win Rate ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Δ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            " ────────────────────────────────────────────",
            Style::default().fg(theme.muted),
        )));

        for run in state.tuning_history.iter().take(8) {
            let wr = if run.wins + run.losses > 0 {
                run.wins as f64 / (run.wins + run.losses) as f64 * 100.0
            } else {
                0.0
            };
            let wr_color = if wr >= 50.0 {
                theme.success
            } else {
                theme.warning
            };

            // Parse old/new weights to compute a change indicator
            let delta = if let (Ok(old), Ok(new)) = (
                serde_json::from_str::<serde_json::Value>(&run.old_weights),
                serde_json::from_str::<serde_json::Value>(&run.new_weights),
            ) {
                // Sum absolute differences
                let mut total_delta = 0.0;
                for key in [
                    "w_momentum",
                    "w_safety",
                    "w_holder",
                    "w_liquidity",
                    "w_dev",
                    "w_social",
                ] {
                    let old_val = old.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let new_val = new.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    total_delta += (new_val - old_val).abs();
                }
                total_delta
            } else {
                0.0
            };

            lines.push(Line::from(vec![
                Span::raw(format!(" {:>8} ", run.tuned_at.get(5..10).unwrap_or("--"))),
                Span::styled(
                    format!(" {:>4} ", run.sample_size),
                    Style::default().fg(theme.fg),
                ),
                Span::styled(
                    format!("{:>2}/{:<2}", run.wins, run.losses),
                    Style::default().fg(theme.fg),
                ),
                Span::styled(format!(" {:>5.0}%  ", wr), Style::default().fg(wr_color)),
                Span::styled(format!("{:.3}", delta), Style::default().fg(theme.warning)),
            ]));
        }

        // Show latest discrimination analysis if available
        if let Some(latest) = state.tuning_history.first() {
            if let Ok(disc) = serde_json::from_str::<serde_json::Value>(&latest.discrimination) {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    " Latest Discrimination:",
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                )));
                if let Some(obj) = disc.as_object() {
                    for (feature, val) in obj.iter().take(6) {
                        let weight = val.as_f64().unwrap_or(0.0);
                        let bar_len = (weight * 20.0).min(20.0) as usize;
                        let bar = "█".repeat(bar_len);
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!(" {:>12}", feature),
                                Style::default().fg(theme.muted),
                            ),
                            Span::styled(
                                format!(" {:.2} ", bar),
                                Style::default().fg(theme.accent),
                            ),
                            Span::styled(format!("{:.3}", weight), Style::default().fg(theme.fg)),
                        ]));
                    }
                }
            }
        }
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

fn render_post_mortem_history(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let mut lines = vec![
        Line::from(vec![Span::styled(
            " 📋 LLM Post-Mortem",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Provider: [OpenAI] [Anthropic] [Ollama]",
            Style::default().fg(theme.muted),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " [Run Post-Mortem]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    if state.post_mortems.is_empty() {
        lines.push(Line::from(Span::styled(
            " No post-mortem results yet. Run from command palette.",
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(Span::styled(
            " Post-mortems analyze your trading patterns with LLM.",
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(Span::styled(
            " Configure an LLM API key in .env to enable real analysis.",
            Style::default().fg(theme.muted),
        )));
    } else {
        for mortem in state.post_mortems.iter().take(3) {
            let applied = mortem.suggestions_applied;
            let dismissed = mortem.suggestions_dismissed;
            let total = applied + dismissed;
            let accept_rate = if total > 0 {
                applied as f64 / total as f64 * 100.0
            } else {
                0.0
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!(" 📅 {}", mortem.run_at.get(0..10).unwrap_or("--")),
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" │ "),
                Span::styled(
                    format!("{} ({})", mortem.provider, mortem.model),
                    Style::default().fg(theme.muted),
                ),
            ]));
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "   Period: {} → {}",
                    mortem.period_start.get(5..10).unwrap_or("--"),
                    mortem.period_end.get(5..10).unwrap_or("--")
                ),
                Style::default().fg(theme.muted),
            )]));

            // Show truncated response (first 2 lines)
            let response_lines: Vec<&str> = mortem
                .response
                .lines()
                .filter(|l| !l.is_empty())
                .take(3)
                .collect();
            for line in response_lines {
                lines.push(Line::from(Span::styled(
                    format!("   {}", line),
                    Style::default().fg(theme.fg),
                )));
            }

            if total > 0 {
                lines.push(Line::from(vec![
                    Span::styled("   Suggestions: ", Style::default().fg(theme.muted)),
                    Span::styled(
                        format!("{} accepted ", applied),
                        Style::default().fg(theme.success),
                    ),
                    Span::styled(
                        format!("{} dismissed", dismissed),
                        Style::default().fg(theme.danger),
                    ),
                    Span::styled(
                        format!(" ({:.0}% acceptance)", accept_rate),
                        Style::default().fg(theme.muted),
                    ),
                ]));
            }
            lines.push(Line::from(""));
        }
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
