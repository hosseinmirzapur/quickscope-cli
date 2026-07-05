use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Frame,
};

use super::theme::Theme;
use crate::app::AppState;
use crate::data::models::TabIndex;

/// Render the root layout: top bar → tab bar → content → bottom bar.
pub fn render_ui(frame: &mut Frame, state: &AppState) {
    let theme = Theme::from_preset(state.theme_preset.clone());

    // Root split: top bar | tab bar | content | bottom bar
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Top status bar
            Constraint::Length(1),  // Tab bar
            Constraint::Min(3),     // Content
            Constraint::Length(1),  // Bottom keybinding bar
        ])
        .split(frame.area());

    render_top_bar(frame, root[0], state, &theme);
    render_tab_bar(frame, root[1], state, &theme);
    render_content(frame, root[2], state, &theme);
    render_bottom_bar(frame, root[3], state, &theme);
}

/// Top bar: portfolio balance, recording indicator, status
fn render_top_bar(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let pnl_color = if state.daily_pnl >= 0.0 {
        theme.success
    } else {
        theme.danger
    };

    let bar_text = Line::from(vec![
        Span::styled(" ⚡ QuickScope ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw("│ "),
        Span::styled(
            format!("Balance: {:.2} SOL ", state.balance_sol),
            Style::default().fg(theme.fg),
        ),
        Span::styled(
            format!("│ PnL: {:+.2} SOL ", state.daily_pnl),
            Style::default().fg(pnl_color),
        ),
        Span::raw(format!("│ {}", state.status_message)),
    ]);

    let block = Block::default()
        .style(Style::default().bg(theme.card_bg));

    let para = Paragraph::new(bar_text).block(block);
    frame.render_widget(para, area);
}

/// Tab bar: 7 tabs with numbers
fn render_tab_bar(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let tab_labels: Vec<&str> = (0..TabIndex::COUNT)
        .map(|i| TabIndex::from_usize(i).label())
        .collect();

    let selected = state.active_tab.as_usize();
    let tabs = Tabs::new(
        tab_labels.iter().enumerate().map(|(i, label)| {
            let prefix = format!("{} ", i + 1);
            if i == selected {
                Line::from(format!("{}{}", prefix, label))
                    .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
            } else {
                Line::from(format!("{}{}", prefix, label))
                    .style(Style::default().fg(theme.muted))
            }
        }).collect::<Vec<Line>>(),
    )
    .select(selected)
    .style(Style::default().bg(theme.tab_inactive_bg))
    .highlight_style(
        Style::default()
            .bg(theme.tab_active_bg)
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )
    .divider("│");

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border));

    frame.render_widget(tabs.block(block), area);
}

/// Content area — dispatches to the current tab's render function.
fn render_content(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    // Placeholder: render "Active Tab: X" message in content area.
    // Actual tab content implemented in Phase 10.
    let tab_name = state.active_tab.label();
    let text = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {} Tab", tab_name),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Tab content coming in Phase 10...",
                Style::default().fg(theme.muted),
            ),
        ]),
    ]);

    let block = Block::default()
        .borders(Borders::NONE);

    frame.render_widget(Paragraph::new(text).block(block), area);

    // Render modal if active
    if state.show_modal {
        render_modal(frame, area, state, theme);
    }
}

/// Modal dialog overlay
fn render_modal(frame: &mut Frame, content_area: Rect, state: &AppState, theme: &Theme) {
    let modal_w = 60_u16;
    let modal_h = 10_u16;
    let modal_x = content_area.x + (content_area.width.saturating_sub(modal_w)) / 2;
    let modal_y = content_area.y + (content_area.height.saturating_sub(modal_h)) / 2;
    let modal_area = Rect::new(modal_x, modal_y, modal_w, modal_h);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.card_bg));

    let text = Paragraph::new(state.modal_message.as_str())
        .block(block)
        .style(Style::default().fg(theme.fg));

    frame.render_widget(text, modal_area);
}

/// Bottom bar: keybinding hints
fn render_bottom_bar(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let line = Line::from(vec![
        Span::styled(" 1-7:Tabs ", Style::default().fg(theme.muted)),
        Span::styled(" j/k:Nav ", Style::default().fg(theme.muted)),
        Span::styled(" Enter:Select ", Style::default().fg(theme.muted)),
        Span::styled(" r:Refresh ", Style::default().fg(theme.muted)),
        Span::styled(" ?:Help ", Style::default().fg(theme.muted)),
        Span::styled(" q:Quit ", Style::default().fg(theme.danger)),
    ]);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme.border));

    frame.render_widget(Paragraph::new(line).block(block), area);
}