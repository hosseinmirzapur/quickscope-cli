use ratatui::{
    layout::{Alignment, Rect},
    prelude::Widget,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::AppState;
use crate::data::models::{AppCommand, TabIndex, ThemePreset};
use super::super::theme::Theme;

/// A command palette entry.
pub struct PaletteEntry {
    pub label: &'static str,
    pub icon: &'static str,
    pub action: PaletteAction,
}

#[derive(Debug, Clone)]
pub enum PaletteAction {
    SwitchTab(TabIndex),
    ToggleTheme,
    ToggleSidebar,
    Refresh,
    KillSwitch,
    EmergencyExit,
    RunAutoTune,
    RunPostMortem,
}

const PALETTE_COMMANDS: &[PaletteEntry] = &[
    PaletteEntry { label: "Go to Dashboard",    icon: "⬡", action: PaletteAction::SwitchTab(TabIndex::Dashboard) },
    PaletteEntry { label: "Go to Scanner",      icon: "⌕", action: PaletteAction::SwitchTab(TabIndex::Scanner) },
    PaletteEntry { label: "Go to Analyzer",      icon: "◎", action: PaletteAction::SwitchTab(TabIndex::Analyzer) },
    PaletteEntry { label: "Go to Trade Terminal", icon: "⟠", action: PaletteAction::SwitchTab(TabIndex::TradeTerminal) },
    PaletteEntry { label: "Go to Journal",       icon: "☰", action: PaletteAction::SwitchTab(TabIndex::Journal) },
    PaletteEntry { label: "Go to Strategy",      icon: "⚙", action: PaletteAction::SwitchTab(TabIndex::Strategy) },
    PaletteEntry { label: "Go to Settings",      icon: "◆", action: PaletteAction::SwitchTab(TabIndex::Settings) },
    PaletteEntry { label: "Toggle Theme",        icon: "🎨", action: PaletteAction::ToggleTheme },
    PaletteEntry { label: "Toggle Sidebar",      icon: "⊞", action: PaletteAction::ToggleSidebar },
    PaletteEntry { label: "Refresh Data",        icon: "↻", action: PaletteAction::Refresh },
    PaletteEntry { label: "Toggle Kill Switch",  icon: "🔒", action: PaletteAction::KillSwitch },
    PaletteEntry { label: "Emergency Exit All",  icon: "⚠", action: PaletteAction::EmergencyExit },
    PaletteEntry { label: "Run Auto-Tune",       icon: "⚡", action: PaletteAction::RunAutoTune },
    PaletteEntry { label: "Run Post-Mortem",     icon: "📋", action: PaletteAction::RunPostMortem },
];

/// Render the command palette as a centered overlay.
pub fn render_command_palette(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    // Backdrop
    Clear.render(area, frame.buffer_mut());

    let w = 50_u16.min(area.width.saturating_sub(8));
    let h = 20_u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let palette_area = Rect::new(x, y, w, h);

    // Filter the commands
    let query = state.palette_filter.to_lowercase();
    let filtered: Vec<&PaletteEntry> = if query.is_empty() {
        PALETTE_COMMANDS.iter().collect()
    } else {
        PALETTE_COMMANDS.iter()
            .filter(|e| e.label.to_lowercase().contains(&query))
            .collect()
    };

    let cursor = state.palette_cursor.min(filtered.len().saturating_sub(1));

    // Build lines: search bar + results
    let mut lines = vec![
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(theme.accent)),
            Span::styled(&state.palette_filter, Style::default().fg(theme.palette_fg)),
            Span::styled("█", Style::default().fg(theme.accent_dim)),
        ]),
        Line::from(""),
    ];

    for (i, entry) in filtered.iter().enumerate() {
        let selected = i == cursor;
        let style = if selected {
            Style::default()
                .bg(theme.palette_highlight)
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.palette_fg)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} {}", entry.icon, entry.label),
                style,
            ),
        ]));
    }

    let block = Block::default()
        .title(" ⌨ Commands ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent_dim))
        .style(Style::default().bg(theme.palette_bg));

    let para = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(Clear, palette_area);
    frame.render_widget(para, palette_area);
}

/// Execute the currently selected palette action and return the AppCommand.
pub fn execute_palette_selection(state: &mut AppState) -> Option<AppCommand> {
    let query = state.palette_filter.to_lowercase();
    let filtered: Vec<&PaletteEntry> = if query.is_empty() {
        PALETTE_COMMANDS.iter().collect()
    } else {
        PALETTE_COMMANDS.iter()
            .filter(|e| e.label.to_lowercase().contains(&query))
            .collect()
    };

    let cursor = state.palette_cursor.min(filtered.len().saturating_sub(1));
    filtered.get(cursor).map(|entry| match &entry.action {
        PaletteAction::SwitchTab(tab) => {
            state.switch_tab(*tab);
            state.show_command_palette = false;
            AppCommand::ShowModal("".to_string()) // no-op, just closes palette
        }
        PaletteAction::ToggleTheme => {
            state.theme_preset = match state.theme_preset {
                ThemePreset::Dark => ThemePreset::Degen,
                ThemePreset::Degen => ThemePreset::Dark,
            };
            state.show_command_palette = false;
            AppCommand::ShowModal("".to_string())
        }
        PaletteAction::ToggleSidebar => {
            state.toggle_sidebar();
            state.show_command_palette = false;
            AppCommand::ShowModal("".to_string())
        }
        PaletteAction::Refresh => {
            state.show_command_palette = false;
            AppCommand::FetchTrending
        }
        PaletteAction::KillSwitch => {
            state.show_command_palette = false;
            AppCommand::ToggleKillSwitch
        }
        PaletteAction::EmergencyExit => {
            state.show_command_palette = false;
            state.show_modal = true;
            state.modal_message = "⚠️  Emergency Exit All\n\nClose ALL open positions at market price?\n\nPress ENTER to confirm, ESC to cancel.".to_string();
            AppCommand::ShowModal("".to_string())
        }
        PaletteAction::RunAutoTune => {
            state.show_command_palette = false;
            AppCommand::RunAutoTune
        }
        PaletteAction::RunPostMortem => {
            state.show_command_palette = false;
            AppCommand::RunPostMortem(
                (chrono::Utc::now() - chrono::Duration::days(7)).to_rfc3339(),
                chrono::Utc::now().to_rfc3339(),
            )
        }
    })
}
