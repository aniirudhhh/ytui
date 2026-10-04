use tokio::process::Command;
use std::process::Stdio;

pub fn start_mpv(video_id: &str, format: &str) {
    let url = format!("https://www.youtube.com/watch?v={}", video_id);
    let mut args = vec![
        "--no-terminal".to_string(), // Don't spam our terminal with logs
        format!("--ytdl-format={}", format),
        url,
    ];

    if format == "bestaudio" {
        args.push("--no-video".to_string());
    }

    tokio::spawn(async move {
        tracing::info!("Starting MPV with args: {:?}", args);
        let mut child = match Command::new("mpv")
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                tracing::error!("Failed to start mpv: {}", e);
                return;
            }
        };

        let _ = child.wait().await;
        tracing::info!("MPV process exited.");
    });
}
