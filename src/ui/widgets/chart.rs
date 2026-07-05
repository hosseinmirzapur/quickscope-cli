use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Widget},
    buffer::Buffer,
};

/// A candlestick OHLCV chart using unicode box-drawing.
/// Renders one row per candle. Green for up, red for down.
pub struct Chart {
    pub candles: Vec<Candle>,
    pub width: u16,
    pub max_price: f64,
    pub min_price: f64,
    pub scroll: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Widget for Chart {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 3 || self.candles.is_empty() {
            return;
        }
        let range = (self.max_price - self.min_price).max(0.0001);
        let start = self.scroll.min(self.candles.len().saturating_sub(area.width as usize));
        let visible = &self.candles[start..(start + (area.width as usize).min(self.candles.len()))];

        for (i, c) in visible.iter().enumerate() {
            let x = area.x + i as u16;
            let is_up = c.close >= c.open;
            let color = if is_up { Color::Rgb(63, 185, 80) } else { Color::Rgb(248, 81, 73) };
            let style = Style::default().fg(color);

            // High-low line
            let hi_y = area.y + ((self.max_price - c.high) / range * (area.height as f64)).round() as u16;
            let lo_y = area.y + ((self.max_price - c.low) / range * (area.height as f64)).round() as u16;
            let hi_y = hi_y.min(area.y + area.height - 1);
            let lo_y = lo_y.min(area.y + area.height - 1);

            for y in hi_y..=lo_y {
                buf.set_string(x, y, "│", style);
            }

            // Open (left tick)
            let open_y = area.y + ((self.max_price - c.open) / range * (area.height as f64)).round() as u16;
            let open_y = open_y.min(area.y + area.height - 1);
            buf.set_string(x.saturating_sub(1), open_y, "┤", style);

            // Close (right tick)
            let close_y = area.y + ((self.max_price - c.close) / range * (area.height as f64)).round() as u16;
            let close_y = close_y.min(area.y + area.height - 1);
            buf.set_string(x, close_y, "├", style);
        }
    }
}