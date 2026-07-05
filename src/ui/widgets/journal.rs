use ratatui::{layout::Rect, style::{Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::app::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let text = Text::from(vec![
        Line::from(Span::styled(" Journal — Trade history", Style::default().fg(theme.fg))),
        Line::from(Span::styled(format!("Today: {} trades, PnL: {:+.2} SOL", 0, state.daily_pnl), Style::default().fg(theme.accent))),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}