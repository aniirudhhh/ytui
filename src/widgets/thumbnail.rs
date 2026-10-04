/// Thumbnail placeholder widget (full implementation in M8).
///
/// In M1 this renders a coloured block with the first letter of the video
/// title as a stand-in, sized to match a typical thumbnail aspect ratio.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Span,
    widgets::{Block, Borders, Widget, StatefulWidget},
};

use crate::theme;

/// A placeholder thumbnail.
pub struct Thumbnail<'a> {
    pub initials: &'a str,
    pub data: Option<std::rc::Rc<crate::data::models::ThumbnailData>>,
}

impl<'a> Thumbnail<'a> {
    pub fn new(initials: &'a str, data: Option<std::rc::Rc<crate::data::models::ThumbnailData>>) -> Self {
        Self { initials, data }
    }
}

impl Widget for Thumbnail<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(data) = self.data {
            // Render actual image via ratatui-image Kitty protocol
            ratatui_image::StatefulImage::new()
                .render(area, buf, &mut *data.protocol.borrow_mut());
        } else {
            // Render placeholder
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border_unfocused())
                .render(area, buf);

            if area.width < 3 || area.height < 2 { return; }

            let text = self.initials.chars().take(2).collect::<String>();
            let x = area.x + (area.width.saturating_sub(text.len() as u16)) / 2;
            let y = area.y + area.height / 2;
            
            let span = Span::styled(text, theme::muted());
            buf.set_string(x, y, span.content.as_ref(), span.style);
        }
    }
}
