//! Automatic `ttwid` capture.
//!
//! `piratetok-live-rs`'s own anonymous `ttwid` fetch goes through a plain
//! HTTP client, which TikTok's Slardar WAF blocks (it returns a JS
//! challenge page instead of a Set-Cookie). A real browser engine passes
//! that challenge trivially because it actually executes the JS — and we
//! already ship one: the OS webview that Tauri uses for every window.
//!
//! So instead of asking the user to open DevTools and copy a cookie by
//! hand, we open a fully hidden webview pointed at tiktok.com, let it load
//! for real, then read the `ttwid` cookie back out of the webview's own
//! cookie store via `WebviewWindow::cookies_for_url` (this reads
//! HttpOnly/Secure cookies too, unlike `document.cookie`). Works the same
//! way on Windows and macOS since it's Tauri's own cross-platform API —
//! no `webview2-com`/`WKHTTPCookieStore` plumbing needed.
//!
//! This is deliberately NOT trying to reverse-engineer or forge the
//! ttwid/msToken generation algorithm the way some scraping tools do —
//! that approach breaks every time TikTok tweaks the WAF. Driving a real
//! browser engine is slower per-attempt but far more durable.

use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};

const CAPTURE_WINDOW_LABEL: &str = "tiktok-ttwid-capture";
const TIKTOK_HOME: &str = "https://www.tiktok.com/";

/// How long we let the hidden webview sit on tiktok.com before reading
/// cookies back. The Slardar challenge is JS-driven and usually resolves
/// within a second or two of the page loading; this is generous padding.
const CHALLENGE_WAIT: Duration = Duration::from_secs(4);

/// Once captured, reuse the cookie for this long before bothering to
/// recapture — avoids popping the hidden webview open on every single
/// reconnect. If TikTok rejects the cached value earlier than this,
/// `tiktok.rs` should just call `capture_ttwid` again directly.
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

/// Returns a `ttwid` value, using the cached one if it's still fresh,
/// otherwise capturing a new one via the hidden webview.
pub async fn get_ttwid(app: &AppHandle) -> Result<String, String> {
    if let Some(cached) = read_cache(app) {
        return Ok(cached);
    }
    let fresh = capture_ttwid(app).await?;
    write_cache(app, &fresh);
    Ok(fresh)
}

/// Forces a fresh capture, bypassing the cache. Call this if a connect
/// attempt fails and you suspect the cached cookie is stale/invalid.
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
    // Clean up any leftover window from a previous (e.g. crashed/aborted) attempt.
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

    // Let the page (and its JS challenge) actually run. `cookies_for_url`
    // is a blocking call under the hood, so run it off the async task via
    // spawn_blocking — this also sidesteps the documented Windows deadlock
    // that can happen when reading cookies synchronously from certain
    // contexts.
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
