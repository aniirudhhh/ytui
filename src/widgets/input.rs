/// Inline text-input widget with cursor tracking and basic editing.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::theme;

/// A single-line text input widget.
///
/// Cursor position is tracked as a byte offset in [`value`].
#[derive(Debug, Clone, Default)]
pub struct TextInput {
    /// Current text content.
    pub value: String,
    /// Cursor byte offset (always on a char boundary).
    pub cursor: usize,
    /// Placeholder shown when the input is empty.
    pub placeholder: String,
    /// True when the widget has keyboard focus.
    pub focused: bool,
    /// Horizontal scroll offset (for values wider than the widget).
    scroll: usize,
}

impl TextInput {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..Default::default()
        }
    }

    /// Insert a character at the cursor position.
    pub fn insert(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete the character to the left of the cursor (backspace).
    pub fn backspace(&mut self) {
        if self.cursor == 0 { return; }
        // Find previous char boundary.
        let mut prev = self.cursor - 1;
        while !self.value.is_char_boundary(prev) { prev -= 1; }
        self.value.remove(prev);
        self.cursor = prev;
    }

    /// Delete the character under the cursor (delete key).
    pub fn delete_forward(&mut self) {
        if self.cursor >= self.value.len() { return; }
        self.value.remove(self.cursor);
    }

    /// Move cursor left one character.
    pub fn move_left(&mut self) {
        if self.cursor == 0 { return; }
        let mut prev = self.cursor - 1;
        while !self.value.is_char_boundary(prev) { prev -= 1; }
        self.cursor = prev;
    }

    /// Move cursor right one character.
    pub fn move_right(&mut self) {
        if self.cursor >= self.value.len() { return; }
        let mut next = self.cursor + 1;
        while !self.value.is_char_boundary(next) { next += 1; }
        self.cursor = next;
    }

    /// Jump to start of line.
    pub fn move_home(&mut self) { self.cursor = 0; }

    /// Jump to end of line.
    pub fn move_end(&mut self) { self.cursor = self.value.len(); }

    /// Delete from cursor to end of line (Ctrl-K).
    pub fn kill_to_end(&mut self) {
        self.value.truncate(self.cursor);
    }

    /// Clear the entire input.
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
        self.scroll = 0;
    }

    /// Render the widget into `area`.
    ///
    /// The caller is responsible for passing the right [`Rect`]; this widget
    /// does **not** request its own layout space.
    pub fn render_with_block(&self, area: Rect, buf: &mut Buffer, block: Block) {
        let inner = block.inner(area);
        block.render(area, buf);
        self.render_inner(inner, buf);
    }

    /// Render into an already-computed inner area (no block drawn).
    pub fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 { return; }

        let width = area.width as usize;

        if self.value.is_empty() {
            // Show placeholder.
            let ph: String = self.placeholder.chars().take(width).collect();
            let span = Span::styled(ph, theme::placeholder());
            Paragraph::new(Line::from(span)).render(area, buf);
            return;
        }

        // Compute scroll so the cursor stays visible.
        let cursor_col = self.value[..self.cursor].chars().count();
        let scroll = if cursor_col >= width {
            cursor_col - width + 1
        } else {
            0
        };

        // Slice characters starting at `scroll`.
        let display: String = self.value.chars().skip(scroll).take(width).collect();
        let cursor_in_view = cursor_col.saturating_sub(scroll);

        let before: String = display.chars().take(cursor_in_view).collect();
        let cursor_char: String = display.chars().nth(cursor_in_view).map(String::from).unwrap_or_else(|| " ".into());
        let after: String = display.chars().skip(cursor_in_view + 1).collect();

        let fg = if self.focused { theme::FG } else { theme::MUTED };
        let cursor_style = if self.focused {
            Style::default().bg(theme::ACCENT).fg(theme::BG)
        } else {
            Style::default().fg(fg)
        };

        let spans = vec![
            Span::styled(before, Style::default().fg(fg)),
            Span::styled(cursor_char, cursor_style),
            Span::styled(after, Style::default().fg(fg)),
        ];
        Paragraph::new(Line::from(spans)).render(area, buf);
    }
}

impl Widget for &TextInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            theme::border_focused()
        } else {
            theme::border_unfocused()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(theme::surface());
        self.render_with_block(area, buf, block);
    }
}
