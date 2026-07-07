use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

/// A centered modal overlay with a dimmed backdrop, title, message, and optional actions.
/// Styled like a "sweet alert" — dark panel, colored border, clear action hints.
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
        // 1. Clear the entire area
        Clear.render(area, buf);

        // 2. Render a dimmed backdrop (solid dark fill to obscure content behind)
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                let cell = buf.cell_mut((x, y)).unwrap();
                cell.set_fg(Color::Rgb(0, 0, 0));
                cell.set_bg(Color::Rgb(0, 0, 0));
                cell.set_char(' ');
            }
        }

        // 3. Calculate centered modal area
        let w = self.width.min(area.width.saturating_sub(4));
        let h = self.height.min(area.height.saturating_sub(4));
        let x = area.x + (area.width - w) / 2;
        let y = area.y + (area.height - h) / 2;
        let modal_area = Rect::new(x, y, w, h);

        // 4. Build content text
        let mut lines: Vec<Line> = Vec::new();

        // Title line (bold, accent colored)
        lines.push(Line::from(vec![Span::styled(
            format!(" {}", self.title),
            Style::default()
                .fg(self.accent_color)
                .add_modifier(Modifier::BOLD),
        )]));

        // Separator
        lines.push(Line::from(Span::styled(
            format!(" {}", "─".repeat((w as usize).saturating_sub(4))),
            Style::default().fg(Color::Rgb(48, 54, 61)),
        )));

        // Message body (left-aligned for readability)
        for line in self.message.lines() {
            lines.push(Line::from(Span::styled(
                format!(" {}", line),
                Style::default().fg(Color::Rgb(201, 209, 217)),
            )));
        }

        // Action hints at bottom
        if self.confirm_label.is_some() || self.cancel_label.is_some() {
            lines.push(Line::from(""));
            let mut hint_spans: Vec<Span> = Vec::new();
            if let Some(c) = self.confirm_label {
                hint_spans.push(Span::styled(
                    format!(" [{}] ", c),
                    Style::default()
                        .fg(Color::Rgb(63, 185, 80))
                        .add_modifier(Modifier::BOLD),
                ));
            }
            if let Some(c) = self.cancel_label {
                hint_spans.push(Span::styled(
                    format!(" [{}] ", c),
                    Style::default()
                        .fg(Color::Rgb(139, 148, 158))
                        .add_modifier(Modifier::BOLD),
                ));
            }
            lines.push(Line::from(hint_spans));
        }

        // 5. Modal panel block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.accent_color))
            .style(Style::default().bg(Color::Rgb(22, 27, 34)));

        let para = Paragraph::new(Text::from(lines))
            .block(block)
            .alignment(Alignment::Left);

        para.render(modal_area, buf);
    }
}
