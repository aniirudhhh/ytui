/// Pill-badge widget helper.
///
/// Renders a ` label ` span styled as a coloured pill.
use ratatui::{
    style::Style,
    text::Span,
};

use crate::theme;

/// Preset badge categories.
#[derive(Debug, Clone, Copy)]
pub enum BadgeKind {
    Accent,
    Info,
    Success,
    Muted,
}

/// Build a single pill-badge `Span` ready for use in a `Line`.
pub fn badge(label: &str, kind: BadgeKind) -> Span<'static> {
    let text = format!(" {} ", label);
    let style = match kind {
        BadgeKind::Accent  => theme::badge_accent(),
        BadgeKind::Info    => theme::badge_info(),
        BadgeKind::Success => theme::badge_success(),
        BadgeKind::Muted   => Style::default().fg(theme::MUTED),
    };
    Span::styled(text, style)
}

/// Separator span between badges.
pub fn badge_sep() -> Span<'static> {
    Span::styled(" ", Style::default())
}
