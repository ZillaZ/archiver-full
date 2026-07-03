use shared::messages::{ScrapedVideo, ScrapedLive, ScrapeResult};
use std::process::Command;

pub async fn fetch_metadata(
    platform: &str,
    platform_id: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<ScrapeResult, String> {
    let cookies_file = write_cookies_temp(platform, cookies).await;

    let result = match platform {
        "youtube" => fetch_youtube(platform_id, proxy, cookies_file.as_deref()).await,
        "tiktok" => fetch_tiktok(platform_id, proxy, cookies_file.as_deref()).await,
        "instagram" => fetch_instagram(platform_id, proxy, cookies_file.as_deref()).await,
        _ => Err(format!("unsupported platform: {platform}")),
    };

    cleanup_cookies(cookies_file).await;
    result
}

pub async fn fetch_video_heatmap(
    platform: &str,
    platform_id: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<Option<serde_json::Value>, String> {
    if platform != "youtube" {
        return Ok(None);
    }

    let cookies_file = write_cookies_temp(platform, cookies).await;
    let url = format!("https://www.youtube.com/watch?v={platform_id}");

    let mut args = vec![
        &url as &str,
        "--dump-json",
        "--no-warnings",
        "--ignore-errors",
        "--extractor-args", "youtube:include_heatmap=1",
    ];

    if let Some(p) = proxy {
        args.push("--proxy");
        args.push(p);
    }
    if let Some(ref c) = cookies_file {
        args.push("--cookies");
        args.push(c);
    }

    let output = Command::new("yt-dlp")
        .args(&args)
        .output()
        .map_err(|e| format!("run yt-dlp: {e}"))?;

    cleanup_cookies(cookies_file).await;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entry: serde_json::Value = serde_json::from_str(stdout.lines().next().unwrap_or(""))
        .map_err(|e| format!("parse json: {e}"))?;

    if let Some(heatmap) = entry.get("heatmap") {
        let segments: Vec<serde_json::Value> = heatmap
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|p| {
                serde_json::json!({
                    "start_time": p["start_time"].as_f64().unwrap_or(0.0),
                    "end_time": p["end_time"].as_f64().unwrap_or(0.0),
                    "value": p["value"].as_f64().unwrap_or(0.0),
                })
            })
            .collect();

        if segments.is_empty() {
            return Ok(None);
        }

        Ok(Some(serde_json::json!({ "segments": segments })))
    } else {
        Ok(None)
    }
}

async fn write_cookies_temp(platform: &str, cookies: Option<&str>) -> Option<String> {
    if let Some(c) = cookies {
        if !c.is_empty() {
            let path = format!("/tmp/cookies_{platform}.txt");
            tokio::fs::write(&path, c).await.ok()?;
            return Some(path);
        }
    }
    None
}

async fn cleanup_cookies(path: Option<String>) {
    if let Some(p) = path {
        let _ = tokio::fs::remove_file(p).await;
    }
}

fn build_ytdlp_args(
    url: &str,
    proxy: Option<&str>,
    cookies_file: Option<&str>,
    extra: &[&str],
) -> Vec<String> {
    let mut args = vec![
        url.into(),
        "--dump-json".into(),
        "--no-warnings".into(),
        "--ignore-errors".into(),
        "--flat-playlist".into(),
    ];

    if let Some(p) = proxy {
        args.push("--proxy".into());
        args.push(p.into());
    }
    if let Some(c) = cookies_file {
        args.push("--cookies".into());
        args.push(c.into());
    }

    args.extend(extra.iter().map(|s| s.to_string()));
    args
}

async fn run_ytdlp(args: &[String]) -> Result<Vec<serde_json::Value>, String> {
    let output = Command::new("yt-dlp")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run yt-dlp: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("403") || stderr.contains("Forbidden") {
            return Err("403".into());
        }
        return Err(format!("yt-dlp failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<serde_json::Value> = stdout
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    Ok(entries)
}

async fn fetch_youtube(
    channel_id: &str,
    proxy: Option<&str>,
    cookies_file: Option<&str>,
) -> Result<ScrapeResult, String> {
    let url = if channel_id.starts_with("UC") {
        format!("https://www.youtube.com/channel/{channel_id}/videos")
    } else {
        format!("https://www.youtube.com/@{channel_id}/videos")
    };

    let args = build_ytdlp_args(&url, proxy, cookies_file, &[]);
    let entries = run_ytdlp(&args).await?;

    let mut videos = Vec::new();
    let mut active_lives = Vec::new();

    for entry in &entries {
        let title = entry["title"].as_str().unwrap_or("").to_string();
        let uploader = entry["channel"].as_str().unwrap_or("").to_string();
        let uploader_id = entry["channel_id"].as_str().unwrap_or("").to_string();
        let pid = entry["id"].as_str().unwrap_or("").to_string();
        let is_live = entry["live_status"]
            .as_str()
            .map(|s| s == "is_live" || s == "is_upcoming")
            .unwrap_or(false);

        if is_live {
            active_lives.push(ScrapedLive {
                platform_id: pid, title, uploader, uploader_id,
                thumbnail_url: entry["thumbnail"].as_str().map(|s| s.to_string()),
                description: entry["description"].as_str().map(|s| s.to_string()),
            });
        } else {
            videos.push(ScrapedVideo {
                platform_id: pid, title, uploader, uploader_id,
                duration: entry["duration"].as_i64().unwrap_or(0) as i32,
                upload_date: entry["upload_date"].as_str().map(|s| s.to_string()),
                thumbnail_url: entry["thumbnail"].as_str().map(|s| s.to_string()),
                description: entry["description"].as_str().map(|s| s.to_string()),
            });
        }
    }

    Ok(ScrapeResult { channel_id: uuid::Uuid::nil(), videos, active_lives })
}

async fn fetch_tiktok(
    user_id: &str,
    proxy: Option<&str>,
    cookies_file: Option<&str>,
) -> Result<ScrapeResult, String> {
    let url = format!("https://www.tiktok.com/@{user_id}");
    let args = build_ytdlp_args(&url, proxy, cookies_file, &["--extract-flat"]);
    let entries = run_ytdlp(&args).await?;

    let mut videos = Vec::new();
    let mut active_lives = Vec::new();

    for entry in &entries {
        let title = entry["title"].as_str().unwrap_or("").to_string();
        let uploader = entry["uploader"].as_str().unwrap_or("").to_string();
        let uploader_id = entry["uploader_id"].as_str().unwrap_or("").to_string();
        let pid = entry["id"].as_str().unwrap_or("").to_string();
        let is_live = entry["live_status"].as_str().map(|s| s == "is_live").unwrap_or(false);

        if is_live {
            active_lives.push(ScrapedLive {
                platform_id: pid, title, uploader, uploader_id,
                thumbnail_url: entry["thumbnail"].as_str().map(|s| s.to_string()),
                description: entry["description"].as_str().map(|s| s.to_string()),
            });
        } else {
            videos.push(ScrapedVideo {
                platform_id: pid, title, uploader, uploader_id,
                duration: entry["duration"].as_i64().unwrap_or(0) as i32,
                upload_date: entry["upload_date"].as_str().map(|s| s.to_string()),
                thumbnail_url: entry["thumbnail"].as_str().map(|s| s.to_string()),
                description: entry["description"].as_str().map(|s| s.to_string()),
            });
        }
    }

    Ok(ScrapeResult { channel_id: uuid::Uuid::nil(), videos, active_lives })
}

async fn fetch_instagram(
    user_id: &str,
    proxy: Option<&str>,
    cookies_file: Option<&str>,
) -> Result<ScrapeResult, String> {
    let url = format!("https://www.instagram.com/{user_id}/");
    let args = build_ytdlp_args(&url, proxy, cookies_file, &[]);
    let entries = run_ytdlp(&args).await?;

    let mut videos = Vec::new();
    let active_lives: Vec<ScrapedLive> = Vec::new();

    for entry in &entries {
        let title = entry["title"].as_str().unwrap_or("").to_string();
        let uploader = entry["uploader"].as_str().unwrap_or("").to_string();
        let uploader_id = entry["uploader_id"].as_str().unwrap_or("").to_string();
        let pid = entry["id"].as_str().unwrap_or("").to_string();

        videos.push(ScrapedVideo {
            platform_id: pid, title, uploader, uploader_id,
            duration: entry["duration"].as_i64().unwrap_or(0) as i32,
            upload_date: entry["upload_date"].as_str().map(|s| s.to_string()),
            thumbnail_url: entry["thumbnail"].as_str().map(|s| s.to_string()),
            description: entry["description"].as_str().map(|s| s.to_string()),
        });
    }

    Ok(ScrapeResult { channel_id: uuid::Uuid::nil(), videos, active_lives })
}