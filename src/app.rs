/// Application state, action / event enums, update function, and event loop.
///
/// Follows an Elm-style unidirectional data flow:
///
/// ```text
/// Input / background tasks
///     │ Action / Event
///     ▼
/// update(&mut AppState, Action)  ──► spawn background tasks if needed
///     │
///     ▼
/// render(frame, &AppState)        ──► TUI output (pure, no I/O)
/// ```
use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::{Constraint, Direction, Layout}, widgets::Widget, Frame};
use tokio::sync::mpsc;

use crate::{
    config::Config,
    screens,
    theme,
    widgets::{
        player_bar::{PlayerBar, PlayerState},
        toast::{Severity, Toast, ToastStack},
    },
};

// ── Screen enum ───────────────────────────────────────────────────────────────

/// Which full-screen view is currently rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Home,
    Results,
    Details,
    Logs,
    Help,
}

// ── Action ────────────────────────────────────────────────────────────────────

/// Every mutation of [`AppState`] is described by an `Action`.
///
/// Actions are created from key events (in the event loop) or from background
/// task results (sent back via `mpsc`).
#[derive(Debug)]
pub enum Action {
    // ── Navigation ────────────────────────────────────────────────────────────
    NavigateTo(Screen),
    Back,
    Quit,
    MoveLeft,
    MoveRight,

    // ── Search ────────────────────────────────────────────────────────────────
    FocusSearch,
    SearchInput(char),
    SearchBackspace,
    SearchDeleteForward,
    SearchMoveCursorLeft,
    SearchMoveCursorRight,
    SearchMoveHome,
    SearchMoveEnd,
    SearchKillToEnd,
    SearchClear,
    /// Submit the current query.
    Search,
    /// Results came back (M2).
    #[allow(dead_code)]
    SearchResults(Vec<crate::data::models::Video>),
    /// A custom thumbnail was successfully loaded.
    ThumbnailLoaded(String, image::DynamicImage),

    // ── Player ────────────────────────────────────────────────────────────────
    PlayVideo(String),
    CloseFormatInput,
    StartPlayback,
    FormatsLoaded(Vec<String>),
    FormatScrollUp,
    FormatScrollDown,
    FormatInputChar(char),
    FormatInputBackspace,
    FormatInputMoveCursorLeft,
    FormatInputMoveCursorRight,
    TogglePause,
    SeekBack,
    SeekForward,
    VolumeUp,
    VolumeDown,
    /// mpv state update (M3).
    #[allow(dead_code)]
    PlayerUpdate(PlayerState),

    // ── Queue / misc ──────────────────────────────────────────────────────────
    AddToQueue,
    CopyUrl,
    OpenQueue,

    // ── Logs ──────────────────────────────────────────────────────────────────
    ToggleLogs,
    /// Toggle the help overlay.
    ToggleHelp,
    AppendLog(String),

    // ── Toast ─────────────────────────────────────────────────────────────────
    ShowToast(Severity, String),

    /// Move the list selection up.
    MoveUp,
    /// Move the list selection down.
    MoveDown,

    // ── UI tick ───────────────────────────────────────────────────────────────
    Tick,
}


// ── AppState ──────────────────────────────────────────────────────────────────

/// Full UI state. The render function is a pure function of this struct.
pub struct AppState {
    pub config: Config,
    pub screen: Screen,

    // Search bar
    pub search_query: String,
    pub search_cursor: usize,
    pub search_focused: bool,

    // Player
    pub player: PlayerState,

    // Overlays / modals
    pub logs_open: bool,
    pub help_open: bool,

    // Toasts
    pub toasts: ToastStack,

    // In-app log buffer (fed by the custom tracing layer).
    pub log_entries: Vec<String>,

    // Shutdown flag.
    pub should_quit: bool,

    pub search_results: Vec<crate::data::models::Video>,
    pub results_cursor: usize,
    pub thumbnails: HashMap<String, std::rc::Rc<crate::data::models::ThumbnailData>>,
    pub picker: ratatui_image::picker::Picker,
    
    // Format Input Modal
    pub format_input_open: Option<String>, // contains the video ID
    pub format_input: String,
    pub format_cursor: usize,
    pub format_list: Option<Vec<String>>,
    pub format_scroll: usize,

    // Sender half for the action channel (so update() can spawn tasks that
    // send actions back).
    pub action_tx: mpsc::UnboundedSender<Action>,
}

