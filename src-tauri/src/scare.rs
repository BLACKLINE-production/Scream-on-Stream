use crate::media::media_root;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

#[derive(Serialize, Clone)]
pub struct ScareMedia {
    pub path: String,
    pub kind: String, 
}

pub struct PendingScare(pub Mutex<Option<ScareMedia>>);

impl Default for PendingScare {
    fn default() -> Self {
        PendingScare(Mutex::new(None))
    }
}

#[tauri::command]
pub fn trigger_scare(
    app: AppHandle,
    state: tauri::State<PendingScare>,
    id: String,
) -> Result<(), String> {
    let root = media_root(&app)?;
    let path = root.join(&id);
    if !path.exists() {
        return Err("File not found".into());
    }
    let kind = if id.starts_with("Videos") { "video" } else { "audio" };

    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        *guard = Some(ScareMedia {
            path: path.to_string_lossy().to_string(),
            kind: kind.to_string(),
        });
    }

    if let Some(existing) = app.get_webview_window("scare") {
        let _ = existing.close();
    }

    let window = WebviewWindowBuilder::new(&app, "scare", WebviewUrl::App("overlay.html".into()))
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .transparent(true)
        .focused(false)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;

    if kind == "video" {
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let _ = window.set_size(*monitor.size());
            let _ = window.set_position(*monitor.position());
        }
    } else {
        let _ = window.set_size(PhysicalSize::new(1u32, 1u32));
        let _ = window.set_position(PhysicalPosition::new(-10i32, -10i32));
    }

    let _ = window.set_ignore_cursor_events(true);
    window.show().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn take_scare_media(state: tauri::State<PendingScare>) -> Result<Option<ScareMedia>, String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    Ok(guard.take())
}

#[tauri::command]
pub fn force_close_scare(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("scare") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}