use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme::Theme;
use crate::app::AppState;
use crate::data::models::TabIndex;

/// Tab metadata for the sidebar.
struct TabMeta {
    icon: &'static str,
    label: &'static str,
}

const TAB_META: [TabMeta; 7] = [
    TabMeta { icon: "⬡", label: "Dashboard" },
    TabMeta { icon: "⌕", label: "Scanner" },
    TabMeta { icon: "◎", label: "Analyzer" },
    TabMeta { icon: "⟠", label: "Trade" },
    TabMeta { icon: "☰", label: "Journal" },
    TabMeta { icon: "⚙", label: "Strategy" },
    TabMeta { icon: "◆", label: "Settings" },
];

/// Render the sidebar with tab icons.
/// `sidebar_width` is the width allocated to the sidebar (6 expanded, 3 collapsed).
pub fn render_sidebar(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme, sidebar_width: u16) {
    let collapsed = sidebar_width < 5;
    let mut lines: Vec<Line> = Vec::with_capacity(TabIndex::COUNT);

    // Spacer before first tab
    lines.push(Line::from(Span::raw("")));

    for (i, meta) in TAB_META.iter().enumerate() {
        let is_active = i == state.active_tab.as_usize();
        let bg = theme.sidebar_bg;
        let fg = if is_active { theme.sidebar_active } else { theme.muted };

        // Active indicator bar on the left
        let indicator = if is_active {
            Span::styled("▌", Style::default().fg(theme.sidebar_active))
        } else {
            Span::styled(" ", Style::default().fg(bg))
        };

        let icon_style = Style::default()
            .fg(fg)
            .bg(bg)
            .add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() });

        let icon = Span::styled(meta.icon, icon_style);

        let mut spans = vec![indicator, icon];

        if !collapsed {
            let label_style = Style::default().fg(fg).bg(bg);
            spans.push(Span::raw(" "));
            spans.push(Span::styled(meta.label, label_style));
        }

        // Pad to fill width
        let text_len: usize = spans.iter().map(|s| s.content.len()).sum();
        if text_len < sidebar_width as usize {
            spans.push(Span::raw(" ".repeat(sidebar_width as usize - text_len)));
        }

        lines.push(Line::from(spans));
    }

    // Kill switch indicator at bottom
    if state.risk_state.kill_switch_active && !collapsed {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled("🛑", Style::default().fg(theme.danger)),
        ]));
    }

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.sidebar_bg));

    let para = Paragraph::new(Text::from(lines)).block(block);
    frame.render_widget(para, area);
}

/// Get the next/previous tab index based on the current active tab.
pub fn next_tab(current: usize) -> usize {
    (current + 1) % TabIndex::COUNT
}

pub fn prev_tab(current: usize) -> usize {
    if current == 0 {
        TabIndex::COUNT - 1
    } else {
        current - 1
    }
}

/// Determine which tab was clicked based on the mouse row within the sidebar area.
pub fn sidebar_tab_at(row: u16, area: Rect) -> Option<TabIndex> {
    // First row after top bar is row 0 of sidebar content area
    // Adjust: the sidebar area starts at some y offset
    let rel_y = row.saturating_sub(area.y);
    // Row 0 = spacer, rows 1-7 = tabs
    if (1..=7).contains(&rel_y) {
        let idx = (rel_y - 1) as usize;
        if idx < TabIndex::COUNT {
            return Some(TabIndex::from_usize(idx));
        }
    }
    None
}
