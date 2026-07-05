use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
    buffer::Buffer,
};

/// A right-click context menu.
pub struct ContextMenu<'a> {
    pub items: &'a [&'a str],
    pub cursor: usize,
    pub x: u16,
    pub y: u16,
}

impl Widget for ContextMenu<'_> {
    fn render(self, area: Rect, _buf: &mut Buffer) {
        // TODO: full context menu rendering with click zones
        // For now, this is a stub — the mouse handler will be wired in A3.
    }
}

/// Render a context menu inline (for when we need to show it outside a Widget context).
pub fn render_context_menu(items: &[&str], cursor: usize, x: u16, y: u16, area: Rect) -> impl Widget {
    // Returns a block with items, cursor highlighted
    let mut lines = Vec::new();
    for (i, item) in items.iter().enumerate() {
        if i == cursor {
            lines.push(Line::from(Span::styled(
                format!(" ▶ {}", item),
                Style::default().fg(Color::Rgb(88, 166, 255)).bg(Color::Rgb(48, 54, 61)),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("   {}", item),
                Style::default().fg(Color::Rgb(201, 209, 217)),
            )));
        }
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(48, 54, 61)))
        .style(Style::default().bg(Color::Rgb(13, 17, 23)));
    Paragraph::new(Text::from(lines.clone()))
        .block(block)
        .style(Style::default())
}