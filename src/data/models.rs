/// Core data models shared across the whole application.
///
/// Kept in one place so that the UI, data layer, and player
/// can all import them without circular dependencies.
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Full dynamic image data for ratatui-image.
pub struct ThumbnailData {
    pub protocol: std::cell::RefCell<ratatui_image::protocol::StatefulProtocol>,
}
impl std::fmt::Debug for ThumbnailData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThumbnailData").finish()
    }
}

// ── Video ─────────────────────────────────────────────────────────────────────

/// A YouTube video entry (from search results or the subscription feed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    /// YouTube video ID (e.g. `dQw4w9WgXcQ`).
    pub id: String,
    /// Display title.
    pub title: String,
    /// Channel name.
    pub channel: String,
    /// Channel ID.
    pub channel_id: String,
    /// Formatted duration string (e.g. `"3:32"`).
    pub duration: Option<String>,
    /// Duration in seconds.
    pub duration_secs: Option<f64>,
    /// View count.
    pub view_count: Option<u64>,
    /// Upload / publish date string (ISO-8601 or similar).
    pub upload_date: Option<String>,
    /// Best thumbnail URL.
    pub thumbnail: Option<String>,
    /// Short description / snippet.
    pub description: Option<String>,
    /// Where this entry came from.
    pub source: VideoSource,
}

impl Video {
    /// Canonical watch URL.
    pub fn url(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }

    /// Human-readable view count (e.g. `"1.2M"`).
    pub fn views_display(&self) -> String {
        match self.view_count {
            None => String::new(),
            Some(v) if v >= 1_000_000 => format!("{:.1}M views", v as f64 / 1_000_000.0),
            Some(v) if v >= 1_000 => format!("{:.1}K views", v as f64 / 1_000.0),
            Some(v) => format!("{v} views"),
        }
    }
}

/// Where a [`Video`] originated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoSource {
    Search,
    Feed,
    History,
    Queue,
}

// ── Format / quality ──────────────────────────────────────────────────────────

/// A single audio/video format entry from `yt-dlp -F`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Format {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub fps: Option<f64>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<u64>,
    pub tbr: Option<f64>,
    pub abr: Option<f64>,
    pub vbr: Option<f64>,
    pub dynamic_range: Option<String>,
    pub format_note: Option<String>,
}

impl Format {
    /// True if this format has a video stream.
    pub fn has_video(&self) -> bool {
        self.vcodec.as_deref().is_some_and(|c| c != "none")
    }

    /// True if this format has an audio stream.
    pub fn has_audio(&self) -> bool {
        self.acodec.as_deref().is_some_and(|c| c != "none")
    }

    /// Short display label for the quality picker modal.
    pub fn label(&self) -> String {
        let res = self.resolution.as_deref().unwrap_or("audio");
        let fps_str = self
            .fps
            .filter(|&f| f >= 50.0)
            .map(|f| format!("@{:.0}fps", f))
            .unwrap_or_default();
        let size_str = self
            .filesize
            .map(|b| {
                if b >= 1_000_000 {
                    format!(" {:.1}MB", b as f64 / 1_000_000.0)
                } else {
                    format!(" {:.0}KB", b as f64 / 1_000.0)
                }
            })
            .unwrap_or_default();
        format!(
            "{}{} [{}]{}",
            res,
            fps_str,
            self.ext,
            size_str
        )
    }

    /// Badge strings to show alongside this format (HDR, 60fps, etc.).
    pub fn badges(&self) -> Vec<&'static str> {
        let mut b = Vec::new();
        if self.dynamic_range.as_deref() == Some("HDR") { b.push("HDR"); }
        if self.fps.is_some_and(|f| f >= 50.0) { b.push("60fps"); }
        b
    }
}

// ── Channel ───────────────────────────────────────────────────────────────────

/// A subscribed channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub thumbnail: Option<String>,
    pub subscribed_at: Option<SystemTime>,
}

// ── History entry ─────────────────────────────────────────────────────────────

/// One entry in the watch history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub video: Video,
    /// Last played position in seconds.
    pub position_secs: f64,
    /// Last watched timestamp.
    pub last_watched: SystemTime,
    /// The format ID chosen last time.
    pub chosen_format_id: Option<String>,
}

impl HistoryEntry {
    /// Progress fraction (0.0 – 1.0).
    pub fn progress(&self) -> f64 {
        match self.video.duration_secs {
            Some(d) if d > 0.0 => (self.position_secs / d).min(1.0),
            _ => 0.0,
        }
    }
}
