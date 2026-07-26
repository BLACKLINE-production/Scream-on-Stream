mod autostart;
mod hotkey;
mod media;
mod scare;
mod tiktok;
mod ttwid;
mod twitch;
mod vote;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::ShortcutState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = scare::panic_button(app.clone());
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(scare::PendingScare::default())
        .manage(scare::AutoScareState::default())
        .manage(scare::MasterVolume::default())
        .manage(scare::ScareWidgetState::default())
        .manage(vote::VoteState::default())
        .manage(vote::WidgetServer::default())
        .manage(hotkey::PanicHotkeyState::default())
        .manage(ttwid::TtwidCache::default())
        .invoke_handler(tauri::generate_handler![
            media::list_screamers,
            media::add_screamer_files,
            media::rename_screamer,
            media::delete_screamer,
            scare::trigger_scare,
            scare::take_scare_media,
            scare::force_close_scare,
            scare::panic_button,
            scare::start_random_scares,
            scare::stop_random_scares,
            scare::set_master_volume,
            scare::get_master_volume,
            vote::start_vote_round,
            vote::cast_vote,
            vote::finish_vote_round,
            vote::cancel_vote_round,
            vote::get_vote_state,
            vote::ensure_widget_server,
            twitch::connect_twitch_chat,
            twitch::disconnect_twitch_chat,
            tiktok::connect_tiktok_chat,
            tiktok::disconnect_tiktok_chat,
            hotkey::set_panic_hotkey,
            autostart::set_autostart,
            autostart::get_autostart_enabled,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            if let Err(e) = media::ensure_media_dirs(&app_handle) {
                eprintln!("Failed to prepare Media folder: {e}");
            }
            if let Err(e) = hotkey::register_default(&app_handle) {
                eprintln!("Failed to register the panic button hotkey: {e}");
            }
            if let Err(e) = scare::spawn_overlay_window(&app_handle) {
                eprintln!("Failed to create scare overlay window: {e}");
            }

            let show_i = MenuItem::with_id(app, "show", "Open", true, None::<&str>)?;
            let stop_scare_i = MenuItem::with_id(app, "stop_scare", "Force stop scare", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &stop_scare_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "stop_scare" => {
                        let _ = scare::force_close_scare(app.clone());
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    window.hide().unwrap();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}