impl AppState {
    pub fn new(config: Config, action_tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            config,
            screen: Screen::Home,
            search_query: String::new(),
            search_cursor: 0,
            search_focused: false,
            player: PlayerState::default(),
            logs_open: false,
            help_open: false,
            toasts: ToastStack::default(),
            log_entries: Vec::new(),
            should_quit: false,
            search_results: Vec::new(),
            results_cursor: 0,
            thumbnails: HashMap::new(),
            picker: ratatui_image::picker::Picker::from_query_stdio().unwrap_or_else(|_| ratatui_image::picker::Picker::halfblocks()),
            format_input_open: None,
            format_input: String::from("best"),
            format_cursor: 4,
            format_list: None,
            format_scroll: 0,
            action_tx,
        }
    }
}

// ── update() ─────────────────────────────────────────────────────────────────

/// Mutate `state` in response to `action`.
///
/// This is the single place where state transitions happen.
/// Side-effecting tasks (network, subprocess) are spawned here but their
/// results come back as further `Action`s, never touching state directly.
pub fn update(state: &mut AppState, action: Action) {
    match action {
        // ── Navigation ────────────────────────────────────────────────────────
        Action::NavigateTo(screen) => {
            state.logs_open = false;
            state.help_open = false;
            state.screen = screen;
        }
        Action::Back => {
            if state.help_open {
                state.help_open = false;
            } else if state.logs_open {
                state.logs_open = false;
            } else if state.search_focused {
                state.search_focused = false;
            } else {
                match state.screen {
                    Screen::Home => {}
                    _ => state.screen = Screen::Home,
                }
            }
        }
        Action::Quit => {
            state.should_quit = true;
        }

        // ── Search ────────────────────────────────────────────────────────────
        Action::FocusSearch => {
            state.search_focused = true;
            state.screen = Screen::Home;
        }
        Action::SearchInput(c) => {
            // Insert at cursor.
            state.search_query.insert(state.search_cursor, c);
            state.search_cursor += c.len_utf8();
        }
        Action::SearchBackspace => {
            if state.search_cursor > 0 {
                let mut prev = state.search_cursor - 1;
                while !state.search_query.is_char_boundary(prev) { prev -= 1; }
                state.search_query.remove(prev);
                state.search_cursor = prev;
            }
        }
        Action::SearchDeleteForward => {
            if state.search_cursor < state.search_query.len() {
                state.search_query.remove(state.search_cursor);
            }
        }
        Action::SearchMoveCursorLeft => {
            if state.search_cursor > 0 {
                let mut prev = state.search_cursor - 1;
                while !state.search_query.is_char_boundary(prev) { prev -= 1; }
                state.search_cursor = prev;
            }
        }
        Action::SearchMoveCursorRight => {
            if state.search_cursor < state.search_query.len() {
                let mut next = state.search_cursor + 1;
                while !state.search_query.is_char_boundary(next) { next += 1; }
                state.search_cursor = next;
            }
        }
        Action::SearchMoveHome => { state.search_cursor = 0; }
        Action::SearchMoveEnd  => { state.search_cursor = state.search_query.len(); }
        Action::SearchKillToEnd => { state.search_query.truncate(state.search_cursor); }
        Action::SearchClear => {
            state.search_query.clear();
            state.search_cursor = 0;
        }
        Action::Search => {
            if !state.search_query.is_empty() {
                state.search_focused = false;
                state.screen = Screen::Results;
                state.search_results.clear();
                state.results_cursor = 0;
                
                let query = state.search_query.clone();
                state.toasts.info(format!("Searching: {}", query));
                let tx = state.action_tx.clone();
                let _ = tx.send(Action::AppendLog(format!("search: {}", query)));
                
                // Spawn yt-dlp search
                tokio::spawn(async move {
                    match crate::data::youtube::search(&query, 20).await {
                        Ok(videos) => {
                            let _ = tx.send(Action::SearchResults(videos));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::ShowToast(crate::widgets::toast::Severity::Error, format!("Search failed: {}", e)));
                            let _ = tx.send(Action::AppendLog(format!("yt-dlp error: {}", e)));
                        }
                    }
                });
            }
        }
        Action::SearchResults(videos) => {
            state.search_results = videos.clone();
            state.results_cursor = 0;
            if state.search_results.is_empty() {
                state.toasts.warn("No results found.");
            }

            for video in videos {
                if let Some(url) = video.thumbnail.clone() {
                    let tx = state.action_tx.clone();
                    let id = video.id.clone();
                    tokio::spawn(async move {
                        if let Ok(res) = reqwest::get(&url).await {
                            if let Ok(bytes) = res.bytes().await {
                                if let Ok(img) = image::load_from_memory(&bytes) {
                                    let _ = tx.send(Action::ThumbnailLoaded(id, img));
                                }
                            }
                        }
                    });
                }
            }
        }
        Action::ThumbnailLoaded(id, img) => {
            let protocol = state.picker.new_resize_protocol(img);
            state.thumbnails.insert(id, std::rc::Rc::new(crate::data::models::ThumbnailData {
                protocol: std::cell::RefCell::new(protocol),
            }));
        }

        // ── Player ────────────────────────────────────────────────────────────
        Action::PlayVideo(id) => {
            state.format_input_open = Some(id.clone());
            state.format_list = None;
            state.format_scroll = 0;
            state.format_input = "Loading defaults...".to_string();
            state.format_cursor = state.format_input.len();
            
            let tx = state.action_tx.clone();
            tokio::spawn(async move {
                let url = format!("https://www.youtube.com/watch?v={}", id);
                if let Ok(output) = tokio::process::Command::new("yt-dlp").arg("-F").arg(&url).output().await {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let lines: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
                    let _ = tx.send(Action::FormatsLoaded(lines));
                }
            });
        }
        Action::FormatsLoaded(lines) => {
            // Check if 401 and 251 exist
            let has_401 = lines.iter().any(|l| l.starts_with("401 ") || l.contains(" 401 "));
            let has_251 = lines.iter().any(|l| l.starts_with("251 ") || l.contains(" 251 "));
            
            if has_401 && has_251 {
                state.format_input = "401+251".to_string();
            } else {
                state.format_input = "bestvideo+bestaudio/best".to_string();
            }
            state.format_cursor = state.format_input.len();
            state.format_list = Some(lines);
        }
        Action::FormatScrollUp => {
            if state.format_scroll > 0 {
                state.format_scroll -= 1;
            }
        }
        Action::FormatScrollDown => {
            state.format_scroll += 1;
        }
        Action::CloseFormatInput => {
            state.format_input_open = None;
        }
        Action::FormatInputChar(c) => {
            state.format_input.insert(state.format_cursor, c);
            state.format_cursor += 1;
        }
        Action::FormatInputBackspace => {
            if state.format_cursor > 0 {
                state.format_cursor -= 1;
                state.format_input.remove(state.format_cursor);
            }
        }
        Action::FormatInputMoveCursorLeft => {
            if state.format_cursor > 0 {
                state.format_cursor -= 1;
            }
        }
        Action::FormatInputMoveCursorRight => {
            if state.format_cursor < state.format_input.len() {
                state.format_cursor += 1;
            }
        }
        Action::StartPlayback => {
            if let Some(id) = state.format_input_open.take() {
                let format = if state.format_input.is_empty() { "best" } else { &state.format_input };
                
                state.toasts.info(format!("Starting MPV... (ID: {})", id));
                state.screen = Screen::Home;
                crate::player::mpv::start_mpv(&id, format);
            }
        }
        Action::TogglePause => {
            state.player.paused = !state.player.paused;
            state.toasts.info(if state.player.paused { "Paused" } else { "Playing" });
        }
        Action::SeekBack => {
            state.player.position = (state.player.position - 5.0).max(0.0);
        }
        Action::SeekForward => {
            state.player.position += 5.0;
        }
        Action::VolumeUp => {
            state.player.volume = (state.player.volume + 5.0).min(200.0);
        }
        Action::VolumeDown => {
            state.player.volume = (state.player.volume - 5.0).max(0.0);
        }
        Action::PlayerUpdate(ps) => {
            state.player = ps;
        }

        Action::ToggleLogs => {
            state.logs_open = !state.logs_open;
        }
        Action::ToggleHelp => {
            state.help_open = !state.help_open;
        }
        Action::AppendLog(entry) => {
            state.log_entries.push(entry);
            // Keep at most 1 000 entries.
            if state.log_entries.len() > 1000 {
                state.log_entries.drain(..100);
            }
        }

        // ── Toast ─────────────────────────────────────────────────────────────
        Action::ShowToast(sev, msg) => {
            state.toasts.push(Toast::new(sev, msg));
        }

        // ── Tick ──────────────────────────────────────────────────────────────
        Action::Tick => {
            state.toasts.prune();
        }

        // ── Navigation helpers ────────────────────────────────────────────────
        // Per-screen list scrolling is handled in M2+ with a cursor field.
        Action::MoveUp => {
            if state.screen == Screen::Results && state.results_cursor >= 2 {
                state.results_cursor -= 2;
            }
        }
        Action::MoveDown => {
            if state.screen == Screen::Results && state.results_cursor + 2 < state.search_results.len() {
                state.results_cursor += 2;
            } else if state.screen == Screen::Results && state.results_cursor + 1 < state.search_results.len() {
                // if at the second to last row and there is an odd number of items
                state.results_cursor += 1;
            }
        }
        Action::MoveLeft => {
            if state.screen == Screen::Results && state.results_cursor % 2 != 0 {
                state.results_cursor -= 1;
            }
        }
        Action::MoveRight => {
            if state.screen == Screen::Results && state.results_cursor % 2 == 0 && state.results_cursor + 1 < state.search_results.len() {
                state.results_cursor += 1;
            }
        }
        // AddToQueue, CopyUrl, OpenQueue — stubs for M7.
        Action::AddToQueue | Action::CopyUrl | Action::OpenQueue => {
            state.toasts.info("Coming soon!");
        }
    }
}

