use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    buffer::Buffer,
};

/// An inline search bar activated by `/` key.
pub struct SearchBar<'a> {
    pub query: &'a str,
    pub active: bool,
    pub result_count: usize,
}

impl Widget for SearchBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 {
            return;
        }
        let (fg, bg) = if self.active {
            (Color::Rgb(88, 166, 255), Color::Rgb(22, 27, 34))
        } else {
            (Color::Rgb(139, 148, 158), Color::Rgb(22, 27, 34))
        };
        let cursor = if self.active { "█" } else { "" };
        let result_str = if self.result_count > 0 {
            format!(" ({})", self.result_count)
        } else {
            String::new()
        };
        let text = Line::from(vec![
            Span::styled("/", Style::default().fg(fg)),
            Span::styled(self.query, Style::default().fg(fg)),
            Span::styled(cursor, Style::default().fg(fg).bg(bg)),
            Span::styled(result_str, Style::default().fg(Color::Rgb(139, 148, 158))),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(48, 54, 61)));
        let para = Paragraph::new(text).block(block).style(Style::default().bg(bg));
        para.render(area, buf);
    }
}