use crate::media;
use crate::scare;
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
const WIDGET_PORT_RANGE: std::ops::Range<u16> = 47100..47120;

#[tauri::command]
pub fn ensure_widget_server(
    vote_state: tauri::State<VoteState>,
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
    std::thread::spawn(move || {
        for request in http_server.incoming_requests() {
            let url = request.url().to_string();
            let response_result = if url.starts_with("/api/vote-state") {
                let payload = {
                    let inner = vote_state_bg.lock().unwrap();
                    snapshot(&inner)
                };
                let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into());
                let header_json =
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap();
                let header_cors = tiny_http::Header::from_bytes(
                    &b"Access-Control-Allow-Origin"[..],
                    &b"*"[..],
                )
                .unwrap();
                request.respond(
                    tiny_http::Response::from_string(body)
                        .with_header(header_json)
                        .with_header(header_cors),
                )
            } else {
                let header_html = tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"text/html; charset=utf-8"[..],
                )
                .unwrap();
                request.respond(
                    tiny_http::Response::from_string(WIDGET_HTML).with_header(header_html),
                )
            };
            let _ = response_result;
        }
    });

    let mut guard = server.port.lock().map_err(|e| e.to_string())?;
    *guard = Some(port);
    Ok(port)
}