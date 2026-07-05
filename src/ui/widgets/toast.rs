use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

/// A notification toast rendered as a floating bar at the top-right of the content area.
/// Shows for a limited duration and then fades.
pub struct Toast {
    pub message: String,
    pub style: ToastStyle,
    pub remaining_ms: u32,
}

pub enum ToastStyle {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastStyle {
    fn color(&self) -> Color {
        match self {
            ToastStyle::Info => Color::Rgb(88, 166, 255),
            ToastStyle::Success => Color::Rgb(63, 185, 80),
            ToastStyle::Warning => Color::Rgb(210, 153, 34),
            ToastStyle::Error => Color::Rgb(248, 81, 73),
        }
    }

    fn prefix(&self) -> &'static str {
        match self {
            ToastStyle::Info => "ℹ",
            ToastStyle::Success => "✓",
            ToastStyle::Warning => "⚠",
            ToastStyle::Error => "✕",
        }
    }
}

impl Widget for Toast {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let color = self.style.color();
        let mut w = area.width.min(40);
        if self.remaining_ms < 1000 {
            // Fade opacity near end (simplified: shorter width)
            w = (w as f32 * (self.remaining_ms as f32 / 1000.0)) as u16;
        }
        let toast_area = Rect::new(
            area.x + area.width.saturating_sub(w + 2),
            area.y,
            w + 2,
            1,
        );
        let text = Line::from(vec![
            Span::styled(
                format!(" {} ", self.style.prefix()),
                Style::default().fg(color).bg(Color::Rgb(30, 30, 30)),
            ),
            Span::styled(
                &self.message,
                Style::default().fg(color),
            ),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(color).bg(Color::Rgb(20, 20, 20)));
        let para = Paragraph::new(text)
            .block(block)
            .style(Style::default().bg(Color::Rgb(20, 20, 20)));
        para.render(toast_area, buf);
    }
}