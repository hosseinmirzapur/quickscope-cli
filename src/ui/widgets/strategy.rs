use ratatui::{layout::Rect, style::{Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::app::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let text = Text::from(vec![
        Line::from(Span::styled(" Strategy & Learning", Style::default().fg(theme.accent))),
        Line::from(Span::styled("Auto-tuner: View/adjust weights. Post-mortem: Run LLM analysis.", Style::default().fg(theme.fg))),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}