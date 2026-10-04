/// Color palette and style helpers for ytui's dark theme.
///
/// One accent color (amber/gold), muted secondary text,
/// pill-badge helpers, and border-style presets.
use ratatui::style::{Color, Modifier, Style};

// ── Base palette ─────────────────────────────────────────────────────────────

/// The main dark background (transparent).
pub const BG: Color = Color::Reset;
/// Surface card background (transparent).
pub const SURFACE: Color = Color::Reset;
/// Slightly elevated surface (selected rows, modal bg). Keep opaque so modals don't bleed.
pub const ELEVATED: Color = Color::Rgb(30, 30, 42);
/// Border / divider color (subtle).
pub const BORDER: Color = Color::Rgb(45, 45, 65);
/// Active border (focused widget).
pub const BORDER_ACTIVE: Color = Color::Rgb(80, 80, 120);

/// Primary accent – amber.
pub const ACCENT: Color = Color::Rgb(255, 185, 55);
/// Dimmed accent (e.g., badge background).
pub const ACCENT_DIM: Color = Color::Rgb(80, 58, 10);

/// Primary text – near-white.
pub const FG: Color = Color::Rgb(230, 230, 245);
/// Muted / secondary text.
pub const MUTED: Color = Color::Rgb(120, 120, 155);
/// Very faint placeholder text.
pub const PLACEHOLDER: Color = Color::Rgb(70, 70, 95);

/// Semantic colors
pub const SUCCESS: Color = Color::Rgb(80, 200, 120);
pub const WARNING: Color = Color::Rgb(255, 180, 50);
pub const ERROR: Color = Color::Rgb(240, 80, 80);
pub const INFO: Color = Color::Rgb(80, 160, 240);

// ── Named styles ─────────────────────────────────────────────────────────────

/// Default background fill.
#[inline]
pub fn bg() -> Style {
    Style::default().bg(BG).fg(FG)
}

/// Surface card.
#[inline]
pub fn surface() -> Style {
    Style::default().bg(SURFACE).fg(FG)
}

/// Title text (bright, bold).
#[inline]
pub fn title() -> Style {
    Style::default().fg(FG).add_modifier(Modifier::BOLD)
}

/// Accent-colored text (amber).
#[inline]
pub fn accent() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// Muted secondary text.
#[inline]
pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

/// Placeholder text inside inputs.
#[inline]
pub fn placeholder() -> Style {
    Style::default().fg(PLACEHOLDER).add_modifier(Modifier::ITALIC)
}

/// Highlighted / selected list row.
#[inline]
pub fn selected() -> Style {
    Style::default().bg(ELEVATED).fg(FG).add_modifier(Modifier::BOLD)
}

/// Focused border.
#[inline]
pub fn border_focused() -> Style {
    Style::default().fg(ACCENT)
}

/// Unfocused border.
#[inline]
pub fn border_unfocused() -> Style {
    Style::default().fg(BORDER)
}

// ── Toast variants ────────────────────────────────────────────────────────────

#[inline]
pub fn toast_info() -> Style {
    Style::default().bg(Color::Rgb(20, 40, 70)).fg(INFO).add_modifier(Modifier::BOLD)
}

#[inline]
pub fn toast_success() -> Style {
    Style::default().bg(Color::Rgb(15, 40, 25)).fg(SUCCESS).add_modifier(Modifier::BOLD)
}

#[inline]
pub fn toast_warning() -> Style {
    Style::default().bg(Color::Rgb(50, 38, 5)).fg(WARNING).add_modifier(Modifier::BOLD)
}

#[inline]
pub fn toast_error() -> Style {
    Style::default().bg(Color::Rgb(50, 15, 15)).fg(ERROR).add_modifier(Modifier::BOLD)
}

// ── Badge helpers ─────────────────────────────────────────────────────────────

/// Pill-badge style (accent background).
#[inline]
pub fn badge_accent() -> Style {
    Style::default().bg(ACCENT_DIM).fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// Pill-badge for info (blue tint).
#[inline]
pub fn badge_info() -> Style {
    Style::default()
        .bg(Color::Rgb(15, 35, 65))
        .fg(INFO)
        .add_modifier(Modifier::BOLD)
}

/// Pill-badge for success (green tint).
#[inline]
pub fn badge_success() -> Style {
    Style::default()
        .bg(Color::Rgb(15, 40, 25))
        .fg(SUCCESS)
        .add_modifier(Modifier::BOLD)
}

// ── Player bar ────────────────────────────────────────────────────────────────

#[inline]
pub fn player_bar() -> Style {
    Style::default().bg(Color::Rgb(18, 18, 26)).fg(FG)
}

#[inline]
pub fn player_progress_filled() -> Style {
    Style::default().fg(ACCENT)
}

#[inline]
pub fn player_progress_empty() -> Style {
    Style::default().fg(BORDER)
}
