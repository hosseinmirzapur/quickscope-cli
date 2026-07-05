use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
    buffer::Buffer,
};

/// A centered modal overlay with title, message, and optional confirm/cancel.
pub struct Modal<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub width: u16,
    pub height: u16,
    pub confirm_label: Option<&'a str>,
    pub cancel_label: Option<&'a str>,
    pub accent_color: Color,
}

impl Widget for Modal<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Backdrop
        Clear.render(area, buf);

        let w = self.width.min(area.width.saturating_sub(4));
        let h = self.height.min(area.height.saturating_sub(4));
        let x = area.x + (area.width - w) / 2;
        let y = area.y + (area.height - h) / 2;
        let modal_area = Rect::new(x, y, w, h);

        let mut text = Text::from(self.message);

        // Action hint at bottom
        if self.confirm_label.is_some() || self.cancel_label.is_some() {
            let mut hint = String::from("\n\n");
            if let Some(c) = self.confirm_label {
                hint.push_str(&format!(" [{}] ", c));
            }
            if let Some(c) = self.cancel_label {
                hint.push_str(&format!(" [{}] ", c));
            }
            text.push_line(hint);
        }

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.accent_color))
            .style(Style::default().bg(Color::Rgb(22, 27, 34)));

        let para = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::Rgb(201, 209, 217)))
            .alignment(ratatui::layout::Alignment::Center);

        para.render(modal_area, buf);
    }
}