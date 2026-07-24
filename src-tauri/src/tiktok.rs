use crate::vote;
use futures_util::StreamExt;
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

static GENERATION: AtomicU64 = AtomicU64::new(0);

type TikTokWs = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Deserialize)]
struct TikTokEventEnvelope {
    event: String,
    #[serde(default)]
    data: TikTokEventData,
}

#[derive(Deserialize, Default)]
struct TikTokEventData {
    #[serde(default)]
    comment: String,
}

#[tauri::command]
pub async fn connect_tiktok_chat(app: AppHandle, username: String, api_key: String) -> Result<(), String> {
    let username = username.trim().trim_start_matches('@').to_lowercase();
    let api_key = api_key.trim().to_string();
    if username.is_empty() {
        return Err("Username is empty".into());
    }
    if api_key.is_empty() {
        return Err("API key is empty".into());
    }

    let url = build_url(&username, &api_key);

    let (ws_stream, _) = connect_async(&url)
        .await
        .map_err(|e| format!("Could not connect to TikTok LIVE: {e}"))?;

    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    tauri::async_runtime::spawn(async move {
        let mut pending: Option<TikTokWs> = Some(ws_stream);
        loop {
            if GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }

            let stream = match pending.take() {
                Some(s) => s,
                None => match connect_async(&url).await {
                    Ok((s, _)) => s,
                    Err(e) => {
                        eprintln!("TikTok LIVE reconnect failed: {e}");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                },
            };

            if let Err(e) = read_events(&app, stream, generation).await {
                eprintln!("TikTok LIVE session ended: {e}");
            }

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

fn build_url(username: &str, api_key: &str) -> String {
    format!(
        "wss://api.tik.tools?uniqueId={}&apiKey={}",
        urlencoding::encode(username),
        urlencoding::encode(api_key)
    )
}

async fn read_events(app: &AppHandle, mut stream: TikTokWs, generation: u64) -> Result<(), String> {
    while let Some(msg) = stream.next().await {
        if GENERATION.load(Ordering::SeqCst) != generation {
            return Ok(());
        }
        let msg = msg.map_err(|e| e.to_string())?;
        let Message::Text(text) = msg else { continue };

        if let Ok(envelope) = serde_json::from_str::<TikTokEventEnvelope>(&text) {
            if envelope.event == "chat" {
                handle_chat_message(app, &envelope.data.comment);
            }
        }
    }
    Ok(())
}

fn handle_chat_message(app: &AppHandle, text: &str) {
    if let Ok(choice) = text.trim().parse::<usize>() {
        if (1..=3).contains(&choice) {
            let state = app.state::<vote::VoteState>();
            let _ = vote::cast_vote(state, choice);
        }
    }
}