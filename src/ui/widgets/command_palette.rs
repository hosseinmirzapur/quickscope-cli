use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::super::theme::Theme;
use crate::app::AppState;
use crate::data::models::{AppCommand, TabIndex, ThemePreset};

/// A command palette entry.
pub struct PaletteEntry {
    pub label: &'static str,
    pub icon: &'static str,
    pub action: PaletteAction,
}

#[derive(Debug, Clone)]
pub enum PaletteAction {
    SwitchTab(TabIndex),
    CycleTheme,
    ToggleSidebar,
    Refresh,
    KillSwitch,
    EmergencyExit,
    RunAutoTune,
    RunPostMortem,
    Search,
    Filter,
    Address,
}

const PALETTE_COMMANDS: &[PaletteEntry] = &[
    PaletteEntry {
        label: "Go to Dashboard",
        icon: "⬡",
        action: PaletteAction::SwitchTab(TabIndex::Dashboard),
    },
    PaletteEntry {
        label: "Go to Scanner",
        icon: "⌕",
        action: PaletteAction::SwitchTab(TabIndex::Scanner),
    },
    PaletteEntry {
        label: "Go to Analyzer",
        icon: "◎",
        action: PaletteAction::SwitchTab(TabIndex::Analyzer),
    },
    PaletteEntry {
        label: "Go to Trade Terminal",
        icon: "⟠",
        action: PaletteAction::SwitchTab(TabIndex::TradeTerminal),
    },
    PaletteEntry {
        label: "Go to Journal",
        icon: "☰",
        action: PaletteAction::SwitchTab(TabIndex::Journal),
    },
    PaletteEntry {
        label: "Go to Strategy",
        icon: "⚙",
        action: PaletteAction::SwitchTab(TabIndex::Strategy),
    },
    PaletteEntry {
        label: "Go to Settings",
        icon: "◆",
        action: PaletteAction::SwitchTab(TabIndex::Settings),
    },
    PaletteEntry {
        label: "Cycle Theme",
        icon: "🎨",
        action: PaletteAction::CycleTheme,
    },
    PaletteEntry {
        label: "Toggle Sidebar",
        icon: "⊞",
        action: PaletteAction::ToggleSidebar,
    },
    PaletteEntry {
        label: "Refresh Data",
        icon: "↻",
        action: PaletteAction::Refresh,
    },
    PaletteEntry {
        label: "Search Tokens",
        icon: "🔍",
        action: PaletteAction::Search,
    },
    PaletteEntry {
        label: "Filter Tokens",
        icon: "⚙",
        action: PaletteAction::Filter,
    },
    PaletteEntry {
        label: " by Address",
        icon: "🔎",
        action: PaletteAction::Address,
    },
    PaletteEntry {
        label: "Toggle Kill Switch",
        icon: "🔒",
        action: PaletteAction::KillSwitch,
    },
    PaletteEntry {
        label: "Emergency Exit All",
        icon: "⚠",
        action: PaletteAction::EmergencyExit,
    },
    PaletteEntry {
        label: "Run Auto-Tune",
        icon: "⚡",
        action: PaletteAction::RunAutoTune,
    },
    PaletteEntry {
        label: "Run Post-Mortem",
        icon: "📋",
        action: PaletteAction::RunPostMortem,
    },
];

/// Render the command palette as a centered overlay with a dimmed backdrop.
pub fn render_command_palette(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    // 1. Dim the entire screen by drawing a dark backdrop
    let backdrop = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));
    frame.render_widget(backdrop, area);

    // 2. Calculate centered palette area
    let w = 54_u16.min(area.width.saturating_sub(8));
    let h = 22_u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let palette_area = Rect::new(x, y, w, h);

    // Clear the palette area first
    frame.render_widget(Clear, palette_area);

    // 3. Filter commands
    let query = state.palette_filter.to_lowercase();
    let filtered: Vec<&PaletteEntry> = if query.is_empty() {
        PALETTE_COMMANDS.iter().collect()
    } else {
        PALETTE_COMMANDS
            .iter()
            .filter(|e| e.label.to_lowercase().contains(&query))
            .collect()
    };

    let cursor = if filtered.is_empty() {
        0
    } else {
        state.palette_cursor % filtered.len()
    };

    // 4. Build lines: title + search bar + results + footer
    let mut lines: Vec<Line> = Vec::new();

    // Search bar
    lines.push(Line::from(vec![
        Span::styled(" 🔍 ", Style::default().fg(theme.accent)),
        Span::styled(&state.palette_filter, Style::default().fg(theme.palette_fg)),
        Span::styled("█", Style::default().fg(theme.accent_dim)),
    ]));
    lines.push(Line::from(Span::styled(
        format!(" {} ", "─".repeat(w.saturating_sub(2) as usize)),
        Style::default().fg(theme.border),
    )));

    if filtered.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " No matching commands",
            Style::default().fg(theme.muted),
        )));
    } else {
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

            let icon_style = if selected {
                Style::default()
                    .bg(theme.palette_highlight)
                    .fg(theme.accent)
            } else {
                Style::default().fg(theme.accent_dim)
            };

            let prefix = if selected { "▶ " } else { "  " };

            lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.accent)),
                Span::styled(format!(" {} ", entry.icon), icon_style),
                Span::styled(entry.label, style),
            ]));
        }
    }

    // Footer with hints
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!(" {} ", "─".repeat(w.saturating_sub(2) as usize)),
        Style::default().fg(theme.border),
    )]));
    lines.push(Line::from(vec![
        Span::styled(" ↑↓ ", Style::default().fg(theme.muted)),
        Span::styled("Navigate   ", Style::default().fg(theme.muted)),
        Span::styled("Enter ", Style::default().fg(theme.muted)),
        Span::styled("Select   ", Style::default().fg(theme.muted)),
        Span::styled("Esc ", Style::default().fg(theme.muted)),
        Span::styled("Close", Style::default().fg(theme.muted)),
    ]));

    let block = Block::default()
        .title(" ⌨ Command Palette ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.palette_bg));

    let para = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(para, palette_area);
}

