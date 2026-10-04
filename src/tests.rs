/// Unit tests for M1: theme helpers, TextInput, toast stack, and the update()
/// reducer.  No I/O, no yt-dlp, no mpv.
#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use crate::{
        app::{update, Action, AppState, Screen},
        config::Config,
        widgets::toast::Severity,
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_state() -> (AppState, mpsc::UnboundedReceiver<Action>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let state = AppState::new(Config::default(), tx);
        (state, rx)
    }

    // ── Screen navigation ─────────────────────────────────────────────────────

    #[test]
    fn navigate_to_results_and_back() {
        let (mut s, _rx) = make_state();
        assert_eq!(s.screen, Screen::Home);
        update(&mut s, Action::NavigateTo(Screen::Results));
        assert_eq!(s.screen, Screen::Results);
        update(&mut s, Action::Back);
        assert_eq!(s.screen, Screen::Home);
    }

    #[test]
    fn back_from_home_stays_home() {
        let (mut s, _rx) = make_state();
        update(&mut s, Action::Back);
        assert_eq!(s.screen, Screen::Home);
    }

    // ── Quit ─────────────────────────────────────────────────────────────────

    #[test]
    fn quit_sets_flag() {
        let (mut s, _rx) = make_state();
        assert!(!s.should_quit);
        update(&mut s, Action::Quit);
        assert!(s.should_quit);
    }

    // ── Search ────────────────────────────────────────────────────────────────

    #[test]
    fn focus_search_sets_flag() {
        let (mut s, _rx) = make_state();
        assert!(!s.search_focused);
        update(&mut s, Action::FocusSearch);
        assert!(s.search_focused);
    }

    #[test]
    fn search_input_builds_query() {
        let (mut s, _rx) = make_state();
        for c in "hello".chars() {
            update(&mut s, Action::SearchInput(c));
        }
        assert_eq!(s.search_query, "hello");
        assert_eq!(s.search_cursor, 5);
    }

    #[test]
    fn search_backspace_removes_char() {
        let (mut s, _rx) = make_state();
        update(&mut s, Action::SearchInput('a'));
        update(&mut s, Action::SearchInput('b'));
        update(&mut s, Action::SearchBackspace);
        assert_eq!(s.search_query, "a");
        assert_eq!(s.search_cursor, 1);
    }

    #[test]
    fn search_clear_empties_query() {
        let (mut s, _rx) = make_state();
        for c in "test".chars() {
            update(&mut s, Action::SearchInput(c));
        }
        update(&mut s, Action::SearchClear);
        assert!(s.search_query.is_empty());
        assert_eq!(s.search_cursor, 0);
    }

    #[test]
    fn search_cursor_movement() {
        let (mut s, _rx) = make_state();
        for c in "abc".chars() {
            update(&mut s, Action::SearchInput(c));
        }
        assert_eq!(s.search_cursor, 3);
        update(&mut s, Action::SearchMoveCursorLeft);
        assert_eq!(s.search_cursor, 2);
        update(&mut s, Action::SearchMoveHome);
        assert_eq!(s.search_cursor, 0);
        update(&mut s, Action::SearchMoveEnd);
        assert_eq!(s.search_cursor, 3);
    }

    #[tokio::test]
    async fn search_submit_navigates_to_results() {
        let (mut s, _rx) = make_state();
        for c in "ratatui".chars() {
            update(&mut s, Action::SearchInput(c));
        }
        update(&mut s, Action::FocusSearch);
        update(&mut s, Action::Search);
        assert_eq!(s.screen, Screen::Results);
        assert!(!s.search_focused);
    }

    #[test]
    fn empty_search_does_not_navigate() {
        let (mut s, _rx) = make_state();
        update(&mut s, Action::FocusSearch);
        update(&mut s, Action::Search);
        // Empty query: no navigation.
        assert_eq!(s.screen, Screen::Home);
    }

    // ── Toasts ────────────────────────────────────────────────────────────────

    #[test]
    fn show_toast_adds_entry() {
        let (mut s, _rx) = make_state();
        update(
            &mut s,
            Action::ShowToast(Severity::Info, "hello".into()),
        );
        assert!(!s.toasts.is_empty());
    }

    // ── Player ────────────────────────────────────────────────────────────────

    #[test]
    fn toggle_pause_flips_flag() {
        let (mut s, _rx) = make_state();
        assert!(!s.player.paused);
        update(&mut s, Action::TogglePause);
        assert!(s.player.paused);
        update(&mut s, Action::TogglePause);
        assert!(!s.player.paused);
    }

    #[test]
    fn volume_clamped_to_range() {
        let (mut s, _rx) = make_state();
        s.player.volume = 0.0;
        update(&mut s, Action::VolumeDown);
        assert_eq!(s.player.volume, 0.0); // clamped at 0

        s.player.volume = 200.0;
        update(&mut s, Action::VolumeUp);
        assert_eq!(s.player.volume, 200.0); // clamped at 200
    }

    #[test]
    fn seek_back_clamped_at_zero() {
        let (mut s, _rx) = make_state();
        s.player.position = 2.0;
        update(&mut s, Action::SeekBack);
        assert_eq!(s.player.position, 0.0); // 2 - 5 → clamped
    }

    // ── Logs ──────────────────────────────────────────────────────────────────

    #[test]
    fn toggle_logs_flips_flag() {
        let (mut s, _rx) = make_state();
        assert!(!s.logs_open);
        update(&mut s, Action::ToggleLogs);
        assert!(s.logs_open);
        update(&mut s, Action::ToggleLogs);
        assert!(!s.logs_open);
    }

    #[test]
    fn append_log_grows_buffer() {
        let (mut s, _rx) = make_state();
        update(&mut s, Action::AppendLog("entry 1".into()));
        update(&mut s, Action::AppendLog("entry 2".into()));
        assert_eq!(s.log_entries.len(), 2);
    }

    // ── Help ──────────────────────────────────────────────────────────────────

    #[test]
    fn toggle_help_flips_flag() {
        let (mut s, _rx) = make_state();
        assert!(!s.help_open);
        update(&mut s, Action::ToggleHelp);
        assert!(s.help_open);
        update(&mut s, Action::ToggleHelp);
        assert!(!s.help_open);
    }

    // ── TextInput widget ──────────────────────────────────────────────────────

    #[test]
    fn text_input_insert_and_backspace() {
        use crate::widgets::input::TextInput;
        let mut inp = TextInput::new("placeholder");
        inp.insert('a');
        inp.insert('b');
        inp.insert('c');
        assert_eq!(inp.value, "abc");
        inp.backspace();
        assert_eq!(inp.value, "ab");
    }

    #[test]
    fn text_input_move_and_insert() {
        use crate::widgets::input::TextInput;
        let mut inp = TextInput::new("");
        for c in "abc".chars() { inp.insert(c); }
        inp.move_home();
        inp.insert('X');
        assert_eq!(inp.value, "Xabc");
    }
}
