use ratatui::{layout::Rect, style::{Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::app::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let text = Text::from(vec![
        Line::from(Span::styled(" Settings", Style::default().fg(theme.accent))),
        Line::from(Span::raw("")),
        Line::from(Span::raw("1. Theme: Dark / Degen")),
        Line::from(Span::raw("2. API: GMGN key + Alph AI cookie")),
        Line::from(Span::raw("3. Risk: loss cap, per-trade max, tp/sl config")),
        Line::from(Span::raw("4. Display: notification preferences")),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}