// ── render() ─────────────────────────────────────────────────────────────────

/// Pure render pass — draws the complete UI for the current state.
///
/// No I/O, no async, no blocking. Called on every frame.
pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    // ── Overall layout: body | player_bar ───────────────────────────────────
    let player_bar_height: u16 = if state.player.idle { 3 } else { 4 };

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(player_bar_height),
        ])
        .split(area);

    let body_area = root[0];
    let bar_area  = root[1];

    // Fill background.
    frame.render_widget(
        ratatui::widgets::Block::default().style(ratatui::style::Style::default().bg(theme::BG)),
        body_area,
    );

    // ── Active screen ────────────────────────────────────────────────────────
    if state.logs_open {
        screens::logs::render(state, body_area, frame.buffer_mut());
    } else {
        match &state.screen {
            Screen::Home    => screens::home::render(state, body_area, frame.buffer_mut()),
            Screen::Results => screens::results::render(state, body_area, frame.buffer_mut()),
            Screen::Details => screens::details::render(state, body_area, frame.buffer_mut()),
            Screen::Logs    => screens::logs::render(state, body_area, frame.buffer_mut()),
            Screen::Help    => screens::home::render(state, body_area, frame.buffer_mut()),
        }
    }

    // ── Help overlay ─────────────────────────────────────────────────────────
    if state.help_open {
        screens::help::render(state, body_area, frame.buffer_mut());
    }

    // ── Format Input Modal ────────────────────────────────────────────────────
    if state.format_input_open.is_some() {
        let modal = crate::widgets::modal::Modal::new("yt-dlp Format Code")
            .width(0.8)
            .height(0.8);
        
        let inner = modal.render_and_inner(body_area, frame.buffer_mut());
        
        let list_area = ratatui::layout::Rect {
            x: inner.x + 1,
            y: inner.y,
            width: inner.width.saturating_sub(2),
            height: inner.height.saturating_sub(4),
        };
        
        if let Some(lines) = &state.format_list {
            let max_scroll = lines.len().saturating_sub(list_area.height as usize);
            let scroll = state.format_scroll.min(max_scroll);
            
            let text_lines: Vec<ratatui::text::Line> = lines.iter()
                .skip(scroll)
                .take(list_area.height as usize)
                .map(|l| ratatui::text::Line::from(ratatui::text::Span::styled(l, ratatui::style::Style::default().fg(crate::theme::FG))))
                .collect();
                
            ratatui::widgets::Paragraph::new(text_lines)
                .render(list_area, frame.buffer_mut());
        } else {
            ratatui::widgets::Paragraph::new(ratatui::text::Span::styled("Loading formats... (runs yt-dlp -F)", crate::theme::muted()))
                .render(list_area, frame.buffer_mut());
        }
        
        let mut input = crate::widgets::input::TextInput::new("e.g. bestvideo+bestaudio/best");
        input.value = state.format_input.clone();
        input.cursor = state.format_cursor;
        input.focused = true;
        
        let input_area = ratatui::layout::Rect {
            x: inner.x + 1,
            y: inner.y + inner.height.saturating_sub(3),
            width: inner.width.saturating_sub(2),
            height: 3,
        };
        
        frame.render_widget(&input, input_area);
    }

    // ── Player bar ───────────────────────────────────────────────────────────
    frame.render_widget(PlayerBar::new(&state.player), bar_area);

    // ── Toasts (top-right, above everything) ─────────────────────────────────
    state.toasts.render(area, frame.buffer_mut());
}

