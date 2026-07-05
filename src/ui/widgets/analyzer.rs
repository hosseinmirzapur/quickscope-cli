use ratatui::{layout::Rect, style::{Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::app::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let text = Text::from(vec![
        Line::from(Span::styled(" Analyzer — Token detail view", Style::default().fg(theme.fg))),
        Line::from(Span::styled(
            match &state.selected_token { Some(t) => format!("Selected: {} (${:.8})", t.token.symbol, t.token.price_usd), None => "No token selected".into() },
            Style::default().fg(theme.accent),
        )),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}