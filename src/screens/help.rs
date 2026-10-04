/// Help overlay — shows all keybinds in a modal.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, Widget},
};

use crate::{app::AppState, theme, widgets::modal::Modal};

pub fn render(_state: &AppState, area: Rect, buf: &mut Buffer) {
    let modal = Modal::new("Keybinds").width(0.55).height(0.75);
    let inner = modal.render_and_inner(area, buf);

    let entries: &[(&str, &str)] = &[
        ("/",       "Open search"),
        ("Enter",   "Confirm / open / play"),
        ("Esc",     "Go back / close"),
        ("q",       "Quit"),
        ("Space",   "Pause / resume"),
        ("h",       "Seek back 5 s"),
        ("l",       "Seek forward 5 s"),
        ("j / ↓",   "Move down"),
        ("k / ↑",   "Move up"),
        ("J",       "Volume down"),
        ("K",       "Volume up"),
        ("a",       "Add to queue"),
        ("y",       "Copy video URL"),
        ("Q",       "Open queue"),
        ("Ctrl+L",  "Toggle log panel"),
        ("?",       "Toggle this help"),
    ];

    let items: Vec<ListItem> = entries
        .iter()
        .map(|(key, desc)| {
            let key_span = Span::styled(format!("  {:10}", key), theme::accent());
            let desc_span = Span::styled(format!(" {}", desc), Style::default().fg(theme::FG));
            ListItem::new(Line::from(vec![key_span, desc_span]))
        })
        .collect();

    List::new(items)
        .style(Style::default().bg(theme::SURFACE))
        .render(inner, buf);
}