// ── Event loop ────────────────────────────────────────────────────────────────

/// Run the main event loop until `state.should_quit` is set.
///
/// Receives crossterm key events and internal actions via `action_rx`,
/// calling `update()` for each and then `render()`.
pub async fn run_event_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    state: &mut AppState,
    action_rx: &mut mpsc::UnboundedReceiver<Action>,
) -> Result<()> {
    use crossterm::event::EventStream;
    use futures::StreamExt;

    let tick_rate = Duration::from_millis(50); // ~20 fps
    let mut event_stream = EventStream::new();
    let mut last_tick = tokio::time::Instant::now();

    // Initial render.
    terminal.draw(|f| render(f, state))?;

    loop {
        let tick_delay = tick_rate.saturating_sub(last_tick.elapsed());

        tokio::select! {
            // ── Crossterm events ─────────────────────────────────────────────
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(TermEvent::Key(key))) => {
                        if let Some(action) = key_to_action(state, key) {
                            update(state, action);
                        }
                    }
                    Some(Ok(_)) => {} // mouse / resize / paste – ignore for now
                    Some(Err(e)) => {
                        tracing::warn!("crossterm event error: {e}");
                    }
                    None => break, // stdin closed
                }
            }

            // ── Internal actions ─────────────────────────────────────────────
            Some(action) = action_rx.recv() => {
                update(state, action);
            }

            // ── Tick ─────────────────────────────────────────────────────────
            _ = tokio::time::sleep(tick_delay) => {
                last_tick = tokio::time::Instant::now();
                update(state, Action::Tick);
            }
        }

        // Render after every event.
        terminal.draw(|f| render(f, state))?;

        if state.should_quit {
            break;
        }
    }

    Ok(())
}

