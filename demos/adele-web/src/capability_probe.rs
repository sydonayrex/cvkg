//! Runtime browser-capability probe and user-facing fallback for the web demo.
//!
//! `adele-web` renders into a single `<canvas>` and historically had **no**
//! browser-level feature detection: the WGSL shader variants are selected at
//! compile time by `#[cfg(target_arch = "wasm32")]`, which says nothing about
//! whether a given browser actually exposes WebGPU or WebGL2. This module adds
//! a small, honest runtime probe and a perceivable fallback message.
//!
//! Scope and honesty notes:
//!
//! - The probe reports **availability of a JS API surface**, not whether CVKG
//!   can successfully initialise a device. A `true` result means "the API
//!   object exists", not "rendering is proven to work".
//! - `navigator.gpu` is read dynamically through [`js_sys::Reflect`] so no
//!   unstable `web-sys` GPU binding is required.
//! - WebGL2 is probed on a **throwaway** `<canvas>` so the demo's real canvas
//!   context is never claimed or stolen by the probe.
//! - [`BrowserCapabilities::is_webkit_like`] is a **user-agent heuristic**, not
//!   a compatibility guarantee. It is advisory only.
//! - The fallback is a **message**, not a software renderer. Nothing here
//!   switches the demo to a different rendering backend.
//! - No `unsafe`, and no `unwrap()`/`expect()` in any runtime path. The whole
//!   module compiles on the host as a conservative stub so `cargo check`
//!   keeps working without a wasm target.

/// Coarse rendering tier recommended by a capability probe.
///
/// This is a *recommendation* derived from API availability only; it does not
/// select or configure a renderer and is not a compatibility guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendedTier {
    /// `navigator.gpu` was present, so WebGPU is the preferred path.
    WebGpu,
    /// WebGPU was absent but a WebGL2 context could be created.
    WebGl2,
    /// Neither WebGPU nor WebGL2 was detected; the demo cannot render.
    Unsupported,
}

/// Plain-data result of probing the current browser's graphics capabilities.
///
/// Every field is an owned Rust value (no JS handles) so callers can log,
/// serialize or compare the result without touching `wasm-bindgen` types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserCapabilities {
    /// Whether `navigator.gpu` exists (WebGPU API surface present).
    pub webgpu: bool,
    /// Whether a WebGL2 context could be obtained on a throwaway canvas.
    pub webgl2: bool,
    /// Whether the user-agent string looks WebKit/Safari-like.
    ///
    /// This is a heuristic from `navigator.userAgent`, **not** a compatibility
    /// guarantee. Blink-based browsers may also match because their UA still
    /// advertises AppleWebKit.
    pub is_webkit_like: bool,
}

impl BrowserCapabilities {
    /// Returns `true` when at least one supported graphics API was detected.
    pub fn is_supported(&self) -> bool {
        self.webgpu || self.webgl2
    }

    /// Picks the best rendering tier suggested by this probe.
    ///
    /// WebGPU wins over WebGL2 when both are present.
    pub fn recommended_tier(&self) -> RecommendedTier {
        if self.webgpu {
            RecommendedTier::WebGpu
        } else if self.webgl2 {
            RecommendedTier::WebGl2
        } else {
            RecommendedTier::Unsupported
        }
    }

    /// Returns a short human-readable reason when no supported API was found.
    ///
    /// `None` means at least one of WebGPU or WebGL2 was detected.
    pub fn fallback_reason(&self) -> Option<&'static str> {
        if self.is_supported() {
            None
        } else {
            Some("Neither WebGPU nor WebGL2 is available in this browser configuration.")
        }
    }
}

