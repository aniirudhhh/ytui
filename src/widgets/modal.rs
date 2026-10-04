/// Dimmed-backdrop modal container.
///
/// Wraps any content and renders a semi-transparent overlay behind it.
/// The caller renders child widgets into the `inner()` area.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Clear, Widget},
};

use crate::theme;

/// A full-screen-overlay modal.
pub struct Modal<'a> {
    pub title: &'a str,
    /// Width as a fraction of the parent area (0.0–1.0).
    pub width_frac: f64,
    /// Height as a fraction of the parent area (0.0–1.0).
    pub height_frac: f64,
}

impl<'a> Modal<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title, width_frac: 0.6, height_frac: 0.6 }
    }

    pub fn width(mut self, frac: f64) -> Self { self.width_frac = frac; self }
    pub fn height(mut self, frac: f64) -> Self { self.height_frac = frac; self }

    /// Compute the centred inner [`Rect`] for this modal.
    pub fn inner_area(&self, area: Rect) -> Rect {
        let w = (area.width as f64 * self.width_frac) as u16;
        let h = (area.height as f64 * self.height_frac) as u16;
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y + area.height.saturating_sub(h) / 2;
        Rect { x, y, width: w, height: h }
    }

    /// Render the overlay dimming + modal border and return the inner area
    /// where the caller should draw content.
    pub fn render_and_inner(&self, area: Rect, buf: &mut Buffer) -> Rect {
        // Dim the whole area first.
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_style(
                    Style::default()
                        .bg(theme::BG)
                        .add_modifier(ratatui::style::Modifier::DIM),
                );
            }
        }

        let modal_area = self.inner_area(area);
        Clear.render(modal_area, buf);

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .title_style(theme::accent())
            .borders(Borders::ALL)
            .border_style(theme::border_focused())
            .style(Style::default().bg(theme::SURFACE));
        let inner = block.inner(modal_area);
        block.render(modal_area, buf);
        inner
    }
}
