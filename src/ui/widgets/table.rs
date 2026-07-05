use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
    buffer::Buffer,
};

/// A scrollable table with headers, alternating row colors, and cursor.
pub struct Table<'a> {
    pub headers: Vec<&'a str>,
    pub rows: Vec<Vec<&'a str>>,
    pub column_widths: Vec<u16>,
    pub cursor: Option<usize>,
    pub scroll: usize,
    pub title: Option<&'a str>,
    pub highlight_color: Color,
    pub header_color: Color,
    pub row_colors: (Color, Color), // even, odd
}

impl Widget for Table<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 2 || area.height < 2 {
            return;
        }
        let mut y = area.y;

        // Header row
        if !self.headers.is_empty() {
            let mut spans = Vec::new();
            for (i, h) in self.headers.iter().enumerate() {
                let w = self.column_widths.get(i).copied().unwrap_or(10) as usize;
                spans.push(Span::styled(
                    format!("{:^width$}", h, width = w),
                    Style::default().fg(self.header_color).add_modifier(Modifier::BOLD),
                ));
            }
            let line = Line::from(spans);
            let para = Paragraph::new(line).style(Style::default().bg(self.header_color));
            let header_area = Rect::new(area.x, y, area.width, 1);
            para.render(header_area, buf);
            y += 1;
        }

        // Data rows (with scroll offset)
        let start = self.scroll;
        let max_rows = area.height as usize - 1;

        for row_idx in start..(start + max_rows).min(self.rows.len()) {
            let bg = if Some(row_idx) == self.cursor {
                self.highlight_color
            } else if (row_idx - start).is_multiple_of(2) {
                self.row_colors.0
            } else {
                self.row_colors.1
            };

            let mut spans = Vec::new();
            if let Some(row) = self.rows.get(row_idx) {
                for (i, cell) in row.iter().enumerate() {
                    let w = self.column_widths.get(i).copied().unwrap_or(10) as usize;
                    spans.push(Span::raw(format!("{:width$}", cell, width = w)));
                }
            }
            let line = Line::from(spans);
            let para = Paragraph::new(line).style(Style::default().bg(bg));
            let row_area = Rect::new(area.x, y, area.width, 1);
            para.render(row_area, buf);
            y += 1;
        }
    }
}