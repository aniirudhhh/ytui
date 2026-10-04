/// Toast notification widget.
///
/// Toasts are rendered as a stack in the top-right corner of the screen.
/// Each toast has a severity level that determines its colour and an
/// associated log entry that gets forwarded to the in-app log panel.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Widget},
};
use std::time::{Duration, Instant};

use crate::theme;

// ── Severity ──────────────────────────────────────────────────────────────────

/// Severity level of a toast message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Success,
    Warning,
    Error,
}

impl Severity {
    fn icon(self) -> &'static str {
        match self {
            Severity::Info    => "  ",
            Severity::Success => "  ",
            Severity::Warning => "  ",
            Severity::Error   => "  ",
        }
    }

    fn style(self) -> ratatui::style::Style {
        match self {
            Severity::Info    => theme::toast_info(),
            Severity::Success => theme::toast_success(),
            Severity::Warning => theme::toast_warning(),
            Severity::Error   => theme::toast_error(),
        }
    }
}

// ── Toast ─────────────────────────────────────────────────────────────────────

/// A single toast notification.
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub severity: Severity,
    born: Instant,
    ttl: Duration,
}

impl Toast {
    /// Create a new toast with a given severity and default TTL (4 s).
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity,
            born: Instant::now(),
            ttl: Duration::from_secs(4),
        }
    }

    /// Create a toast with a custom TTL.
    pub fn with_ttl(mut self, secs: u64) -> Self {
        self.ttl = Duration::from_secs(secs);
        self
    }

    /// `true` if the toast has expired and should be removed.
    pub fn expired(&self) -> bool {
        self.born.elapsed() >= self.ttl
    }

    /// Progress fraction (0.0 fresh → 1.0 expired).
    pub fn age_frac(&self) -> f64 {
        (self.born.elapsed().as_secs_f64() / self.ttl.as_secs_f64()).min(1.0)
    }
}

// ── ToastStack ────────────────────────────────────────────────────────────────

/// Manages the list of active toasts and renders them.
#[derive(Debug, Default)]
pub struct ToastStack {
    toasts: Vec<Toast>,
}

impl ToastStack {
    /// Add a new toast.
    pub fn push(&mut self, toast: Toast) {
        // Cap the stack at 5 visible toasts.
        if self.toasts.len() >= 5 {
            self.toasts.remove(0);
        }
        self.toasts.push(toast);
    }

    /// Convenience helpers.
    pub fn info(&mut self, msg: impl Into<String>) {
        self.push(Toast::new(Severity::Info, msg));
    }
    pub fn success(&mut self, msg: impl Into<String>) {
        self.push(Toast::new(Severity::Success, msg));
    }
    pub fn warn(&mut self, msg: impl Into<String>) {
        self.push(Toast::new(Severity::Warning, msg));
    }
    pub fn error(&mut self, msg: impl Into<String>) {
        self.push(Toast::new(Severity::Error, msg));
    }

    /// Remove expired toasts. Call once per render tick.
    pub fn prune(&mut self) {
        self.toasts.retain(|t| !t.expired());
    }

    /// `true` if there are any active toasts.
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// Render all active toasts into `area`.
    ///
    /// Toasts stack upward from the top-right corner; newest is at the bottom
    /// of the stack (closest to top-right).
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        const MAX_WIDTH: u16 = 48;
        const PADDING: u16 = 1; // right-margin from the edge

        let mut y = area.top() + 1;

        for toast in &self.toasts {
            let msg = format!("{} {}", toast.severity.icon(), toast.message);
            // Clamp width.
            let w = (msg.len() as u16 + 4).min(MAX_WIDTH).min(area.width.saturating_sub(2));
            let x = area.right().saturating_sub(w + PADDING);

            let toast_area = Rect {
                x,
                y,
                width: w,
                height: 1,
            };

            if toast_area.bottom() >= area.bottom() {
                break;
            }

            // Dim the text as the toast ages.
            let age = toast.age_frac();
            let base_style = toast.severity.style();
            let style = if age > 0.75 {
                base_style.add_modifier(ratatui::style::Modifier::DIM)
            } else {
                base_style
            };

            Clear.render(toast_area, buf);
            let content = msg.chars().take(w as usize - 2).collect::<String>();
            Paragraph::new(Line::from(vec![Span::styled(format!(" {content} "), style)]))
                .block(Block::default())
                .render(toast_area, buf);

            y += 2; // spacing between toasts
        }
    }
}
