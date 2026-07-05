use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Widget},
    buffer::Buffer,
};

/// A mini sparkline chart using unicode block characters (▁▂▃▄▅▆▇█).
pub struct SparklineWidget {
    pub data: Vec<f64>,
    pub color: Color,
    pub max: Option<f64>,
}

impl Widget for SparklineWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || self.data.is_empty() {
            return;
        }
        let n = (area.width as usize).min(self.data.len());
        let data: Vec<f64> = self.data[self.data.len() - n..].to_vec();
        let max = self.max.unwrap_or_else(|| data.iter().cloned().fold(0.0_f64, f64::max));
        if max <= 0.0 {
            return;
        }

        const BLOCKS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        for (i, &val) in data.iter().enumerate() {
            let idx = ((val / max) * 8.0).round() as usize;
            let ch = BLOCKS[idx.min(8)];
            let x = area.x + i as u16;
            buf.set_string(x, area.y, ch.to_string(), Style::default().fg(self.color));
        }
    }
}