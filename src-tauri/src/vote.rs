use crate::media;
use crate::scare;
use crate::scare::ScareWidgetState;
use rand::seq::SliceRandom;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

#[derive(Clone, Serialize)]
pub struct VoteCandidate {
    pub id: String,
    pub name: String,
}

#[derive(Default)]
struct VoteSessionInner {
    active: bool,
    candidates: Vec<VoteCandidate>,
    votes: [u32; 3],
}

#[derive(Clone)]
pub struct VoteState(pub Arc<Mutex<VoteSessionInner>>);

impl Default for VoteState {
    fn default() -> Self {
        VoteState(Arc::new(Mutex::new(VoteSessionInner::default())))
    }
}

#[derive(Serialize, Clone)]
pub struct VoteStatePayload {
    pub active: bool,
    pub candidates: Vec<VoteCandidate>,
    pub votes: [u32; 3],
}

fn snapshot(inner: &VoteSessionInner) -> VoteStatePayload {
    VoteStatePayload {
        active: inner.active,
        candidates: inner.candidates.clone(),
        votes: inner.votes,
    }
}

#[tauri::command]
pub fn start_vote_round(
    app: AppHandle,
    vote_state: tauri::State<VoteState>,
) -> Result<Vec<VoteCandidate>, String> {
    let list = media::list_screamers(app)?;
    if list.len() < 3 {
        return Err("Need at least 3 screamers in the library to start a vote".into());
    }

    let mut rng = rand::thread_rng();
    let chosen: Vec<VoteCandidate> = list
        .choose_multiple(&mut rng, 3)
        .map(|f| VoteCandidate {
            id: f.id.clone(),
            name: f.name.clone(),
        })
        .collect();

    let mut inner = vote_state.0.lock().map_err(|e| e.to_string())?;
    inner.active = true;
    inner.candidates = chosen.clone();
    inner.votes = [0, 0, 0];
    Ok(chosen)
}

#[tauri::command]
pub fn cast_vote(vote_state: tauri::State<VoteState>, choice: usize) -> Result<(), String> {
    if choice == 0 || choice > 3 {
        return Err("Choice must be 1, 2 or 3".into());
    }
    let mut inner = vote_state.0.lock().map_err(|e| e.to_string())?;
    if !inner.active {
        return Err("No active vote round".into());
    }
    inner.votes[choice - 1] += 1;
    Ok(())
}

pub(crate) fn resolve_vote_round(vote_state: tauri::State<VoteState>) -> Result<Option<String>, String> {
    let mut inner = vote_state.0.lock().map_err(|e| e.to_string())?;
    if !inner.active || inner.candidates.is_empty() {
        inner.active = false;
        return Ok(None);
    }
    let max_votes = *inner.votes.iter().max().unwrap_or(&0);
    let winners: Vec<usize> = inner
        .votes
        .iter()
        .enumerate()
        .filter(|(_, v)| **v == max_votes)
        .map(|(i, _)| i)
        .collect();
    let idx = *winners.choose(&mut rand::thread_rng()).unwrap_or(&0);
    let id = inner.candidates[idx].id.clone();
    inner.active = false;
    inner.candidates.clear();
    inner.votes = [0, 0, 0];
    Ok(Some(id))
}

#[tauri::command]
pub async fn finish_vote_round(
    app: AppHandle,
    vote_state: tauri::State<'_, VoteState>,
) -> Result<Option<String>, String> {
    let Some(winner_id) = resolve_vote_round(vote_state)? else {
        return Ok(None);
    };

    scare::trigger_scare(app, winner_id.clone()).await?;
    Ok(Some(winner_id))
}

#[tauri::command]
pub fn cancel_vote_round(vote_state: tauri::State<VoteState>) -> Result<(), String> {
    let mut inner = vote_state.0.lock().map_err(|e| e.to_string())?;
    inner.active = false;
    inner.candidates.clear();
    inner.votes = [0, 0, 0];
    Ok(())
}

