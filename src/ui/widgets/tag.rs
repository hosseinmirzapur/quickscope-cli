use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Paragraph, Widget},
    buffer::Buffer,
};

/// A colored tag/chip, e.g. `[ Pump.fun ]`.
pub struct Tag {
    pub label: String,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
}

impl Tag {
    pub fn new(label: &str, fg: Color, bg: Color) -> Self {
        Self {
            label: label.to_string(),
            fg,
            bg,
            bold: false,
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
}

impl Widget for Tag {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = format!(" {} ", self.label);
        let mut style = Style::default().fg(self.fg).bg(self.bg);
        if self.bold {
            style = style.add_modifier(ratatui::style::Modifier::BOLD);
        }
        let para = Paragraph::new(text).style(style);
        para.render(area, buf);
    }
}