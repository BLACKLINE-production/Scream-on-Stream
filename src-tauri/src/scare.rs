use crate::media::{self, media_root};
use crate::vote;
use rand::Rng;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::Notify;

pub const PANIC_COOLDOWN_SECS: u64 = 5;

#[derive(Serialize, Clone)]
pub struct ScareMedia {
    pub path: String,
    pub kind: String,
    pub volume: f32,
}

pub struct PendingScare(pub Mutex<Option<ScareMedia>>);

impl Default for PendingScare {
    fn default() -> Self {
        PendingScare(Mutex::new(None))
    }
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ScareWidgetEvent {
    Play {
        url: String,
        kind: String,
        volume: f32,
    },
    Stop,
}

#[derive(Serialize, Clone)]
pub struct ScareWidgetPayload {
    pub seq: u64,
    pub event: Option<ScareWidgetEvent>,
}

struct ScareWidgetInner {
    seq: u64,
    event: Option<ScareWidgetEvent>,
}

#[derive(Clone)]
pub struct ScareWidgetState(std::sync::Arc<Mutex<ScareWidgetInner>>);

impl Default for ScareWidgetState {
    fn default() -> Self {
        ScareWidgetState(std::sync::Arc::new(Mutex::new(ScareWidgetInner {
            seq: 0,
            event: None,
        })))
    }
}

impl ScareWidgetState {
    fn push(&self, event: ScareWidgetEvent) {
        if let Ok(mut inner) = self.0.lock() {
            inner.seq += 1;
            inner.event = Some(event);
        }
    }

    pub fn snapshot(&self) -> ScareWidgetPayload {
        match self.0.lock() {
            Ok(inner) => ScareWidgetPayload {
                seq: inner.seq,
                event: inner.event.clone(),
            },
            Err(_) => ScareWidgetPayload { seq: 0, event: None },
        }
    }
}

fn media_widget_url(id: &str) -> String {
    let mut parts = id.splitn(2, '/');
    let folder = parts.next().unwrap_or("");
    let filename = parts.next().unwrap_or("");
    format!("/media/{folder}/{}", percent_encode(filename))
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

pub struct MasterVolume(pub Mutex<f32>);

impl Default for MasterVolume {
    fn default() -> Self {
        MasterVolume(Mutex::new(1.0))
    }
}

#[tauri::command]
pub fn set_master_volume(state: tauri::State<MasterVolume>, volume: f32) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = volume.clamp(0.0, 1.0);
    Ok(())
}

#[tauri::command]
pub fn get_master_volume(state: tauri::State<MasterVolume>) -> Result<f32, String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;
    Ok(*guard)
}

pub fn spawn_overlay_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window("scare").is_some() {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, "scare", WebviewUrl::App("overlay.html".into()))
        .title("SoS Scare Overlay")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(true)
        .build()
        .map_err(|e| e.to_string())?;

    if let Ok(Some(monitor)) = window.primary_monitor() {
        let _ = window.set_size(*monitor.size());
        let _ = window.set_position(*monitor.position());
    }

    let _ = window.set_ignore_cursor_events(true);
    Ok(())
}

async fn fire_scare(app: &AppHandle, id: &str) -> Result<(), String> {
    let root = media_root(app)?;
    let path = root.join(id);
    if !path.exists() {
        return Err("File not found".into());
    }
    let kind = if id.starts_with("Videos") { "video" } else { "audio" };
    let volume = {
        let vol_state = app.state::<MasterVolume>();
        let guard = vol_state.0.lock().map_err(|e| e.to_string())?;
        *guard
    };

    let media = ScareMedia {
        path: path.to_string_lossy().to_string(),
        kind: kind.to_string(),
        volume,
    };

    {
        let state = app.state::<PendingScare>();
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        *guard = Some(media.clone());
    }

    app.emit_to("scare", "scare://play", &media)
        .map_err(|e| e.to_string())?;

    app.state::<ScareWidgetState>().push(ScareWidgetEvent::Play {
        url: media_widget_url(id),
        kind: kind.to_string(),
        volume,
    });

    Ok(())
}

#[tauri::command]
pub async fn trigger_scare(app: AppHandle, id: String) -> Result<(), String> {
    fire_scare(&app, &id).await
}

#[derive(Default)]
pub struct AutoScareInner {
    generation: u64,
}

pub struct AutoScareState {
    inner: Mutex<AutoScareInner>,
    panic_notify: Notify,
}

