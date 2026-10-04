# ytui

> Keyboard-first terminal YouTube client — browse, search, and stream YouTube from your terminal.

![ytui dark theme TUI with search bar, results list, and player bar]()

---

## Features

- 🔍 **Search** YouTube via `yt-dlp` (no API key required)
- 📺 **Stream** to `mpv` — full video or audio-only
- 📋 **Quality picker** — choose format with yt-dlp format codes
- 📡 **Subscription feed** — via YouTube RSS channel feeds
- 🕓 **Watch history** — resume from where you left off
- 🎵 **Persistent player bar** — progress, volume, pause state
- 🔔 **Toast notifications** and in-app **log panel**
- ⌨️  **Fully keyboard-driven** — all binds configurable
- 💾 **SQLite cache** — instant results, background refresh

---

## Requirements

| Dependency | Purpose                      | Install                           |
|------------|------------------------------|-----------------------------------|
| `yt-dlp`   | Search, format extraction    | `pip install yt-dlp` or pkg mgr   |
| `mpv`      | Video/audio playback         | `apt/brew/pkg install mpv`        |
| Rust 1.70+ | Build toolchain              | `curl https://sh.rustup.rs -sSf \| sh` |

---

## Install

```bash
# From source
git clone https://github.com/you/ytui
cd ytui
cargo install --path .

# Or (once published)
cargo install ytui
```

---

## Running

```bash
ytui
```

On first launch ytui creates a default config at:
- Linux/macOS: `~/.config/ytui/config.toml`
- Windows: `%APPDATA%\ytui\config.toml`

---

## Keybinds

| Key        | Action                  |
|------------|-------------------------|
| `/`        | Focus search bar        |
| `Enter`    | Confirm / open / play   |
| `Esc`      | Back / close modal      |
| `q`        | Quit                    |
| `Space`    | Pause / resume          |
| `h` / `←` | Seek back 5 s           |
| `l` / `→` | Seek forward 5 s        |
| `j` / `↓` | Move down               |
| `k` / `↑` | Move up                 |
| `J`        | Volume down             |
| `K`        | Volume up               |
| `a`        | Add to queue            |
| `y`        | Copy video URL          |
| `Q`        | Open queue              |
| `Ctrl+L`   | Toggle log panel        |
| `?`        | Toggle help overlay     |

All keybinds are configurable in `config.toml`.

---

## Configuration

```toml
# ~/.config/ytui/config.toml

[player]
video = true               # false = audio-only (--no-video)
# mpv_path = "/usr/bin/mpv"
# ytdlp_path = "/usr/local/bin/yt-dlp"
save_interval_secs = 5

[cache]
search_ttl_secs = 3600
info_ttl_secs = 3600

[ui]
search_debounce_ms = 400
page_size = 20
thumbnails = true
thumbnail_protocol = "auto"   # kitty | sixel | halfblocks | auto

# Custom keybinds (action names from the source)
# [keys]
# "ctrl+f" = "search"
```

---

## Architecture

```
Input / mpv events
        │ Action
        ▼
   update(state, action)   ── spawns tokio tasks ──►  yt-dlp / mpv / SQLite
        │                                                       │
        │                                             result as Action
        ▼
   render(frame, &state)   ── pure, no I/O
```

- **`app.rs`** — `AppState`, `Action` enum, `update()`, event loop
- **`config.rs`** — TOML config + keybind map
- **`theme.rs`** — dark palette, style helpers
- **`data/`** — yt-dlp wrapper, RSS feed, SQLite cache, shared models
- **`player/`** — mpv spawn + IPC socket controller
- **`screens/`** — Home, Results, Details, Logs, Help
- **`widgets/`** — VideoCard, PlayerBar, TextInput, Toast, Modal, Badge

---

## Milestones

| # | Contents                                              | Status |
|---|-------------------------------------------------------|--------|
| M1 | Terminal setup, event loop, theme, Home screen, search input, toasts | ✅ |
| M2 | yt-dlp search, results list, SQLite cache             | ⏳ |
| M3 | mpv playback + player bar                             | ⏳ |
| M4 | Subscription feed via RSS                             | ⏳ |
| M5 | Watch history, continue watching, resume              | ⏳ |
| M6 | Details screen + quality modal                        | ⏳ |
| M7 | Queue, log panel, help overlay, config file           | ⏳ |
| M8 | Thumbnails (kitty/sixel/halfblocks), polish, README   | ⏳ |

---

## License

MIT
