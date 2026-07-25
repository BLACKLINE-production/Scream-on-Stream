use crate::vote;
use piratetok_live_rs::{errors::TikTokLiveError, structs::TikTokLiveEvent, TikTokLive, TikTokLiveStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};

static GENERATION: AtomicU64 = AtomicU64::new(0);

fn manual_ttwid_override() -> Option<String> {
    std::env::var("SOS_TIKTOK_TTWID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

async fn resolve_ttwid(app: &AppHandle, force_refresh: bool) -> Option<String> {
    if let Some(manual) = manual_ttwid_override() {
        return Some(manual);
    }
    let result = if force_refresh {
        crate::ttwid::refresh_ttwid(app).await
    } else {
        crate::ttwid::get_ttwid(app).await
    };
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("Automatic ttwid capture failed: {e}");
            None
        }
    }
}

#[tauri::command]
pub async fn connect_tiktok_chat(app: AppHandle, username: String) -> Result<(), String> {
    let username = username.trim().trim_start_matches('@').to_lowercase();
    if username.is_empty() {
        return Err("Username is empty".into());
    }

    let mut builder = TikTokLive::builder(&username);
    if let Some(ttwid) = resolve_ttwid(&app, false).await {
        builder = builder.ttwid(ttwid);
    }
    let mut stream = builder.connect().await;

    if stream.is_err() && manual_ttwid_override().is_none() {
        let mut retry_builder = TikTokLive::builder(&username);
        if let Some(ttwid) = resolve_ttwid(&app, true).await {
            retry_builder = retry_builder.ttwid(ttwid);
        }
        stream = retry_builder.connect().await;
    }

    let stream = stream.map_err(|e| describe_connect_error(&username, e))?;

    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    tauri::async_runtime::spawn(async move {
        let mut pending = Some(stream);
        loop {
            if GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }

            let stream = match pending.take() {
                Some(s) => s,
                None => {
                    let mut builder = TikTokLive::builder(&username);
                    if let Some(ttwid) = resolve_ttwid(&app, false).await {
                        builder = builder.ttwid(ttwid);
                    }
                    match builder.connect().await {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("PirateTok reconnect failed for @{username}: {e}");
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    }
                }
            };

            read_events(&app, stream, generation).await;

            if GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub fn disconnect_tiktok_chat() -> Result<(), String> {
    GENERATION.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

async fn read_events(app: &AppHandle, mut stream: TikTokLiveStream, generation: u64) {
    while let Some(event) = stream.next_event().await {
        if GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }

        match event {
            TikTokLiveEvent::Chat(msg) => handle_chat_message(app, &msg.comment),
            TikTokLiveEvent::Disconnected => {
                eprintln!("TikTok LIVE session ended");
                return;
            }
            _ => {}
        }
    }
    eprintln!("TikTok LIVE stream closed");
}

fn handle_chat_message(app: &AppHandle, text: &str) {
    if let Ok(choice) = text.trim().parse::<usize>() {
        if (1..=3).contains(&choice) {
            let state = app.state::<vote::VoteState>();
            let _ = vote::cast_vote(state, choice);
        }
    }
}

fn describe_connect_error(username: &str, err: TikTokLiveError) -> String {
    match err {
        TikTokLiveError::HostNotOnline(_) => format!(
            "not_live:@{username} is not currently LIVE on TikTok. Start the broadcast, then connect."
        ),
        TikTokLiveError::UserNotFound(_) => {
            format!("@{username} doesn't look like a TikTok account. Check the spelling and try again.")
        }
        TikTokLiveError::DeviceBlocked => {
            "TikTok temporarily blocked this connection attempt. Wait a bit and try again.".to_string()
        }
        other => format!("Could not connect to TikTok LIVE: {other}"),
    }
}