//! ytui — keyboard-first terminal YouTube client.
//!
//! # Entry point responsibilities
//!
//! 1. Initialise the tracing subscriber (including the in-app log layer).
//! 2. Load config.
//! 3. Check that `yt-dlp` and `mpv` exist on `PATH` (warn if not).
//! 4. Set up the alternate-screen / raw-mode terminal with a panic hook that
//!    guarantees terminal teardown even on a panic.
//! 5. Run the event loop.
//! 6. Restore the terminal on clean exit.

// M1: many items are stubs that will be used in later milestones.
#![allow(dead_code)]
#![allow(unused_imports)]

mod app;
mod config;
mod data;
mod logging;
mod player;
mod screens;
mod tests;
mod theme;
mod widgets;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{
    app::{run_event_loop, update, Action, AppState},
    config::Config,
    logging::InAppLogLayer,
    widgets::toast::Severity,
};

#[tokio::main]
async fn main() -> Result<()> {
    // ── Tracing ───────────────────────────────────────────────────────────────
    let log_buffer: logging::LogBuffer = Arc::new(Mutex::new(Vec::new()));
    let in_app_layer = InAppLogLayer::new(log_buffer.clone());

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(in_app_layer)
        .init();

    tracing::info!("ytui starting up");

    // ── Config ────────────────────────────────────────────────────────────────
    let config = Config::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config: {e}; using defaults");
        Config::default()
    });

    // ── Dependency check ──────────────────────────────────────────────────────
    let (ytdlp_ok, mpv_ok) = check_dependencies(&config);

    // ── Action channel ────────────────────────────────────────────────────────
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

    // ── App state ─────────────────────────────────────────────────────────────
    let mut state = AppState::new(config, action_tx.clone());

    // Flush any buffered log lines gathered before the state was ready.
    {
        let buf = log_buffer.lock().unwrap();
        for line in buf.iter() {
            state.log_entries.push(line.clone());
        }
    }

    // Warn about missing dependencies.
    if !ytdlp_ok {
        update(
            &mut state,
            Action::ShowToast(
                Severity::Warning,
                "yt-dlp not found on PATH. Search won't work.".into(),
            ),
        );
        update(
            &mut state,
            Action::AppendLog("[WARN] yt-dlp binary not found on PATH".into()),
        );
    }
    if !mpv_ok {
        update(
            &mut state,
            Action::ShowToast(
                Severity::Warning,
                "mpv not found on PATH. Playback won't work.".into(),
            ),
        );
        update(
            &mut state,
            Action::AppendLog("[WARN] mpv binary not found on PATH".into()),
        );
    }

    // ── Panic hook ────────────────────────────────────────────────────────────
    // Install a hook that restores the terminal before printing the panic.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Attempt terminal restore — ignore errors (we're already panicking).
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    // ── Terminal setup ────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // ── Event loop ────────────────────────────────────────────────────────────
    let result = run_event_loop(&mut terminal, &mut state, &mut action_rx).await;

    // ── Terminal teardown ─────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("ytui error: {e:?}");
        std::process::exit(1);
    }

    Ok(())
}

// ── Dependency detection ──────────────────────────────────────────────────────

/// Returns `(yt-dlp_found, mpv_found)`.
fn check_dependencies(config: &Config) -> (bool, bool) {
    let ytdlp = which_binary(config.ytdlp_bin().to_str().unwrap_or("yt-dlp"));
    let mpv    = which_binary(config.mpv_bin().to_str().unwrap_or("mpv"));
    if ytdlp { tracing::info!("yt-dlp found on PATH"); } else { tracing::warn!("yt-dlp NOT found"); }
    if mpv   { tracing::info!("mpv found on PATH");    } else { tracing::warn!("mpv NOT found");    }
    (ytdlp, mpv)
}

/// Return `true` if `name` resolves to an executable on the system `PATH`.
fn which_binary(name: &str) -> bool {
    // Fast path: use the `which`-style approach via std::process.
    std::process::Command::new(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}
