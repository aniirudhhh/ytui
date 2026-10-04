/// Configuration file loading, defaults, and keybind mapping.
///
/// Config is read from `~/.config/ytui/config.toml` at startup.
/// Missing keys fall back to compiled-in defaults.
use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

// ── Keybind ───────────────────────────────────────────────────────────────────

/// An abstract action triggered by a key press.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyAction {
    Search,
    Confirm,
    Back,
    Quit,
    TogglePause,
    SeekBack,
    SeekForward,
    MoveUp,
    MoveDown,
    VolumeUp,
    VolumeDown,
    AddToQueue,
    CopyUrl,
    OpenQueue,
    ToggleLogs,
    Help,
    NextPage,
    PrevPage,
}

// ── Player config ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    /// Open mpv with a video window (`false` = audio-only).
    #[serde(default = "default_true")]
    pub video: bool,
    /// Path override for the mpv binary.
    #[serde(default)]
    pub mpv_path: Option<PathBuf>,
    /// Path override for yt-dlp.
    #[serde(default)]
    pub ytdlp_path: Option<PathBuf>,
    /// Save position every N seconds.
    #[serde(default = "default_save_interval")]
    pub save_interval_secs: u64,
    /// IPC socket path (defaults to a temp path).
    #[serde(default)]
    pub ipc_socket: Option<PathBuf>,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            video: true,
            mpv_path: None,
            ytdlp_path: None,
            save_interval_secs: 5,
            ipc_socket: None,
        }
    }
}

// ── Cache config ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// TTL for search result cache entries (seconds).
    #[serde(default = "default_cache_ttl")]
    pub search_ttl_secs: u64,
    /// TTL for video-info cache entries (seconds).
    #[serde(default = "default_cache_ttl")]
    pub info_ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            search_ttl_secs: 3600,
            info_ttl_secs: 3600,
        }
    }
}

// ── UI config ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Milliseconds before search is triggered after the last keystroke.
    #[serde(default = "default_debounce")]
    pub search_debounce_ms: u64,
    /// Number of results per search page.
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    /// Show thumbnails if the terminal supports it.
    #[serde(default = "default_true")]
    pub thumbnails: bool,
    /// Preferred thumbnail protocol: "kitty", "sixel", "halfblocks", "auto".
    #[serde(default = "default_thumb_proto")]
    pub thumbnail_protocol: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            search_debounce_ms: 400,
            page_size: 20,
            thumbnails: true,
            thumbnail_protocol: "auto".into(),
        }
    }
}

// ── Top-level config ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub player: PlayerConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub ui: UiConfig,
    /// Custom keybinds (key string → action name).
    /// e.g. `"ctrl+l" = "toggle_logs"`
    #[serde(default)]
    pub keys: HashMap<String, KeyAction>,
}
// Config manually implements Default using the #[derive] below.

impl Config {
    /// Load config from the standard platform path, creating it with defaults
    /// if it does not exist.
    pub fn load() -> Result<Self> {
        let path = config_path();
        if let Some(p) = &path {
            if p.exists() {
                let raw = std::fs::read_to_string(p)?;
                let cfg: Config = toml::from_str(&raw)?;
                return Ok(cfg);
            }
            // Create parent dirs and write defaults.
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let cfg = Config::default();
            std::fs::write(p, toml::to_string_pretty(&cfg)?)?;
            return Ok(cfg);
        }
        Ok(Config::default())
    }

    /// Resolve the effective binary path for yt-dlp.
    pub fn ytdlp_bin(&self) -> PathBuf {
        self.player
            .ytdlp_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("yt-dlp"))
    }

    /// Resolve the effective binary path for mpv.
    pub fn mpv_bin(&self) -> PathBuf {
        self.player
            .mpv_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("mpv"))
    }
}

/// Returns the platform-specific config file path.
pub fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "ytui")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}

// ── Default value fns (required by serde) ────────────────────────────────────

fn default_true() -> bool { true }
fn default_save_interval() -> u64 { 5 }
fn default_cache_ttl() -> u64 { 3600 }
fn default_debounce() -> u64 { 400 }
fn default_page_size() -> usize { 20 }
fn default_thumb_proto() -> String { "auto".into() }

// ── Keybind resolution ────────────────────────────────────────────────────────

/// Default keyboard map (key string → action).
pub fn default_keymap() -> HashMap<String, KeyAction> {
    let mut m = HashMap::new();
    m.insert("/".into(),       KeyAction::Search);
    m.insert("enter".into(),   KeyAction::Confirm);
    m.insert("esc".into(),     KeyAction::Back);
    m.insert("q".into(),       KeyAction::Quit);
    m.insert(" ".into(),       KeyAction::TogglePause);
    m.insert("h".into(),       KeyAction::SeekBack);
    m.insert("l".into(),       KeyAction::SeekForward);
    m.insert("j".into(),       KeyAction::MoveDown);
    m.insert("k".into(),       KeyAction::MoveUp);
    m.insert("J".into(),       KeyAction::VolumeDown);
    m.insert("K".into(),       KeyAction::VolumeUp);
    m.insert("a".into(),       KeyAction::AddToQueue);
    m.insert("y".into(),       KeyAction::CopyUrl);
    m.insert("Q".into(),       KeyAction::OpenQueue);
    m.insert("ctrl+l".into(),  KeyAction::ToggleLogs);
    m.insert("?".into(),       KeyAction::Help);
    m
}