#[tauri::command]
pub fn get_vote_state(vote_state: tauri::State<VoteState>) -> Result<VoteStatePayload, String> {
    let inner = vote_state.0.lock().map_err(|e| e.to_string())?;
    Ok(snapshot(&inner))
}

pub struct WidgetServer {
    port: Mutex<Option<u16>>,
}

impl Default for WidgetServer {
    fn default() -> Self {
        WidgetServer {
            port: Mutex::new(None),
        }
    }
}

const WIDGET_HTML: &str = include_str!("../widget/vote.html");
const SCARE_WIDGET_HTML: &str = include_str!("../widget/scare.html");
const WIDGET_PORT_RANGE: std::ops::Range<u16> = 47100..47120;

#[tauri::command]
pub fn ensure_widget_server(
    app: AppHandle,
    vote_state: tauri::State<VoteState>,
    scare_state: tauri::State<ScareWidgetState>,
    server: tauri::State<WidgetServer>,
) -> Result<u16, String> {
    {
        let guard = server.port.lock().map_err(|e| e.to_string())?;
        if let Some(port) = *guard {
            return Ok(port);
        }
    }

    let mut bound = None;
    for port in WIDGET_PORT_RANGE {
        if let Ok(s) = tiny_http::Server::http(format!("127.0.0.1:{port}")) {
            bound = Some((s, port));
            break;
        }
    }
    let (http_server, port) =
        bound.ok_or("Could not find a free local port for the widget server")?;

    let vote_state_bg = vote_state.0.clone();
    let scare_state_bg: ScareWidgetState = (*scare_state).clone();
    let app_bg = app.clone();

    std::thread::spawn(move || {
        for request in http_server.incoming_requests() {
            let raw_url = request.url().to_string();
            let path = raw_url.split('?').next().unwrap_or("").to_string();

            let response_result = if path == "/api/vote-state" {
                let payload = {
                    let inner = vote_state_bg.lock().unwrap();
                    snapshot(&inner)
                };
                respond_json(request, &payload)
            } else if path == "/api/scare-state" {
                let payload = scare_state_bg.snapshot();
                respond_json(request, &payload)
            } else if let Some(rest) = path.strip_prefix("/media/") {
                respond_media(request, &app_bg, rest)
            } else if path == "/scare" {
                respond_html(request, SCARE_WIDGET_HTML)
            } else {
                respond_html(request, WIDGET_HTML)
            };
            let _ = response_result;
        }
    });

    let mut guard = server.port.lock().map_err(|e| e.to_string())?;
    *guard = Some(port);
    Ok(port)
}

fn cors_header() -> tiny_http::Header {
    tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap()
}

fn respond_json<T: Serialize>(request: tiny_http::Request, payload: &T) -> std::io::Result<()> {
    let body = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());
    let header_json =
        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
    request.respond(
        tiny_http::Response::from_string(body)
            .with_header(header_json)
            .with_header(cors_header()),
    )
}

fn respond_html(request: tiny_http::Request, html: &str) -> std::io::Result<()> {
    let header_html =
        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
            .unwrap();
    request.respond(
        tiny_http::Response::from_string(html)
            .with_header(header_html)
            .with_header(cors_header()),
    )
}

fn respond_media(request: tiny_http::Request, app: &AppHandle, rest: &str) -> std::io::Result<()> {
    let mut segments = rest.splitn(2, '/');
    let folder = segments.next().unwrap_or("");
    let filename = percent_decode(segments.next().unwrap_or(""));

    match media::read_media_bytes(app, folder, &filename) {
        Ok((bytes, mime)) => {
            let header_type =
                tiny_http::Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap();
            let header_cache =
                tiny_http::Header::from_bytes(&b"Cache-Control"[..], &b"no-store"[..]).unwrap();
            request.respond(
                tiny_http::Response::from_data(bytes)
                    .with_header(header_type)
                    .with_header(header_cache)
                    .with_header(cors_header()),
            )
        }
        Err(_) => {
            request.respond(tiny_http::Response::from_string("Not found").with_status_code(404))
        }
    }
}

fn percent_decode(input: &str) -> String {
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