/// Probe the current browser's graphics capabilities without panicking.
///
/// On non-wasm targets this returns a conservative "nothing available" result
/// so the public API is identical everywhere. Any individual probe that fails
/// (missing window/document, denied context, JS exception) contributes `false`
/// for that field rather than aborting the whole detection.
pub fn detect_capabilities() -> BrowserCapabilities {
    #[cfg(target_arch = "wasm32")]
    {
        let window = match web_sys::window() {
            Some(window) => window,
            None => return unsupported_result(),
        };
        let navigator = window.navigator();
        let document = match window.document() {
            Some(document) => document,
            None => return unsupported_result(),
        };

        BrowserCapabilities {
            webgpu: probe_webgpu(&navigator),
            webgl2: probe_webgl2(&document),
            is_webkit_like: probe_webkit_like(&navigator),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        unsupported_result()
    }
}

/// The message shown to users when no supported graphics API was detected.
pub const UNSUPPORTED_MESSAGE: &str =
    "This demo needs WebGPU or WebGL2; your browser/WebKit configuration is unsupported.";

/// Idempotently insert a perceivable fallback notice into the DOM on wasm.
///
/// Creates a `<div role="alert">` carrying [`UNSUPPORTED_MESSAGE`] and appends
/// it to `document.body` (or the `#cvkg-container` element when present). The
/// element is **visible** — it is never `display:none` — and is tagged with
/// `id="cvkg-unsupported-notice"` so repeated calls do not create duplicates.
///
/// Returns `Ok(())` when the notice exists (or was created), and a descriptive
/// `Err(String)` if a required DOM object is unavailable. On non-wasm targets
/// this is a no-op returning `Ok(())`, because there is no browser UI.
pub fn show_unsupported_notice() -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        const NOTICE_ID: &str = "cvkg-unsupported-notice";
        const NOTICE_STYLE: &str = concat!(
            "position:fixed;top:0;left:0;right:0;",
            "box-sizing:border-box;padding:12px 16px;",
            "z-index:2147483647;",
            "background:#3a0a0a;color:#ffd7d7;",
            "border-bottom:1px solid #ff6b6b;",
            "font:14px/1.4 -apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;",
            "text-align:center;"
        );

        let window = web_sys::window().ok_or_else(|| "no browser window".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "no browser document".to_string())?;

        if document.get_element_by_id(NOTICE_ID).is_some() {
            return Ok(());
        }

        let notice = document
            .create_element("div")
            .map_err(|_| "failed to create notice element".to_string())?;
        let _ = notice.set_attribute("id", NOTICE_ID);
        let _ = notice.set_attribute("role", "alert");
        let _ = notice.set_attribute("aria-live", "assertive");
        let _ = notice.set_attribute("style", NOTICE_STYLE);
        notice.set_text_content(Some(UNSUPPORTED_MESSAGE));

        let host = document.get_element_by_id("cvkg-container");
        match host {
            Some(container) => {
                container
                    .append_child(&notice)
                    .map_err(|_| "failed to append notice to #cvkg-container".to_string())?;
            }
            None => {
                let body = document
                    .body()
                    .ok_or_else(|| "no document.body".to_string())?;
                body.append_child(&notice)
                    .map_err(|_| "failed to append notice to document.body".to_string())?;
            }
        }
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(())
    }
}

/// Conservative "nothing detected" result used by the non-wasm stub and by
/// any wasm probe path that cannot obtain a window or document.
fn unsupported_result() -> BrowserCapabilities {
    BrowserCapabilities {
        webgpu: false,
        webgl2: false,
        is_webkit_like: false,
    }
}

/// Check for the WebGPU API surface via dynamic property access.
///
/// Uses `js_sys::Reflect` on `navigator` so this does not depend on an unstable
/// `web-sys` GPU binding. A `null`/`undefined` value, or any JS exception, is
/// treated as "not available".
#[cfg(target_arch = "wasm32")]
fn probe_webgpu(navigator: &web_sys::Navigator) -> bool {
    use wasm_bindgen::{JsCast, JsValue};

    let target: &JsValue = navigator.unchecked_ref();
    let key = JsValue::from_str("gpu");
    match js_sys::Reflect::get(target, &key) {
        Ok(value) => !value.is_null() && !value.is_undefined(),
        Err(_) => false,
    }
}

/// Check whether a WebGL2 context can be created on a throwaway canvas.
///
/// The probe uses a freshly created `<canvas>` so the demo's real canvas is
/// never bound to a context by detection. Any failure yields `false`.
#[cfg(target_arch = "wasm32")]
fn probe_webgl2(document: &web_sys::Document) -> bool {
    use wasm_bindgen::JsCast;

    let element = match document.create_element("canvas") {
        Ok(element) => element,
        Err(_) => return false,
    };
    let canvas = match element.dyn_into::<web_sys::HtmlCanvasElement>() {
        Ok(canvas) => canvas,
        Err(_) => return false,
    };
    match canvas.get_context("webgl2") {
        Ok(context) => context.is_some(),
        Err(_) => false,
    }
}

/// Heuristically detect a WebKit/Safari-like user agent.
///
/// This is advisory only and explicitly **not** a compatibility guarantee. It
/// looks for WebKit markers in `navigator.userAgent`; Blink-based browsers may
/// also match, so callers must not use it to make correctness decisions.
#[cfg(target_arch = "wasm32")]
fn probe_webkit_like(navigator: &web_sys::Navigator) -> bool {
    let user_agent = navigator
        .user_agent()
        .unwrap_or_default()
        .to_ascii_lowercase();
    user_agent.contains("webkit") || user_agent.contains("safari")
}
