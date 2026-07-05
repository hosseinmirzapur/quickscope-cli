use ratatui::{layout::Rect, style::{Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::app::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let pos_count = state.open_positions.len();
    let text = Text::from(vec![
        Line::from(Span::styled(" Trade Terminal", Style::default().fg(theme.accent))),
        Line::from(Span::styled(format!("Open positions: {}. Press Enter to select, 'b' to buy, 's' to sell", pos_count), Style::default().fg(theme.fg))),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}