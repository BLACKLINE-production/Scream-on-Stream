use crate::vote;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

static GENERATION: AtomicU64 = AtomicU64::new(0);

const AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const SCOPE: &str = "https://www.googleapis.com/auth/youtube.readonly";
const CALLBACK_PORT_RANGE: std::ops::Range<u16> = 47210..47230;
const LOGIN_TIMEOUT_SECS: u64 = 300;

const CLIENT_ID_PLACEHOLDER: &str = "465954325044-062u866blgdcjvqgemekv5hpvqsb11m2.apps.googleusercontent.com";
const CLIENT_SECRET_PLACEHOLDER: &str = "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";

fn client_id() -> String {
    std::env::var("SOS_YOUTUBE_CLIENT_ID").unwrap_or_else(|_| CLIENT_ID_PLACEHOLDER.to_string())
}

fn client_secret() -> String {
    std::env::var("SOS_YOUTUBE_CLIENT_SECRET").unwrap_or_else(|_| CLIENT_SECRET_PLACEHOLDER.to_string())
}

fn is_configured() -> bool {
    !client_id().trim().is_empty() && !client_secret().trim().is_empty()
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct StoredAuth {
    refresh_token: String,
}

fn auth_file_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("youtube_auth.json"))
}

fn load_refresh_token(app: &AppHandle) -> Option<String> {
    let path = auth_file_path(app).ok()?;
    let data = std::fs::read_to_string(path).ok()?;
    let stored: StoredAuth = serde_json::from_str(&data).ok()?;
    if stored.refresh_token.is_empty() {
        None
    } else {
        Some(stored.refresh_token)
    }
}

fn save_refresh_token(app: &AppHandle, token: &str) -> Result<(), String> {
    let path = auth_file_path(app)?;
    let stored = StoredAuth {
        refresh_token: token.to_string(),
    };
    let data = serde_json::to_string(&stored).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| e.to_string())
}

fn clear_refresh_token(app: &AppHandle) {
    if let Ok(path) = auth_file_path(app) {
        let _ = std::fs::remove_file(path);
    }
}

fn random_url_safe(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn code_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest.as_slice())
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn url_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(value) = u8::from_str_radix(hex, 16) {
                    out.push(value);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(if bytes[i] == b'+' { b' ' } else { bytes[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            Some((url_decode(key), url_decode(value)))
        })
        .collect()
}

fn bind_callback_server() -> Result<(tiny_http::Server, u16), String> {
    for port in CALLBACK_PORT_RANGE {
        if let Ok(s) = tiny_http::Server::http(format!("127.0.0.1:{port}")) {
            return Ok((s, port));
        }
    }
    Err("Could not find a free local port for the Google sign-in callback".into())
}

fn wait_for_callback(server: tiny_http::Server, expected_state: &str) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(LOGIN_TIMEOUT_SECS);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("Timed out waiting for Google sign-in".into());
        }
        let request = match server.recv_timeout(remaining) {
            Ok(Some(r)) => r,
            Ok(None) => continue,
            Err(e) => return Err(e.to_string()),
        };

        let url = request.url().to_string();
        if !url.starts_with("/callback") {
            let _ = request.respond(tiny_http::Response::from_string("Not found").with_status_code(404));
            continue;
        }

        let query = url.splitn(2, '?').nth(1).unwrap_or("");
        let params = parse_query(query);

        let response_html = "<html><body style=\"font-family:sans-serif;text-align:center;padding-top:80px\">\
            <h2>You're connected. You can close this tab.</h2></body></html>";
        let header =
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();
        let _ = request.respond(tiny_http::Response::from_string(response_html).with_header(header));

        if let Some(err) = params.get("error") {
            return Err(format!("Google sign-in was cancelled or denied: {err}"));
        }

        let state = params.get("state").cloned().unwrap_or_default();
        if state != expected_state {
            return Err("Sign-in response failed a security check (state mismatch). Please try again.".into());
        }

        let code = params
            .get("code")
            .cloned()
            .ok_or("No authorization code returned by Google")?;
        return Ok(code);
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    expires_in: u64,
}

