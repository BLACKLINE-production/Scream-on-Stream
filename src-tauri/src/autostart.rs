use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Turns "launch on PC startup" on or off. The frontend calls this whenever
/// the user flips the Settings toggle; the OS-level registration (registry
/// entry on Windows, LaunchAgent on macOS, autostart file on Linux) is the
/// only source of truth, so there's nothing to persist on our side.
#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())
    } else {
        autostart.disable().map_err(|e| e.to_string())
    }
}

/// Reads the current OS-level autostart registration. Called once when the
/// Settings page loads so the toggle always reflects reality, even if it
/// was changed outside the app (e.g. removed via Windows Task Manager's
/// Startup tab).
#[tauri::command]
pub fn get_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}
