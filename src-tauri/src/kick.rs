use crate::vote;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio_tungstenite::tungstenite::Message;

static GENERATION: AtomicU64 = AtomicU64::new(0);

const PUSHER_APP_KEY: &str = "32cbd69e4b950bf97679";
const PUSHER_CLUSTER_HOST: &str = "ws-us2.pusher.com";

const CONNECT_TIMEOUT_SECS: u64 = 10;

#[derive(Deserialize)]
struct ChannelLookup {
    chatroom: ChatroomLookup,
}

#[derive(Deserialize)]
struct ChatroomLookup {
    id: u64,
}

async fn fetch_chatroom_id(slug: &str) -> Result<u64, String> {
    let url = format!("https://kick.com/api/v2/channels/{slug}");
    let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Could not reach Kick: {e}"))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(format!("@{slug} doesn't look like a Kick channel. Check the spelling and try again."));
    }
    if !resp.status().is_success() {
        return Err(format!(
            "Kick returned an unexpected response ({}). It may be rate-limiting us — wait a bit and try again.",
            resp.status()
        ));
    }

    let body: ChannelLookup = resp
        .json()
        .await
        .map_err(|_| "Kick's channel lookup response changed shape. This integration may need an update.".to_string())?;

    Ok(body.chatroom.id)
}

#[tauri::command]
pub async fn connect_kick_chat(app: AppHandle, channel: String) -> Result<(), String> {
    let slug = channel.trim().trim_start_matches('@').to_lowercase();
    if slug.is_empty() {
        return Err("Channel name is empty".into());
    }

    let chatroom_id = fetch_chatroom_id(&slug).await?;

    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    tauri::async_runtime::spawn(async move {
        loop {
            if GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            if let Err(e) = run_session(&app, chatroom_id, generation).await {
                eprintln!("Kick chat session for chatroom {chatroom_id} ended: {e}");
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
pub fn disconnect_kick_chat() -> Result<(), String> {
    GENERATION.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

async fn run_session(app: &AppHandle, chatroom_id: u64, generation: u64) -> Result<(), String> {
    let ws_url = format!(
        "wss://{PUSHER_CLUSTER_HOST}/app/{PUSHER_APP_KEY}?protocol=7&client=sos-app&version=1.0&flash=false"
    );

    let (ws_stream, _) = tokio::time::timeout(
        Duration::from_secs(CONNECT_TIMEOUT_SECS),
        tokio_tungstenite::connect_async(ws_url.as_str()),
    )
    .await
    .map_err(|_| "Connection to Kick chat timed out".to_string())?
    .map_err(|e| format!("Could not reach Kick chat: {e}"))?;

    let (mut writer, mut reader) = ws_stream.split();

    let subscribe = serde_json::json!({
        "event": "pusher:subscribe",
        "data": { "auth": "", "channel": format!("chatrooms.{chatroom_id}.v2") }
    });
    writer
        .send(Message::Text(subscribe.to_string().into()))
        .await
        .map_err(|e| e.to_string())?;

    while let Some(msg) = reader.next().await {
        if GENERATION.load(Ordering::SeqCst) != generation {
            return Ok(());
        }

        let msg = msg.map_err(|e| e.to_string())?;
        match msg {
            Message::Text(text) => {
                if let Some(chat_text) = parse_chat_message(&text) {
                    handle_chat_message(app, &chat_text);
                } else if is_ping(&text) {
                    let _ = writer
                        .send(Message::Text(r#"{"event":"pusher:pong","data":{}}"#.into()))
                        .await;
                }
            }
            Message::Ping(payload) => {
                let _ = writer.send(Message::Pong(payload)).await;
            }
            Message::Close(_) => return Ok(()),
            _ => {}
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct PusherEnvelope {
    event: String,
    data: Option<String>,
}

#[derive(Deserialize)]
struct KickChatMessageData {
    content: String,
}

fn is_ping(raw: &str) -> bool {
    matches!(
        serde_json::from_str::<PusherEnvelope>(raw),
        Ok(env) if env.event == "pusher:ping"
    )
}

fn parse_chat_message(raw: &str) -> Option<String> {
    let envelope: PusherEnvelope = serde_json::from_str(raw).ok()?;
    if envelope.event != "App\\Events\\ChatMessageEvent" {
        return None;
    }
    let inner = envelope.data?;
    let payload: KickChatMessageData = serde_json::from_str(&inner).ok()?;
    Some(payload.content)
}

fn handle_chat_message(app: &AppHandle, text: &str) {
    if let Ok(choice) = text.trim().parse::<usize>() {
        if (1..=3).contains(&choice) {
            let state = app.state::<vote::VoteState>();
            let _ = vote::cast_vote(state, choice);
        }
    }
}