use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

use super::models::{Video, VideoSource};

#[derive(Debug, Deserialize)]
struct YtDlpEntry {
    id: Option<String>,
    title: Option<String>,
    uploader: Option<String>,
    uploader_id: Option<String>,
    duration: Option<f64>,
    view_count: Option<u64>,
    upload_date: Option<String>,
    thumbnails: Option<Vec<Thumbnail>>,
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: String,
}

pub async fn search(query: &str, limit: usize) -> Result<Vec<Video>> {
    let search_arg = format!("ytsearch{}:{}", limit, query);

    let output = Command::new("yt-dlp")
        .args([
            "--dump-json",
            "--flat-playlist",
            "--no-warnings",
            &search_arg,
        ])
        .output()
        .await
        .context("Failed to execute yt-dlp. Is it installed?")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp search failed: {}", err);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut videos = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<YtDlpEntry>(line) {
            Ok(entry) => {
                if let (Some(id), Some(title)) = (entry.id, entry.title) {
                    let duration_str = entry.duration.map(|d| {
                        let total = d as u64;
                        let m = total / 60;
                        let s = total % 60;
                        if m >= 60 {
                            let h = m / 60;
                            let m = m % 60;
                            format!("{h}:{m:02}:{s:02}")
                        } else {
                            format!("{m}:{s:02}")
                        }
                    });

                    videos.push(Video {
                        id,
                        title,
                        channel: entry.uploader.unwrap_or_default(),
                        channel_id: entry.uploader_id.unwrap_or_default(),
                        duration: duration_str,
                        duration_secs: entry.duration,
                        view_count: entry.view_count,
                        upload_date: entry.upload_date,
                        thumbnail: entry.thumbnails.and_then(|t| t.last().map(|tn| tn.url.clone())),
                        description: None,
                        source: VideoSource::Search,
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse yt-dlp output line: {}. Line: {}", e, line);
            }
        }
    }

    Ok(videos)
}
