//! A web page by its address (CAP-037): loaded by XcreenShot itself in a private
//! (InPrivate) web view that the user never sees, and taken whole, from top to bottom,
//! through the web view's DevTools protocol. Nothing of the user's browser is used and
//! nothing is kept: the private session goes when the view closes.
//!
//! The view sits off screen while it loads. The page is laid out at the width asked,
//! waits a moment, optionally scrolls to the bottom so late content loads, and is
//! taken in one image up to the largest size (CAP-036).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// Longest wait for the page (CAP-037).
const LOAD_LIMIT: Duration = Duration::from_secs(30);
/// Height of the view while the page is laid out (CSS pixels).
const VIEW_HEIGHT: u32 = 900;
/// Largest side of the image (CAP-036).
const MOST: f64 = 60_000.0;

static NEXT: AtomicU64 = AtomicU64::new(1);
/// Cancel of the page being loaded.
static CANCEL: AtomicBool = AtomicBool::new(false);

/// Scrolls to the bottom in steps (so late images and blocks load) and back.
const LAZY: &str = r#"(async () => {
  const pause = (ms) => new Promise((r) => setTimeout(r, ms));
  const most = 60000;
  let y = 0;
  for (let i = 0; i < 200; i++) {
    const h = Math.max(document.documentElement.scrollHeight, document.body ? document.body.scrollHeight : 0);
    if (y >= h || y >= most) break;
    y += Math.max(200, Math.floor(innerHeight * 0.8));
    scrollTo(0, y);
    await pause(120);
  }
  scrollTo(0, 0);
  await pause(300);
  return true;
})()"#;

/// The page's title, final address, answer code and height.
const INFO: &str = r#"JSON.stringify({
  title: document.title,
  url: location.href,
  status: (performance.getEntriesByType("navigation")[0] || {}).responseStatus || 0,
  height: Math.max(document.documentElement.scrollHeight, document.body ? document.body.scrollHeight : 0)
})"#;

pub fn cancel() {
    CANCEL.store(true, Ordering::Relaxed);
}

