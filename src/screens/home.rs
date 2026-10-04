use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{app::AppState, theme, widgets::input::TextInput};

pub fn render(state: &AppState, area: Rect, buf: &mut Buffer) {
    let total_height = 18;
    let y_offset = area.y + area.height.saturating_sub(total_height) / 2;

    let center_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(62),
            Constraint::Min(0),
        ])
        .split(Rect {
            x: area.x,
            y: y_offset,
            width: area.width,
            height: total_height,
        });

    let main_area = center_layout[1];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Logo + version
            Constraint::Length(3), // Search bar
            Constraint::Length(7), // Discover block
        ])
        .split(main_area);

    render_logo(chunks[0], buf);
    render_search(state, chunks[1], buf);
    render_discover(chunks[2], buf);
}

fn render_logo(area: Rect, buf: &mut Buffer) {
    let ascii = r#" ██╗   ██╗████████╗██╗   ██╗██╗
 ╚██╗ ██╔╝╚══██╔══╝██║   ██║██║
  ╚████╔╝    ██║   ██║   ██║██║
   ╚██╔╝     ██║   ██║   ██║██║
    ██║      ██║   ╚██████╔╝██║
    ╚═╝      ╚═╝    ╚═════╝ ╚═╝"#;

    let lines: Vec<Line> = ascii
        .lines()
        .map(|line| Line::from(Span::styled(line, theme::accent())))
        .collect();

    let p = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    p.render(
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 6,
        },
        buf,
    );

    let version = Line::from(Span::styled("v0.1.0", theme::muted()));
    Paragraph::new(version)
        .alignment(ratatui::layout::Alignment::Right)
        .render(
            Rect {
                x: area.x,
                y: area.y + 6,
                width: area.width.saturating_sub(14),
                height: 1,
            },
            buf,
        );
}

fn render_search(state: &AppState, area: Rect, buf: &mut Buffer) {
    let mut input = TextInput::new("  Search YouTube...");
    input.value = state.search_query.clone();
    input.cursor = state.search_cursor;
    input.focused = state.search_focused;

    let border_style = if state.search_focused {
        theme::border_focused()
    } else {
        theme::border_unfocused()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme::surface());

    input.render_with_block(area, buf, block);

    // Overlap the right-side text on the top border
    let hint = " [yt-dlp · Enter] ";
    let hint_len = hint.chars().count() as u16;
    if area.width > hint_len + 2 {
        let hint_area = Rect {
            x: area.x + area.width - hint_len - 1,
            y: area.y,
            width: hint_len,
            height: 1,
        };
        Paragraph::new(Span::styled(hint, theme::accent().add_modifier(Modifier::BOLD))).render(hint_area, buf);
    }

    let title = " ❯ Search videos, channels & playlists... ";
    let title_span = if state.search_focused {
        Span::styled(title, Style::default().fg(theme::FG))
    } else {
        Span::styled(title, theme::muted())
    };
    let title_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: title.chars().count() as u16,
        height: 1,
    };
    Paragraph::new(title_span).render(title_area, buf);
}

fn render_discover(area: Rect, buf: &mut Buffer) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::BOTTOM | Borders::RIGHT)
        .border_style(theme::border_unfocused())
        .style(theme::surface());

    block.render(area, buf);

    let title = " ─ ✦ Discover Categories ";
    Paragraph::new(Span::styled(title, theme::muted())).render(
        Rect {
            x: area.x,
            y: area.y,
            width: title.chars().count() as u16,
            height: 1,
        },
        buf,
    );

    let shortcut = " [ /search ] ";
    let shortcut_len = shortcut.chars().count() as u16;
    Paragraph::new(Span::styled(shortcut, theme::accent().add_modifier(Modifier::BOLD))).render(
        Rect {
            x: area.x + area.width.saturating_sub(shortcut_len) - 1,
            y: area.y,
            width: shortcut_len,
            height: 1,
        },
        buf,
    );

    let categories = vec![
        ("Trending Now", "Popular & Viral Videos"),
        ("Top Rated", "Highly Acclaimed Content"),
        ("Latest Releases", "Recent Uploads"),
        ("Most Watched", "Community Favorites"),
    ];

    for (i, (left, right)) in categories.iter().enumerate() {
        let y = area.y + 1 + i as u16;
        let line_left = Line::from(vec![
            Span::styled("  · ", theme::muted()),
            Span::styled(
                *left,
                Style::default()
                    .fg(theme::FG)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        Paragraph::new(line_left).render(
            Rect {
                x: area.x + 1,
                y,
                width: area.width / 2,
                height: 1,
            },
            buf,
        );

        let line_right = Line::from(Span::styled(*right, theme::muted()));
        Paragraph::new(line_right)
            .alignment(ratatui::layout::Alignment::Right)
            .render(
                Rect {
                    x: area.x + area.width / 2,
                    y,
                    width: area.width / 2 - 3,
                    height: 1,
                },
                buf,
            );
    }
}
