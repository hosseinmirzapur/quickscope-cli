use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Paragraph, Widget},
    buffer::Buffer,
};

/// A horizontal progress bar with label.
pub struct ProgressBar<'a> {
    pub ratio: f64,
    pub width: u16,
    pub label: Option<&'a str>,
    pub good_color: Color,
    pub bad_color: Color,
}

impl Widget for ProgressBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let w = self.width.min(area.width);
        let filled = ((self.ratio * w as f64).round() as u16).max(0);
        let color = if self.ratio > 0.5 {
            lerp_color(self.bad_color, self.good_color, (self.ratio - 0.5) * 2.0)
        } else {
            lerp_color(self.bad_color, self.good_color, self.ratio * 2.0)
        };
        let style = Style::default().fg(color);

        if let Some(label) = self.label {
            if w > label.len() as u16 + 5 {
                let fill_len = (w as usize).saturating_sub(label.len() + 5);
                let fill = "█".repeat(filled as usize);
                let empty = "░".repeat(fill_len.saturating_sub(filled as usize));
                let display = format!("{}{} {} {:.0}%", &fill[..fill.len().min(fill_len)], empty, label, self.ratio * 100.0);
                Paragraph::new(display).style(style).render(area, buf);
                return;
            }
        }
        let fill = "█".repeat(filled as usize);
        let empty = "░".repeat((w - filled) as usize);
        Paragraph::new(format!("{}{}", fill, empty)).style(style).render(area, buf);
    }
}

fn lerp_color(a: Color, b: Color, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    match (a, b) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => Color::Rgb(
            (r1 as f64 + (r2 as f64 - r1 as f64) * t) as u8,
            (g1 as f64 + (g2 as f64 - g1 as f64) * t) as u8,
            (b1 as f64 + (b2 as f64 - b1 as f64) * t) as u8,
        ),
        _ => b,
    }
}

impl<'a> Default for ProgressBar<'a> {
    fn default() -> Self {
        Self {
            ratio: 0.0,
            width: 10,
            label: None,
            good_color: Color::Rgb(63, 185, 80),
            bad_color: Color::Rgb(248, 81, 73),
        }
    }
}
