use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};

const CAPTURE_WINDOW_LABEL: &str = "tiktok-ttwid-capture";
const TIKTOK_HOME: &str = "https://www.tiktok.com/";

const CHALLENGE_WAIT: Duration = Duration::from_secs(4);

const CACHE_TTL: Duration = Duration::from_secs(60 * 30);

#[derive(Default)]
struct CachedTtwid {
    value: Option<(String, Instant)>,
}

pub struct TtwidCache(Mutex<CachedTtwid>);

impl Default for TtwidCache {
    fn default() -> Self {
        TtwidCache(Mutex::new(CachedTtwid::default()))
    }
}

pub async fn get_ttwid(app: &AppHandle) -> Result<String, String> {
    if let Some(cached) = read_cache(app) {
        return Ok(cached);
    }
    let fresh = capture_ttwid(app).await?;
    write_cache(app, &fresh);
    Ok(fresh)
}

pub async fn refresh_ttwid(app: &AppHandle) -> Result<String, String> {
    let fresh = capture_ttwid(app).await?;
    write_cache(app, &fresh);
    Ok(fresh)
}

fn read_cache(app: &AppHandle) -> Option<String> {
    let state = app.try_state::<TtwidCache>()?;
    let guard = state.0.lock().ok()?;
    let (value, captured_at) = guard.value.as_ref()?;
    if captured_at.elapsed() < CACHE_TTL {
        Some(value.clone())
    } else {
        None
    }
}

fn write_cache(app: &AppHandle, value: &str) {
    if let Some(state) = app.try_state::<TtwidCache>() {
        if let Ok(mut guard) = state.0.lock() {
            guard.value = Some((value.to_string(), Instant::now()));
        }
    }
}

async fn capture_ttwid(app: &AppHandle) -> Result<String, String> {
    if let Some(existing) = app.get_webview_window(CAPTURE_WINDOW_LABEL) {
        let _ = existing.close();
    }

    let url: Url = TIKTOK_HOME.parse().map_err(|e| format!("Bad URL: {e}"))?;

    let window = WebviewWindowBuilder::new(app, CAPTURE_WINDOW_LABEL, WebviewUrl::External(url.clone()))
        .title("")
        .visible(false)
        .skip_taskbar(true)
        .decorations(false)
        .build()
        .map_err(|e| format!("Could not open capture webview: {e}"))?;

    tokio::time::sleep(CHALLENGE_WAIT).await;

    let window_for_read = window.clone();
    let cookies = tauri::async_runtime::spawn_blocking(move || {
        window_for_read.cookies_for_url(url)
    })
    .await
    .map_err(|e| format!("Cookie read task panicked: {e}"))?
    .map_err(|e| format!("Could not read cookies from capture webview: {e}"))?;

    let _ = window.close();

    cookies
        .into_iter()
        .find(|c| c.name() == "ttwid")
        .map(|c| c.value().to_string())
        .ok_or_else(|| {
            "TikTok didn't hand back a ttwid cookie yet (the challenge page may need a bit \
             longer, or TikTok changed something). Falling back to whatever's set in \
             SOS_TIKTOK_TTWID, if anything."
                .to_string()
        })
}