impl Default for AutoScareState {
    fn default() -> Self {
        AutoScareState {
            inner: Mutex::new(AutoScareInner::default()),
            panic_notify: Notify::new(),
        }
    }
}

impl AutoScareState {
    pub fn signal_panic(&self) {
        self.panic_notify.notify_waiters();
    }
}

#[tauri::command]
pub async fn start_random_scares(
    app: AppHandle,
    state: tauri::State<'_, AutoScareState>,
    min_minutes: u64,
    max_minutes: u64,
    chat_vote: bool,
    vote_seconds: u64,
) -> Result<(), String> {
    if min_minutes == 0 || max_minutes == 0 || max_minutes < min_minutes {
        return Err("Invalid interval range".into());
    }

    let generation = {
        let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
        inner.generation += 1;
        inner.generation
    };

    let min_secs = min_minutes * 60;
    let max_secs = max_minutes * 60;
    let vote_secs = vote_seconds.max(5);

    tauri::async_runtime::spawn(async move {
        loop {
            if !is_current_generation(&app, generation) {
                return;
            }

            let mut winner_id: Option<String> = None;
            let mut panicked = false;

            if chat_vote {
                let started = vote::start_vote_round(app.clone(), app.state::<vote::VoteState>());
                match started {
                    Ok(candidates) if !candidates.is_empty() => {
                        let auto_state = app.state::<AutoScareState>();
                        tokio::select! {
                            _ = tokio::time::sleep(Duration::from_secs(vote_secs)) => {}
                            _ = auto_state.panic_notify.notified() => { panicked = true; }
                        }

                        if !is_current_generation(&app, generation) {
                            let _ = vote::cancel_vote_round(app.state::<vote::VoteState>());
                            return;
                        }

                        if panicked {
                            let _ = vote::cancel_vote_round(app.state::<vote::VoteState>());
                        } else {
                            winner_id = vote::resolve_vote_round(app.state::<vote::VoteState>())
                                .ok()
                                .flatten();
                        }
                    }
                    _ => {

                    }
                }
            }

            if !panicked {
                let wait_secs = if max_secs > min_secs {
                    rand::thread_rng().gen_range(min_secs..=max_secs)
                } else {
                    min_secs
                };

                let auto_state = app.state::<AutoScareState>();
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(wait_secs)) => {}
                    _ = auto_state.panic_notify.notified() => { panicked = true; }
                }

                if !is_current_generation(&app, generation) {
                    return;
                }
            }

            if panicked {
                tokio::time::sleep(Duration::from_secs(PANIC_COOLDOWN_SECS)).await;
                if !is_current_generation(&app, generation) {
                    return;
                }
                continue;
            }

            if let Some(id) = winner_id {
                let _ = fire_scare(&app, &id).await;
            } else if let Ok(list) = media::list_screamers(app.clone()) {
                if !list.is_empty() {
                    let idx = rand::thread_rng().gen_range(0..list.len());
                    let id = list[idx].id.clone();
                    let _ = fire_scare(&app, &id).await;
                }
            }
        }
    });

    Ok(())
}

fn is_current_generation(app: &AppHandle, generation: u64) -> bool {
    let auto_state = app.state::<AutoScareState>();
    match auto_state.inner.lock() {
        Ok(guard) => guard.generation == generation,
        Err(_) => false,
    }
}

#[tauri::command]
pub fn stop_random_scares(state: tauri::State<AutoScareState>) -> Result<(), String> {
    let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
    inner.generation += 1;
    Ok(())
}

#[tauri::command]
pub fn take_scare_media(state: tauri::State<PendingScare>) -> Result<Option<ScareMedia>, String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    Ok(guard.take())
}

#[tauri::command]
pub fn force_close_scare(app: AppHandle) -> Result<(), String> {
    {
        let pending = app.state::<PendingScare>();
        let mut guard = pending.0.lock().map_err(|e| e.to_string())?;
        *guard = None;
    }
    let _ = app.emit_to("scare", "scare://stop", ());

    app.state::<ScareWidgetState>().push(ScareWidgetEvent::Stop);

    Ok(())
}

#[tauri::command]
pub fn panic_button(app: AppHandle) -> Result<(), String> {
    force_close_scare(app.clone())?;

    {
        let pending = app.state::<PendingScare>();
        let mut guard = pending.0.lock().map_err(|e| e.to_string())?;
        *guard = None;
    }

    app.state::<AutoScareState>().signal_panic();
    Ok(())
}