/// Calls one DevTools method on the view and waits for its answer.
fn cdp(app: &AppHandle, label: &str, method: &str, params: Value, wait: Duration) -> Result<Value, String> {
    let w = app.get_webview_window(label).ok_or("the page view is gone")?;
    let (tx, rx) = mpsc::channel::<Result<String, String>>();
    let method = method.to_string();
    let params = params.to_string();
    #[cfg(windows)]
    {
        w.with_webview(move |webview| {
            use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
            use windows_core::HSTRING;
            let tx2 = tx.clone();
            let handler =
                CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result, json| {
                    let _ = tx2.send(result.map(|_| json).map_err(|e| e.message()));
                    Ok(())
                }));
            // SAFETY: COM calls on the view's own controller, on its thread.
            unsafe {
                match webview.controller().CoreWebView2() {
                    Ok(core) => {
                        if let Err(e) = core.CallDevToolsProtocolMethod(
                            &HSTRING::from(method),
                            &HSTRING::from(params),
                            &handler,
                        ) {
                            let _ = tx.send(Err(e.message()));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.message()));
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    {
        let _ = (w, method, params, tx);
        return Err("not available on this system yet".into());
    }
    #[allow(unreachable_code)]
    {
        let json = rx
            .recv_timeout(wait)
            .map_err(|_| "the page did not answer in time".to_string())??;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }
}

/// Evaluates `expression` in the page and returns its value.
fn eval(app: &AppHandle, label: &str, expression: &str, wait: Duration) -> Result<Value, String> {
    let r = cdp(
        app,
        label,
        "Runtime.evaluate",
        json!({ "expression": expression, "awaitPromise": true, "returnByValue": true }),
        wait,
    )?;
    if let Some(e) = r.get("exceptionDetails") {
        return Err(e["text"].as_str().unwrap_or("script error").to_string());
    }
    Ok(r["result"]["value"].clone())
}

/// Loads `url` and takes the whole page: `{png (base64), url, title, status, cut}`.
/// `p`: `url`, `width` (CSS px), `scale` (1 or 2), `wait` (seconds), `lazy`.
pub fn capture(app: &AppHandle, p: &Value) -> Result<Value, String> {
    CANCEL.store(false, Ordering::Relaxed);
    let url: tauri::Url = p["url"]
        .as_str()
        .unwrap_or_default()
        .parse()
        .map_err(|e| format!("{e}"))?;
    let width = p["width"].as_u64().unwrap_or(1280).clamp(320, 3840) as u32;
    let scale = p["scale"].as_f64().unwrap_or(1.0).clamp(1.0, 2.0);
    let settle = Duration::from_millis((p["wait"].as_f64().unwrap_or(1.0).clamp(0.0, 10.0) * 1000.0) as u64);
    let lazy = p["lazy"].as_bool().unwrap_or(true);
    let label = format!("webcap-{}", NEXT.fetch_add(1, Ordering::Relaxed));
    let started = Instant::now();

    // The view: private, off screen, never in the taskbar.
    let (loaded_tx, loaded_rx) = mpsc::channel::<()>();
    let (built_tx, built_rx) = mpsc::channel::<Result<(), String>>();
    {
        let app2 = app.clone();
        let label = label.clone();
        app.run_on_main_thread(move || {
            let built = WebviewWindowBuilder::new(&app2, &label, WebviewUrl::External(url))
                .title("XcreenShot")
                .incognito(true)
                .decorations(false)
                .skip_taskbar(true)
                .focused(false)
                .position(-32000.0, -32000.0)
                .inner_size(width as f64, VIEW_HEIGHT as f64)
                .on_page_load(move |_w, payload| {
                    if matches!(payload.event(), PageLoadEvent::Finished) {
                        let _ = loaded_tx.send(());
                    }
                })
                .build()
                .map(|_| ())
                .map_err(|e| e.to_string());
            let _ = built_tx.send(built);
        })
        .map_err(|e| e.to_string())?;
    }
    built_rx
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "the page view did not open".to_string())??;

    let result = (|| {
        // Loaded, cancelled, or out of time.
        loop {
            if CANCEL.load(Ordering::Relaxed) {
                return Err("cancelled".to_string());
            }
            match loaded_rx.recv_timeout(Duration::from_millis(200)) {
                Ok(()) => break,
                Err(mpsc::RecvTimeoutError::Timeout) if started.elapsed() < LOAD_LIMIT => {}
                Err(_) => return Err("timeout".to_string()),
            }
        }
        let wait = LOAD_LIMIT
            .saturating_sub(started.elapsed())
            .max(Duration::from_secs(5));
        cdp(
            app,
            &label,
            "Emulation.setDeviceMetricsOverride",
            json!({ "width": width, "height": VIEW_HEIGHT, "deviceScaleFactor": scale, "mobile": width < 768 }),
            wait,
        )?;
        std::thread::sleep(settle);
        if CANCEL.load(Ordering::Relaxed) {
            return Err("cancelled".to_string());
        }
        if lazy {
            let _ = eval(app, &label, LAZY, wait);
        }
        let info: Value = eval(app, &label, INFO, wait)?
            .as_str()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        let final_url = info["url"].as_str().unwrap_or_default().to_string();
        // A page the view could not open shows the engine's own error page.
        if final_url.starts_with("chrome-error:") || final_url.starts_with("about:") {
            return Err("unreachable".to_string());
        }
        let height = info["height"].as_f64().unwrap_or(VIEW_HEIGHT as f64).max(1.0);
        let most = (MOST / scale).floor();
        let cut = height > most;
        let height = height.min(most);
        let shot = cdp(
            app,
            &label,
            "Page.captureScreenshot",
            json!({
                "format": "png",
                "captureBeyondViewport": true,
                "fromSurface": true,
                "clip": { "x": 0, "y": 0, "width": width, "height": height, "scale": 1 }
            }),
            Duration::from_secs(60),
        )?;
        Ok(json!({
            "png": shot["data"],
            "url": final_url,
            "title": info["title"],
            "status": info["status"],
            "cut": cut,
            "ms": started.elapsed().as_millis() as u64,
        }))
    })();

    // The view goes, and with it the private session.
    let app2 = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(w) = app2.get_webview_window(&label) {
            let _ = w.destroy();
        }
    });
    result
}
