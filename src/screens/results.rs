use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::{app::AppState, theme, widgets::video_card::VideoCard};

pub fn render(state: &AppState, area: Rect, buf: &mut Buffer) {
    Block::default()
        .style(Style::default().bg(theme::BG))
        .render(area, buf);

    let query = &state.search_query;
    let title = format!(" Results for '{}' ", query);
    
    // Header
    let header_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: 1,
    };
    Paragraph::new(Span::styled(title, theme::accent())).render(header_area, buf);

    let content_area = Rect {
        x: area.x + 2,
        y: area.y + 3,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(4),
    };

    if state.search_results.is_empty() {
        Paragraph::new(Span::styled("Searching... (or no results)", theme::muted())).render(content_area, buf);
        return;
    }

    let item_height = VideoCard::HEIGHT;
    let visible_rows = (content_area.height / item_height) as usize;
    if visible_rows == 0 {
        return;
    }
    let columns = 2;
    let visible_items = visible_rows * columns;

    let cursor = state.results_cursor;
    
    // Calculate scrolling window
    let cursor_row = cursor / columns;
    
    let mut start_row = 0;
    if cursor_row >= visible_rows / 2 {
        start_row = cursor_row.saturating_sub(visible_rows / 2);
    }
    
    let total_rows = (state.search_results.len() + columns - 1) / columns;
    if start_row + visible_rows > total_rows {
        start_row = total_rows.saturating_sub(visible_rows);
    }
    
    let start_idx = start_row * columns;
    let col_width = content_area.width / columns as u16;

    for (i, video) in state.search_results.iter().enumerate().skip(start_idx).take(visible_items) {
        let is_selected = i == cursor;
        let thumbnail_data = state.thumbnails.get(&video.id).cloned();
        let card = VideoCard::new(video, is_selected, i, thumbnail_data);
        
        let row = (i - start_idx) / columns;
        let col = (i - start_idx) % columns;
        
        let card_area = Rect {
            x: content_area.x + (col as u16 * col_width),
            y: content_area.y + (row as u16 * item_height),
            width: col_width.saturating_sub(2), // 2 cols padding
            height: item_height,
        };
        card.render(card_area, buf);
    }
}
