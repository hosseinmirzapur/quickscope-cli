use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let text = Text::from(vec![
        Line::from(Span::styled(
            format!(" Scanner — {} items loaded. Press 'r' to refresh, Enter to select.", state.trending.len()),
            Style::default().fg(theme.fg),
        )),
    ]);
    frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), area);
}