/// Execute the currently selected palette action and return the AppCommand.
pub fn execute_palette_selection(state: &mut AppState) -> Option<AppCommand> {
    let query = state.palette_filter.to_lowercase();
    let filtered: Vec<&PaletteEntry> = if query.is_empty() {
        PALETTE_COMMANDS.iter().collect()
    } else {
        PALETTE_COMMANDS
            .iter()
            .filter(|e| e.label.to_lowercase().contains(&query))
            .collect()
    };

    if filtered.is_empty() {
        return None;
    }

    let cursor = state.palette_cursor % filtered.len();
    let entry = filtered[cursor];

    match &entry.action {
        PaletteAction::SwitchTab(tab) => {
            state.switch_tab(*tab);
            state.show_command_palette = false;
            Some(AppCommand::ShowModal("".to_string()))
        }
        PaletteAction::CycleTheme => {
            state.theme_preset = match state.theme_preset {
                ThemePreset::Dark => ThemePreset::Terminal,
                ThemePreset::Terminal => ThemePreset::Degen,
                ThemePreset::Degen => ThemePreset::Cyberpunk,
                ThemePreset::Cyberpunk => ThemePreset::Dark,
            };
            state.show_command_palette = false;
            Some(AppCommand::ShowModal("".to_string()))
        }
        PaletteAction::ToggleSidebar => {
            state.toggle_sidebar();
            state.show_command_palette = false;
            Some(AppCommand::ShowModal("".to_string()))
        }
        PaletteAction::Refresh => {
            state.show_command_palette = false;
            Some(AppCommand::FetchTrending)
        }
        PaletteAction::Search => {
            state.show_command_palette = false;
            state.input_active = true;
            state.input_buffer.clear();
            state.set_status("Search: type query, Enter to submit, Esc to cancel");
            None
        }
        PaletteAction::Address => {
            state.show_command_palette = false;
            state.input_active = true;
            state.input_buffer.clear();
            state.address_search_mode = true;
            state.set_status(" by Address: paste contract address, Enter to lookup");
            Some(AppCommand::ShowModal("".to_string()))
        }
        PaletteAction::Filter => {
            state.show_command_palette = false;
            state.show_modal = true;
            state.modal_message = "Filter tokens\n\nToggle filters to narrow the token list.\nActive filters apply to all views.\n\nPress ENTER to apply, ESC to cancel.".to_string();
            Some(AppCommand::ShowModal("".to_string()))
        }
        PaletteAction::KillSwitch => {
            state.show_command_palette = false;
            Some(AppCommand::ToggleKillSwitch)
        }
        PaletteAction::EmergencyExit => {
            state.show_command_palette = false;
            state.show_modal = true;
            state.modal_message = "Emergency Exit All\n\nClose ALL open positions at market price?\n\nPress ENTER to confirm, ESC to cancel.".to_string();
            Some(AppCommand::ShowModal("".to_string()))
        }
        PaletteAction::RunAutoTune => {
            state.show_command_palette = false;
            Some(AppCommand::RunAutoTune)
        }
        PaletteAction::RunPostMortem => {
            state.show_command_palette = false;
            Some(AppCommand::RunPostMortem(
                (chrono::Utc::now() - chrono::Duration::days(7)).to_rfc3339(),
                chrono::Utc::now().to_rfc3339(),
            ))
        }
    }
}
