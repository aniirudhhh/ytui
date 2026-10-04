/// Details screen — implemented in M6.
use ratatui::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::{Paragraph, Widget}, style::Style};
use crate::{app::AppState, theme};

pub fn render(_state: &AppState, area: Rect, buf: &mut Buffer) {
    Paragraph::new(vec![
        Line::from(Span::styled(" Video Details ", theme::accent())),
        Line::from(""),
        Line::from(Span::styled("Details screen coming in M6.", theme::muted())),
    ])
    .style(Style::default().bg(theme::BG))
    .render(area, buf);
}
