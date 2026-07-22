use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{path::BaseDirectory, AppHandle, Manager};

const VIDEO_EXTS: [&str; 5] = ["mp4", "webm", "mov", "mkv", "avi"];
const AUDIO_EXTS: [&str; 5] = ["mp3", "wav", "ogg", "flac", "m4a"];

#[derive(Serialize, Clone)]
pub struct ScreamerFile {
    pub id: String,
    pub name: String,
    pub kind: String,
}

#[derive(Deserialize)]
pub struct NewScreamerFile {
    pub path: String,
    pub name: Option<String>,
}

pub(crate) fn media_root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .document_dir()
        .map_err(|e| e.to_string())
        .map(|p| p.join("SoS App").join("Media"))
}

fn videos_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(media_root(app)?.join("Videos"))
}

fn sounds_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(media_root(app)?.join("Sounds"))
}

pub fn ensure_media_dirs(app: &AppHandle) -> Result<(), String> {
    let videos = videos_dir(app)?;
    let sounds = sounds_dir(app)?;
    fs::create_dir_all(&videos).map_err(|e| e.to_string())?;
    fs::create_dir_all(&sounds).map_err(|e| e.to_string())?;
    seed_presets(app, &videos, &sounds)?;
    Ok(())
}

fn seed_presets(app: &AppHandle, videos: &Path, sounds: &Path) -> Result<(), String> {
    let marker = media_root(app)?.join(".seeded");
    if marker.exists() {
        return Ok(());
    }
    if let Ok(preset_videos) = app.path().resolve("media-presets/videos", BaseDirectory::Resource) {
        copy_dir_contents(&preset_videos, videos);
    }
    if let Ok(preset_sounds) = app.path().resolve("media-presets/sounds", BaseDirectory::Resource) {
        copy_dir_contents(&preset_sounds, sounds);
    }
    fs::write(&marker, b"1").map_err(|e| e.to_string())?;
    Ok(())
}

fn copy_dir_contents(src: &Path, dst: &Path) {
    let Ok(entries) = fs::read_dir(src) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name() {
                let _ = fs::copy(&path, dst.join(name));
            }
        }
    }
}

fn classify(ext: &str) -> Option<&'static str> {
    let ext = ext.to_lowercase();
    if VIDEO_EXTS.contains(&ext.as_str()) {
        Some("video")
    } else if AUDIO_EXTS.contains(&ext.as_str()) {
        Some("audio")
    } else {
        None
    }
}

fn list_dir(dir: &Path, kind: &str, prefix: &str) -> Vec<ScreamerFile> {
    let Ok(entries) = fs::read_dir(dir) else { return vec![] };
    entries
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            Some(ScreamerFile {
                id: format!("{prefix}/{name}"),
                name,
                kind: kind.to_string(),
            })
        })
        .collect()
}

#[tauri::command]
pub fn list_screamers(app: AppHandle) -> Result<Vec<ScreamerFile>, String> {
    ensure_media_dirs(&app)?;
    let mut items = list_dir(&videos_dir(&app)?, "video", "Videos");
    items.extend(list_dir(&sounds_dir(&app)?, "audio", "Sounds"));
    Ok(items)
}

fn build_dest_file_name(custom_name: Option<&str>, original: &OsStr, ext: &str) -> OsString {
    match custom_name.map(|n| n.trim()).filter(|n| !n.is_empty()) {
        Some(custom) => {
            if Path::new(custom).extension().is_some() {
                OsString::from(custom)
            } else {
                OsString::from(format!("{custom}.{ext}"))
            }
        }
        None => original.to_os_string(),
    }
}

#[tauri::command]
pub fn add_screamer_files(app: AppHandle, files: Vec<NewScreamerFile>) -> Result<Vec<ScreamerFile>, String> {
    ensure_media_dirs(&app)?;
    let videos = videos_dir(&app)?;
    let sounds = sounds_dir(&app)?;

    for file in &files {
        let src_path = Path::new(&file.path);
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let Some(kind) = classify(ext) else { continue };
        let Some(original_name) = src_path.file_name() else { continue };

        let dest_dir = if kind == "video" { &videos } else { &sounds };
        let dest_file_name = build_dest_file_name(file.name.as_deref(), original_name, ext);
        let dest_path = unique_path(dest_dir, &dest_file_name);
        fs::copy(src_path, &dest_path).map_err(|e| e.to_string())?;
    }

    list_screamers(app)
}

fn unique_path(dir: &Path, file_name: &OsStr) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    let ext = Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{s}"))
        .unwrap_or_default();
    let mut i = 1;
    loop {
        let candidate = dir.join(format!("{stem} ({i}){ext}"));
        if !candidate.exists() {
            return candidate;
        }
        i += 1;
    }
}

#[tauri::command]
pub fn rename_screamer(app: AppHandle, id: String, new_name: String) -> Result<ScreamerFile, String> {
    let root = media_root(&app)?;
    let old_path = root.join(&id);
    if !old_path.exists() {
        return Err("File not found".into());
    }
    let parent = old_path.parent().ok_or("Incorrect path")?.to_path_buf();
    let new_path = unique_path(&parent, OsStr::new(&new_name));
    fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;

    let folder = if id.starts_with("Videos") { "Videos" } else { "Sounds" };
    let kind = if folder == "Videos" { "video" } else { "audio" };
    let file_name = new_path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
    Ok(ScreamerFile {
        id: format!("{folder}/{file_name}"),
        name: file_name,
        kind: kind.to_string(),
    })
}

#[tauri::command]
pub fn delete_screamer(app: AppHandle, id: String) -> Result<(), String> {
    let path = media_root(&app)?.join(&id);
    fs::remove_file(path).map_err(|e| e.to_string())
}