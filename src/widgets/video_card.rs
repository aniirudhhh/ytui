/// Video card widget — renders a single video entry in a list.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::{data::models::Video, theme, widgets::thumbnail::Thumbnail};

/// Renders a video as a list row.
///
/// Expects a 3-row-tall area:
///   Row 0 – title (+ duration badge right-aligned)
///   Row 1 – channel · views · date
///   Row 2 – empty separator
pub struct VideoCard<'a> {
    pub video: &'a Video,
    pub selected: bool,
    pub index: usize,
    pub thumbnail_data: Option<std::rc::Rc<crate::data::models::ThumbnailData>>,
}

impl<'a> VideoCard<'a> {
    pub fn new(
        video: &'a Video,
        selected: bool,
        index: usize,
        thumbnail_data: Option<std::rc::Rc<crate::data::models::ThumbnailData>>,
    ) -> Self {
        Self {
            video,
            selected,
            index,
            thumbnail_data,
        }
    }

    /// Required height per card.
    pub const HEIGHT: u16 = 8;
}

impl Widget for VideoCard<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 { return; }

        let v = self.video;
        let base_style = Style::default();

        // Fill background to clear any artifacts, but keep it transparent.
        for row in area.top()..area.bottom() {
            for col in area.left()..area.right() {
                buf[(col, row)].set_style(base_style);
            }
        }

        // ── Selection Indicator & Thumbnail ──────────────────────────────────
        if self.selected {
            buf.set_string(area.x, area.y + 3, "▶", theme::accent());
        }

        let thumb_area = Rect {
            x: area.x + 2,
            y: area.y + 1, // centered vertically in the 8 rows
            width: 28,
            height: 6.min(area.height.saturating_sub(2)),
        };
        let thumb_initials: String = v.title.chars().take(2).collect();
        Thumbnail::new(&thumb_initials, self.thumbnail_data).render(thumb_area, buf);

        let text_x = area.x + 32;
        let text_width = area.width.saturating_sub(34);

        // ── Row 0: title + duration ──────────────────────────────────────────
        if area.height >= 1 {
            let title_area = Rect { x: text_x, y: area.y + 2, width: text_width, height: 1 };

            let dur = v.duration.as_deref().unwrap_or("");
            let dur_len = dur.len() as u16 + 4;
            let title_max = title_area.width.saturating_sub(dur_len + 2) as usize;
            let title: String = v.title.chars().take(title_max).collect();
            let ellipsis = if v.title.chars().count() > title_max { "…" } else { "" };
            let pad = " ".repeat(
                (title_area.width as usize)
                    .saturating_sub(title.chars().count() + ellipsis.len() + dur_len as usize + 1)
            );

            let title_style = if self.selected { theme::title() } else { Style::default().fg(theme::FG) };
            let dur_style = if dur.is_empty() { Style::default() } else { theme::badge_accent() };

            let mut spans = vec![
                Span::styled(title, title_style),
                Span::styled(ellipsis, theme::muted()),
                Span::styled(pad, Style::default()),
            ];
            if !dur.is_empty() {
                spans.push(Span::styled(format!(" {dur} "), dur_style));
            }
            Paragraph::new(Line::from(spans)).render(title_area, buf);
        }

        // ── Row 1: metadata ──────────────────────────────────────────────────
        if area.height >= 2 {
            let meta_area = Rect { x: text_x, y: area.y + 4, width: text_width, height: 1 };

            let channel: String = v.channel.chars().take(28).collect();
            let views = v.views_display();
            let date = v.upload_date.as_deref().unwrap_or("");

            let mut spans = vec![
                Span::styled(channel, theme::accent()),
            ];
            if !views.is_empty() {
                spans.push(Span::styled("  ·  ", theme::muted()));
                spans.push(Span::styled(views, theme::muted()));
            }
            if !date.is_empty() {
                spans.push(Span::styled("  ·  ", theme::muted()));
                spans.push(Span::styled(date, theme::muted()));
            }
            Paragraph::new(Line::from(spans)).render(meta_area, buf);
        }
    }
}