async fn exchange_code(code: &str, verifier: &str, redirect_uri: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let cid = client_id();
    let secret = client_secret();
    let params = [
        ("code", code),
        ("client_id", cid.as_str()),
        ("client_secret", secret.as_str()),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", verifier),
    ];
    let resp = client
        .post(TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Google rejected the sign-in: {body}"));
    }
    resp.json().await.map_err(|e| e.to_string())
}

async fn refresh_access_token(refresh_token: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id()),
        ("client_secret", client_secret()),
        ("refresh_token", refresh_token.to_string()),
        ("grant_type", "refresh_token".to_string()),
    ];
    let resp = client
        .post(TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Could not refresh Google sign-in: {body}"));
    }
    resp.json().await.map_err(|e| e.to_string())
}

async fn perform_login(app: &AppHandle) -> Result<String, String> {
    let (server, port) = bind_callback_server()?;
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    let verifier = random_url_safe(64);
    let challenge = code_challenge(&verifier);
    let state = random_url_safe(16);

    let auth_url = format!(
        "{AUTH_ENDPOINT}?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code\
         &scope={scope}&access_type=offline&prompt=consent&code_challenge={challenge}\
         &code_challenge_method=S256&state={state}",
        client_id = percent_encode(&client_id()),
        redirect_uri = percent_encode(&redirect_uri),
        scope = percent_encode(SCOPE),
        challenge = percent_encode(&challenge),
        state = percent_encode(&state),
    );

    app.opener()
        .open_url(auth_url, None::<&str>)
        .map_err(|e| format!("Could not open the browser for sign-in: {e}"))?;

    let expected_state = state.clone();
    let code = tauri::async_runtime::spawn_blocking(move || wait_for_callback(server, &expected_state))
        .await
        .map_err(|e| format!("Sign-in task panicked: {e}"))??;

    let token = exchange_code(&code, &verifier, &redirect_uri).await?;
    let refresh_token = token.refresh_token.ok_or(
        "Google didn't return a refresh token. Try removing SoS's access at \
         https://myaccount.google.com/permissions and reconnecting."
            .to_string(),
    )?;

    save_refresh_token(app, &refresh_token)?;
    Ok(refresh_token)
}

#[derive(Deserialize)]
struct ChannelListResponse {
    items: Vec<ChannelItem>,
}
#[derive(Deserialize)]
struct ChannelItem {
    snippet: ChannelSnippet,
}
#[derive(Deserialize)]
struct ChannelSnippet {
    title: String,
}

async fn fetch_channel_title(access_token: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/youtube/v3/channels")
        .query(&[("part", "snippet"), ("mine", "true")])
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Could not read channel info ({})", resp.status()));
    }
    let body: ChannelListResponse = resp.json().await.map_err(|e| e.to_string())?;
    body.items
        .into_iter()
        .next()
        .map(|i| i.snippet.title)
        .ok_or_else(|| "No YouTube channel found for this Google account".to_string())
}

#[derive(Deserialize)]
struct LiveBroadcastListResponse {
    items: Vec<LiveBroadcastItem>,
}
#[derive(Deserialize)]
struct LiveBroadcastItem {
    snippet: LiveBroadcastSnippet,
}
#[derive(Deserialize)]
struct LiveBroadcastSnippet {
    #[serde(rename = "liveChatId")]
    live_chat_id: Option<String>,
}

async fn find_active_live_chat_id(access_token: &str) -> Result<Option<String>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/youtube/v3/liveBroadcasts")
        .query(&[
            ("part", "snippet"),
            ("broadcastStatus", "active"),
            ("broadcastType", "all"),
            ("mine", "true"),
        ])
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Could not check for an active livestream ({})", resp.status()));
    }

    let body: LiveBroadcastListResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(body.items.into_iter().find_map(|i| i.snippet.live_chat_id))
}

#[derive(Deserialize)]
struct LiveChatMessagesResponse {
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    #[serde(rename = "pollingIntervalMillis")]
    polling_interval_millis: Option<u64>,
    items: Vec<LiveChatMessageItem>,
}
#[derive(Deserialize)]
struct LiveChatMessageItem {
    snippet: LiveChatMessageSnippet,
}
#[derive(Deserialize)]
struct LiveChatMessageSnippet {
    #[serde(rename = "displayMessage")]
    display_message: Option<String>,
}

