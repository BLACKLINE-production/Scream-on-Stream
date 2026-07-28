use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

const DEFAULT_CODE: Code = Code::KeyP;

pub struct PanicHotkeyState(pub Mutex<Option<Shortcut>>);

impl Default for PanicHotkeyState {
    fn default() -> Self {
        PanicHotkeyState(Mutex::new(None))
    }
}

pub fn register_default(app: &AppHandle) -> Result<(), String> {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), DEFAULT_CODE);
    app.global_shortcut()
        .register(shortcut)
        .map_err(|e| e.to_string())?;

    let state = app.state::<PanicHotkeyState>();
    *state.0.lock().map_err(|e| e.to_string())? = Some(shortcut);
    Ok(())
}

#[tauri::command]
pub fn set_panic_hotkey(
    app: AppHandle,
    code: String,
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
) -> Result<(), String> {
    let parsed_code: Code = code
        .parse()
        .map_err(|_| "That key isn't supported.".to_string())?;

    let mut mods = Modifiers::empty();
    if ctrl {
        mods |= Modifiers::CONTROL;
    }
    if alt {
        mods |= Modifiers::ALT;
    }
    if shift {
        mods |= Modifiers::SHIFT;
    }
    if meta {
        mods |= Modifiers::SUPER;
    }

    if mods.is_empty() {
        return Err("Pick at least one modifier key (Ctrl, Alt or Shift).".into());
    }

    let new_shortcut = Shortcut::new(Some(mods), parsed_code);

    let state = app.state::<PanicHotkeyState>();
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(current) = guard.as_ref() {
        if current == &new_shortcut {
            return Ok(());
        }
    }

    app.global_shortcut()
        .register(new_shortcut)
        .map_err(|_| "That combination is already in use by another app on this system.".to_string())?;

    if let Some(old) = guard.take() {
        let _ = app.global_shortcut().unregister(old);
    }

    *guard = Some(new_shortcut);
    Ok(())
}
