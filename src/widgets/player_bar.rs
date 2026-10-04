/// Persistent player bar rendered at the bottom of every screen.
///
/// Driven by [`PlayerState`] which is updated via mpv IPC observers (M3).
/// In M1 this renders a placeholder / idle state.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::bar,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::theme;

// ── PlayerState ───────────────────────────────────────────────────────────────

/// Live playback state, updated by mpv IPC events.
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub title: Option<String>,
    pub channel: Option<String>,
    pub position: f64,
    pub duration: f64,
    pub paused: bool,
    pub volume: f64,
    pub idle: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            title: None,
            channel: None,
            position: 0.0,
            duration: 0.0,
            paused: false,
            volume: 100.0,
            idle: true,
        }
    }
}

// ── PlayerBar widget ──────────────────────────────────────────────────────────

/// Renders the player status bar.
pub struct PlayerBar<'a> {
    pub state: &'a PlayerState,
}

impl<'a> PlayerBar<'a> {
    pub fn new(state: &'a PlayerState) -> Self {
        Self { state }
    }
}

impl Widget for PlayerBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = self.state;

        // Background fill.
        Block::default()
            .style(theme::player_bar())
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme::BORDER))
            .render(area, buf);

        if area.height < 2 {
            return;
        }

        // ── Row 1: track info ────────────────────────────────────────────────
        let info_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: 1,
        };

        if s.idle {
            let idle_span = Span::styled("  No media playing  ", theme::muted());
            Paragraph::new(Line::from(idle_span)).render(info_area, buf);
        } else {
            let play_icon = if s.paused { "⏸ " } else { "▶ " };
            let title = s.title.as_deref().unwrap_or("Unknown");
            let channel = s.channel.as_deref().unwrap_or("");

            let pos_str = fmt_secs(s.position);
            let dur_str = fmt_secs(s.duration);
            let vol_str = format!(" 🔊{:.0}%", s.volume);

            // Build line: [icon] title · channel      pos/dur  vol
            let right_part = format!("{} / {}  {}", pos_str, dur_str, vol_str);
            let right_len = right_part.chars().count() as u16;
            let left_max = info_area.width.saturating_sub(right_len + 2) as usize;
            let left = format!("{}{} · {}", play_icon, title, channel);
            let left: String = left.chars().take(left_max).collect();
            let pad = " ".repeat(
                (info_area.width as usize)
                    .saturating_sub(left.chars().count())
                    .saturating_sub(right_len as usize),
            );

            let spans = vec![
                Span::styled(left, theme::title()),
                Span::styled(pad, theme::muted()),
                Span::styled(right_part, theme::muted()),
            ];
            Paragraph::new(Line::from(spans)).render(info_area, buf);
        }

        // ── Row 2: progress bar ──────────────────────────────────────────────
        if area.height >= 4 && !s.idle {
            let bar_area = Rect {
                x: area.x + 1,
                y: area.y + 2,
                width: area.width.saturating_sub(2),
                height: 1,
            };
            render_progress(bar_area, buf, s.position, s.duration);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_secs(secs: f64) -> String {
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn render_progress(area: Rect, buf: &mut Buffer, pos: f64, dur: f64) {
    let frac = if dur > 0.0 { (pos / dur).clamp(0.0, 1.0) } else { 0.0 };
    let filled = (frac * area.width as f64).round() as u16;

    for col in 0..area.width {
        let x = area.x + col;
        let style = if col < filled {
            theme::player_progress_filled()
        } else {
            theme::player_progress_empty()
        };
        buf[(x, area.y)]
            .set_symbol(bar::FULL)
            .set_style(style);
    }
}