async fn poll_chat(
    access_token: &str,
    live_chat_id: &str,
    page_token: Option<&str>,
) -> Result<LiveChatMessagesResponse, String> {
    let client = reqwest::Client::new();
    let mut query = vec![
        ("liveChatId", live_chat_id.to_string()),
        ("part", "snippet,authorDetails".to_string()),
    ];
    if let Some(token) = page_token {
        query.push(("pageToken", token.to_string()));
    }
    let resp = client
        .get("https://www.googleapis.com/youtube/v3/liveChat/messages")
        .query(&query)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND || resp.status() == reqwest::StatusCode::FORBIDDEN {
        return Err("live_chat_ended".into());
    }
    if !resp.status().is_success() {
        return Err(format!("YouTube chat polling failed ({})", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

#[derive(Serialize, Clone)]
pub struct YoutubeConnectResult {
    pub channel_title: String,
}

#[tauri::command]
pub async fn connect_youtube_chat(app: AppHandle) -> Result<YoutubeConnectResult, String> {
    if !is_configured() {
        return Err(
            "YouTube integration isn't configured in this build (missing Google OAuth client id/secret)."
                .into(),
        );
    }

    let (refresh_token, access) = match load_refresh_token(&app) {
        Some(token) => match refresh_access_token(&token).await {
            Ok(access) => (token, access),
            Err(_) => {
                clear_refresh_token(&app);
                let new_token = perform_login(&app).await?;
                let access = refresh_access_token(&new_token).await?;
                (new_token, access)
            }
        },
        None => {
            let new_token = perform_login(&app).await?;
            let access = refresh_access_token(&new_token).await?;
            (new_token, access)
        }
    };

    let channel_title = fetch_channel_title(&access.access_token).await?;

    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let app_bg = app.clone();
    tauri::async_runtime::spawn(async move {
        run_chat_loop(app_bg, refresh_token, generation).await;
    });

    Ok(YoutubeConnectResult { channel_title })
}

#[tauri::command]
pub fn disconnect_youtube_chat() -> Result<(), String> {
    GENERATION.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

async fn run_chat_loop(app: AppHandle, refresh_token: String, generation: u64) {
    let mut access_token = String::new();
    let mut access_expires_at = Instant::now();

    loop {
        if GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }

        if access_token.is_empty() || Instant::now() >= access_expires_at {
            match refresh_access_token(&refresh_token).await {
                Ok(tok) => {
                    access_token = tok.access_token;
                    access_expires_at =
                        Instant::now() + Duration::from_secs(tok.expires_in.saturating_sub(60).max(30));
                }
                Err(e) => {
                    eprintln!("YouTube token refresh failed: {e}");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
            }
        }

        let live_chat_id = match find_active_live_chat_id(&access_token).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }
            Err(e) => {
                eprintln!("YouTube live broadcast lookup failed: {e}");
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }
        };

        let mut page_token: Option<String> = None;
        loop {
            if GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            if Instant::now() >= access_expires_at {
                break;
            }

            match poll_chat(&access_token, &live_chat_id, page_token.as_deref()).await {
                Ok(page) => {
                    if page_token.is_some() {
                        for item in &page.items {
                            if let Some(text) = &item.snippet.display_message {
                                handle_chat_message(&app, text);
                            }
                        }
                    }
                    page_token = page.next_page_token.or(page_token);
                    let wait_ms = page.polling_interval_millis.unwrap_or(2000).max(2000);
                    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                }
                Err(e) if e == "live_chat_ended" => break,
                Err(e) => {
                    eprintln!("YouTube chat polling error: {e}");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }
}

fn handle_chat_message(app: &AppHandle, text: &str) {
    if let Ok(choice) = text.trim().parse::<usize>() {
        if (1..=3).contains(&choice) {
            let state = app.state::<vote::VoteState>();
            let _ = vote::cast_vote(state, choice);
        }
    }
}