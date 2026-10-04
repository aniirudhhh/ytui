/// In-app log panel (Ctrl+L).
///
/// Displays yt-dlp / mpv activity captured by the custom tracing layer.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

use crate::{app::AppState, theme};

pub fn render(state: &AppState, area: Rect, buf: &mut Buffer) {
    let block = Block::default()
        .title(Span::styled(" Logs ", theme::accent()))
        .borders(Borders::ALL)
        .border_style(theme::border_focused())
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(area);
    block.render(area, buf);

    if state.log_entries.is_empty() {
        Paragraph::new(Span::styled("No log entries yet.", theme::placeholder()))
            .render(inner, buf);
        return;
    }

    // Show most-recent entries, newest at the bottom.
    let skip = state
        .log_entries
        .len()
        .saturating_sub(inner.height as usize);

    let items: Vec<ListItem> = state.log_entries[skip..]
        .iter()
        .map(|entry| {
            let style = log_level_style(entry);
            ListItem::new(Line::from(Span::styled(entry.clone(), style)))
        })
        .collect();

    List::new(items)
        .style(Style::default().bg(theme::SURFACE))
        .render(inner, buf);
}

fn log_level_style(entry: &str) -> Style {
    if entry.contains("ERROR") || entry.contains("error") {
        Style::default().fg(theme::ERROR)
    } else if entry.contains("WARN") || entry.contains("warn") {
        Style::default().fg(theme::WARNING)
    } else {
        Style::default().fg(theme::MUTED)
    }
}