// ── Key-to-action translation ─────────────────────────────────────────────────

/// Translate a crossterm [`KeyEvent`] into an [`Action`] based on current state.
///
/// Returns `None` for keys that should be ignored.
fn key_to_action(state: &AppState, key: KeyEvent) -> Option<Action> {
    use KeyCode::*;

    // ── Ctrl+L always opens logs ──────────────────────────────────────────────
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == Char('l') {
        return Some(Action::ToggleLogs);
    }

    // ── Search-focused mode ───────────────────────────────────────────────────
    if state.search_focused {
        return match key.code {
            Esc => Some(Action::Back),
            Enter => Some(Action::Search),
            Backspace => Some(Action::SearchBackspace),
            Delete => Some(Action::SearchDeleteForward),
            Left => Some(Action::SearchMoveCursorLeft),
            Right => Some(Action::SearchMoveCursorRight),
            Home => Some(Action::SearchMoveHome),
            End => Some(Action::SearchMoveEnd),
            Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::SearchClear),
            Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::SearchKillToEnd),
            Char(c) => Some(Action::SearchInput(c)),
            _ => None,
        };
    }

    // ── Format Input Modal ────────────────────────────────────────────────────
    if state.format_input_open.is_some() {
        return match key.code {
            Esc => Some(Action::CloseFormatInput),
            Enter => Some(Action::StartPlayback),
            Backspace => Some(Action::FormatInputBackspace),
            Left => Some(Action::FormatInputMoveCursorLeft),
            Right => Some(Action::FormatInputMoveCursorRight),
            Up | Char('k') => Some(Action::FormatScrollUp),
            Down | Char('j') => Some(Action::FormatScrollDown),
            PageUp => {
                for _ in 0..10 { state.action_tx.send(Action::FormatScrollUp).ok(); }
                None
            }
            PageDown => {
                for _ in 0..10 { state.action_tx.send(Action::FormatScrollDown).ok(); }
                None
            }
            Char(c) => Some(Action::FormatInputChar(c)),
            _ => None,
        };
    }

    // ── Normal mode ───────────────────────────────────────────────────────────
    match key.code {
        Char('q') => Some(Action::Quit),
        Char('/') => Some(Action::FocusSearch),
        Esc       => Some(Action::Back),
        Enter     => {
            if state.screen == Screen::Results && state.results_cursor < state.search_results.len() {
                Some(Action::PlayVideo(state.search_results[state.results_cursor].id.clone()))
            } else {
                None
            }
        }
        Char(' ') => Some(Action::TogglePause),
        Char('h') | Left => {
            if state.screen == Screen::Results {
                Some(Action::MoveLeft)
            } else {
                Some(Action::SeekBack)
            }
        }
        Char('l') | Right => {
            if state.screen == Screen::Results {
                Some(Action::MoveRight)
            } else {
                Some(Action::SeekForward)
            }
        }
        Char('K') | Char('+') => Some(Action::VolumeUp),
        Char('J') | Char('-') => Some(Action::VolumeDown),
        Char('j') | Down      => Some(Action::MoveDown),
        Char('k') | Up        => Some(Action::MoveUp),
        Char('?')         => Some(Action::ToggleHelp),
        _ => None,
    }
}
