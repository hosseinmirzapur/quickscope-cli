use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme::Theme;
use super::sidebar;
use super::widgets::{Modal, Toast, ToastStyle, command_palette};
use crate::app::AppState;

/// Render the root layout: top bar → [sidebar + content] → bottom bar → overlays
pub fn render_ui(frame: &mut Frame, state: &AppState) {
    let theme = Theme::from_preset(state.theme_preset.clone());
    let full_area = frame.area();

    // Root split: top bar | main area (sidebar + content) | bottom bar
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Top status bar
            Constraint::Min(3),     // Main area
            Constraint::Length(1),  // Bottom keybinding bar
        ])
        .split(full_area);

    render_top_bar(frame, root[0], state, &theme);

    // Main area: sidebar | content
    let sidebar_w = state.sidebar_width();
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_w), Constraint::Min(10)])
        .split(root[1]);

    sidebar::render_sidebar(frame, main[0], state, &theme, sidebar_w);
    render_content(frame, main[1], state, &theme);

    render_bottom_bar(frame, root[2], state, &theme);

    // Toast notification (top-right overlay)
    if let Some(ref msg) = state.active_toast {
        let toast = Toast {
            message: msg.clone(),
            style: ToastStyle::Info,
            remaining_ms: state.toast_remaining_ms,
        };
        // Position in the top-right of the content area
        let toast_area = Rect::new(
            root[1].x + root[1].width.saturating_sub(42),
            root[1].y,
            42,
            1,
        );
        frame.render_widget(toast, toast_area);
    }

    // Overlays (rendered last = on top)
    if state.show_modal {
        render_modal(frame, full_area, state, &theme);
    }
    if state.show_command_palette {
        command_palette::render_command_palette(frame, full_area, state, &theme);
    }
}

/// Top bar: portfolio balance, PnL, status
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
        // Kill switch indicator
        if state.risk_state.kill_switch_active {
            Span::styled("│ 🛑 KILL ", Style::default().fg(theme.danger).add_modifier(Modifier::BOLD))
        } else {
            Span::raw("")
        },
        Span::raw(format!("│ {}", state.status_message)),
    ]);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.card_bg));

    let para = Paragraph::new(bar_text).block(block);
    frame.render_widget(para, area);
}

/// Content area — dispatches to the current tab's render function.
fn render_content(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    use crate::data::models::TabIndex;
    match state.active_tab {
        TabIndex::Dashboard => super::widgets::dashboard::render(frame, area, state, theme),
        TabIndex::Scanner => super::widgets::scanner::render(frame, area, state, theme),
        TabIndex::Analyzer => super::widgets::analyzer::render(frame, area, state, theme),
        TabIndex::TradeTerminal => super::widgets::trade_terminal::render(frame, area, state, theme),
        TabIndex::Journal => super::widgets::journal::render(frame, area, state, theme),
        TabIndex::Strategy => super::widgets::strategy::render(frame, area, state, theme),
        TabIndex::Settings => super::widgets::settings::render(frame, area, state, theme),
    }
}

/// Modal dialog overlay using the Modal widget (with Clear backdrop)
fn render_modal(frame: &mut Frame, full_area: Rect, state: &AppState, theme: &Theme) {
    let title = state.modal_message.lines().next().unwrap_or("QuickScope");
    let body = state.modal_message.lines().skip(1).collect::<Vec<_>>().join("\n");
    let has_confirm = state.modal_message.contains("ENTER to confirm")
        || state.modal_message.contains("Press ENTER");
    let has_cancel = state.modal_message.contains("ESC to cancel")
        || state.modal_message.contains("Esc");

    let modal = Modal {
        title,
        message: &body,
        width: 60,
        height: 12,
        confirm_label: if has_confirm { Some("Enter") } else { None },
        cancel_label: if has_cancel { Some("Esc") } else { None },
        accent_color: theme.accent,
    };
    frame.render_widget(modal, full_area);
}

/// Bottom bar: contextual keybinding hints
fn render_bottom_bar(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let line = Line::from(vec![
        Span::styled(" ↑↓:Navigate ", Style::default().fg(theme.muted)),
        Span::styled(" Enter:Select ", Style::default().fg(theme.muted)),
        Span::styled(" r:Refresh ", Style::default().fg(theme.muted)),
        Span::styled(" Ctrl+P:Commands ", Style::default().fg(theme.accent)),
        Span::styled(" ?:Help ", Style::default().fg(theme.muted)),
        Span::styled(" q:Quit ", Style::default().fg(theme.danger)),
    ]);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme.border));

    frame.render_widget(Paragraph::new(line).block(block), area);
}
