use crate::vote;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

static GENERATION: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
pub async fn connect_twitch_chat(app: AppHandle, channel: String) -> Result<(), String> {
    let channel = channel.trim().trim_start_matches(|c: char| c == '#' || c == '@').to_lowercase();
    if channel.is_empty() {
        return Err("Channel name is empty".into());
    }

    let probe = TcpStream::connect("irc.chat.twitch.tv:6667")
        .await
        .map_err(|e| format!("Could not reach Twitch chat: {e}"))?;
    drop(probe);

    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    tauri::async_runtime::spawn(async move {
        loop {
            if GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            if let Err(e) = run_session(&app, &channel, generation).await {
                eprintln!("Twitch chat session for #{channel} ended: {e}");
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
pub fn disconnect_twitch_chat() -> Result<(), String> {
    GENERATION.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

async fn run_session(app: &AppHandle, channel: &str, generation: u64) -> Result<(), String> {
    let stream = TcpStream::connect("irc.chat.twitch.tv:6667")
        .await
        .map_err(|e| e.to_string())?;
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let nick = format!("justinfan{}", rand::random::<u32>() % 100000);
    writer
        .write_all(format!("NICK {nick}\r\n").as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    writer
        .write_all(format!("JOIN #{channel}\r\n").as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    loop {
        if GENERATION.load(Ordering::SeqCst) != generation {
            return Ok(());
        }
        let line = match lines.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => return Ok(()),
            Err(e) => return Err(e.to_string()),
        };

        if line.starts_with("PING") {
            let pong = line.replacen("PING", "PONG", 1);
            let _ = writer.write_all(format!("{pong}\r\n").as_bytes()).await;
            continue;
        }

        if let Some(text) = parse_privmsg(&line) {
            handle_chat_message(app, &text);
        }
    }
}

fn parse_privmsg(line: &str) -> Option<String> {
    let after_cmd = line.split_once(" PRIVMSG ")?.1;
    let message = after_cmd.split_once(" :")?.1;
    Some(message.trim().to_string())
}

fn handle_chat_message(app: &AppHandle, text: &str) {
    if let Ok(choice) = text.trim().parse::<usize>() {
        if (1..=3).contains(&choice) {
            let state = app.state::<vote::VoteState>();
            let _ = vote::cast_vote(state, choice);
        }
    }
}