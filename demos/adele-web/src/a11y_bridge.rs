//! Offscreen DOM accessibility / text bridge for the canvas-only `adele-web` demo.
//!
//! **P1-3 spike — proof of concept only, not production code.** CVKG renders the
//! whole demo into a single `<canvas>`, which means assistive technology, IME
//! composition, keyboard focus and the clipboard have nothing in the DOM to
//! attach to. This module bridges a *small* subset of that interaction surface
//! onto real, offscreen DOM nodes. It deliberately does **not** attempt a
//! complete accessibility tree, layout-based hit testing, or a virtualised
//! caret. See `README.md` ("Accessibility / text bridge spike (P1-3)") for the
//! honest list of what works and what does not.
//!
//! Design constraints honoured here:
//!
//! - no `unsafe` code anywhere;
//! - no `panic!`/`unwrap()`/`expect()` in runtime paths — every fallible DOM
//!   operation returns a [`Result`] carrying a [`JsValue`] error;
//! - the whole module compiles on the host as a no-op stub so `cargo check`
//!   keeps working without a wasm target, gated by `target_arch = "wasm32"`.

use wasm_bindgen::prelude::*;

/// An interaction event forwarded from the hidden DOM text proxy to the canvas
/// side of the demo.
///
/// The canvas renderer owns the real text field; this enum is the *only* thing
/// the bridge hands back. Consumers are expected to update their own text
/// model in response and, where appropriate, call [`A11yBridge::announce`] so
/// assistive technology hears the result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessEvent {
    /// The `<textarea>` value changed (an `input` or end-of-composition event).
    ///
    /// Contains the complete current text, not a diff.
    TextChanged(String),
    /// An IME composition session started (`compositionstart`).
    CompositionStarted,
    /// An IME composition session produced interim text (`compositionupdate`).
    ///
    /// The payload is the uncommitted composition string, which may be empty.
    CompositionUpdated(String),
    /// An IME composition session ended (`compositionend`).
    ///
    /// The payload is the committed composition string.
    CompositionEnded(String),
    /// The user pressed Enter in the text proxy; the canvas should commit.
    Commit,
    /// The user pressed Escape in the text proxy; the canvas should cancel.
    Cancel,
    /// The user pressed Tab in the text proxy.
    ///
    /// `shift` is true for Shift+Tab (reverse traversal). The bridge does not
    /// move canvas focus on its own; it only reports the intent.
    Tab {
        /// Whether Shift was held during the Tab keypress.
        shift: bool,
    },
}

// ---------------------------------------------------------------------------
// wasm32 implementation (the real DOM bridge)
// ---------------------------------------------------------------------------

/// CSS applied to every DOM node owned by the bridge.
///
/// The elements must stay in the accessibility tree and remain focusable, so
/// `display:none`, `visibility:hidden` and `hidden` are all avoided. The classic
/// 1px clip technique keeps them out of the visual layout instead.
#[cfg(target_arch = "wasm32")]
const OFFSCREEN_STYLE: &str = concat!(
    "position:absolute;",
    "width:1px;height:1px;",
    "padding:0;margin:-1px;",
    "overflow:hidden;clip:rect(0 0 0 0);",
    "white-space:nowrap;border:0;"
);

/// Deliver one [`AccessEvent`] to the installed callback without ever panicking.
///
/// Uses `try_borrow_mut` so a callback that re-enters the bridge cannot trigger
/// a `RefCell` borrow panic; a re-entrant event is simply dropped.
#[cfg(target_arch = "wasm32")]
fn dispatch_event(
    handler: &std::rc::Rc<std::cell::RefCell<Option<Box<dyn FnMut(AccessEvent)>>>>,
    event: AccessEvent,
) {
    if let Ok(mut slot) = handler.try_borrow_mut() {
        if let Some(callback) = slot.as_mut() {
            callback(event);
        }
    }
}

/// One canvas node mirrored into a focusable, labelled DOM element.
#[cfg(target_arch = "wasm32")]
struct MirroredNode {
    /// Stable id chosen by the canvas side.
    id: u64,
    /// Optional parent id, forming a minimal (flat-in-the-DOM) tree.
    parent: Option<u64>,
    /// ARIA role, e.g. `button`, `textbox`, `heading`.
    #[allow(dead_code)]
    role: String,
    /// Human-readable accessible name exposed via `aria-label`.
    label: String,
    /// Canvas-space geometry: x, y, width, height.
    rect: (f32, f32, f32, f32),
    /// The live DOM element handle.
    element: web_sys::HtmlElement,
}

/// Owns a set of offscreen DOM nodes that mirror canvas UI for assistive
/// technology, plus a hidden focusable `<textarea>` used as an IME/keyboard
/// proxy for canvas text fields.
///
/// Construct with [`A11yBridge::new`], then call [`A11yBridge::attach_text_proxy`]
/// to receive [`AccessEvent`]s. All methods are fallible and return a
/// descriptive [`JsValue`] error instead of panicking.
#[cfg(target_arch = "wasm32")]
pub struct A11yBridge {
    /// The canvas being augmented (kept alive for the lifetime of the bridge).
    #[allow(dead_code)]
    canvas: web_sys::HtmlCanvasElement,
    /// Document used to create mirrored elements.
    document: web_sys::Document,
    /// Offscreen container that holds every node the bridge creates.
    container: web_sys::HtmlElement,
    /// Offscreen `role="status"` / `aria-live="polite"` announcement region.
    live_region: web_sys::HtmlElement,
    /// Offscreen but focusable `<textarea>` acting as the IME/text proxy.
    text_proxy: web_sys::HtmlTextAreaElement,
    /// Mirrored canvas nodes, in insertion order.
    nodes: Vec<MirroredNode>,
    /// Shared slot holding the consumer callback; listeners capture an `Rc`
    /// clone so it can be installed after construction.
    event_handler: std::rc::Rc<std::cell::RefCell<Option<Box<dyn FnMut(AccessEvent)>>>>,
    /// DOM event listener closures. They must be owned by the bridge so the
    /// browser does not drop them (wasm-bindgen requires this explicitly).
    listeners: Vec<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
}

#[cfg(target_arch = "wasm32")]
impl A11yBridge {
    /// Create the bridge for the canvas element with the given DOM `id`.
    ///
    /// Returns an error if there is no window/document, no element with that
    /// id, or the element is not a `<canvas>`. No panics.
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        use wasm_bindgen::JsCast;

        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no browser window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("no browser document"))?;
        let body = document
            .body()
            .ok_or_else(|| JsValue::from_str("no document.body"))?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("canvas element not found"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("element is not a canvas"))?;

        let make_div = |tag: &str| -> Result<web_sys::HtmlElement, JsValue> {
            document
                .create_element(tag)?
                .dyn_into::<web_sys::HtmlElement>()
                .map_err(|_| JsValue::from_str("failed to create div element"))
        };

        let container = make_div("div")?;
        container.set_attribute("style", OFFSCREEN_STYLE)?;
        container.set_attribute("data-cvkg-a11y-spike", "1")?;
        body.append_child(&container)?;

        let live_region = make_div("div")?;
        live_region.set_attribute("style", OFFSCREEN_STYLE)?;
        live_region.set_attribute("role", "status")?;
        live_region.set_attribute("aria-live", "polite")?;
        live_region.set_attribute("aria-atomic", "true")?;
        container.append_child(&live_region)?;

        let text_proxy = document
            .create_element("textarea")?
            .dyn_into::<web_sys::HtmlTextAreaElement>()
            .map_err(|_| JsValue::from_str("failed to create textarea"))?;
        text_proxy.set_attribute("style", OFFSCREEN_STYLE)?;
        text_proxy.set_attribute("aria-label", "Canvas text input proxy")?;
        text_proxy.set_attribute("autocomplete", "off")?;
        text_proxy.set_attribute("autocorrect", "off")?;
        text_proxy.set_attribute("autocapitalize", "off")?;
        text_proxy.set_attribute("spellcheck", "false")?;
        text_proxy.set_attribute("tabindex", "0")?;
        container.append_child(&text_proxy)?;

        let mut bridge = Self {
            canvas,
            document,
            container,
            live_region,
            text_proxy,
            nodes: Vec::new(),
            event_handler: std::rc::Rc::new(std::cell::RefCell::new(None)),
            listeners: Vec::new(),
        };
        bridge.install_text_listeners()?;
        Ok(bridge)
    }

    /// Register the callback that receives [`AccessEvent`]s from the text proxy.
    ///
    /// Calling this again replaces the previous callback; listeners are
    /// installed once in [`A11yBridge::new`] and are never duplicated. Returns
    /// an error (never panics) if the callback slot is already borrowed.
    pub fn attach_text_proxy<F>(&mut self, handler: F) -> Result<(), JsValue>
    where
        F: FnMut(AccessEvent) + 'static,
    {
        let mut slot = self
            .event_handler
            .try_borrow_mut()
            .map_err(|_| JsValue::from_str("text proxy callback is already borrowed"))?;
        *slot = Some(Box::new(handler));
        Ok(())
    }

    /// Insert or update a mirrored canvas node.
    ///
    /// `id` is a stable id chosen by the canvas. `parent` optionally names
    /// another mirrored node that this one should be nested under (pass `None`
    /// for a top-level node). `role` and `label` become the element's `role`
    /// `rect` is `(x, y, width, height)` in canvas space and is mirrored to
    /// `data-cvkg-rect` so DOM tests and AT can inspect node geometry.
    /// If `parent` changes, the element is moved in the DOM.
    pub fn upsert_node(
        &mut self,
        id: u64,
        parent: Option<u64>,
        role: &str,
        label: &str,
        rect: (f32, f32, f32, f32),
    ) -> Result<(), JsValue> {
        use wasm_bindgen::JsCast;

        let rect_attr = format!("{},{},{},{}", rect.0, rect.1, rect.2, rect.3);
        if let Some(index) = self.nodes.iter().position(|node| node.id == id) {
            let element = self.nodes[index].element.clone();
            element.set_attribute("role", role)?;
            element.set_attribute("aria-label", label)?;
            element.set_attribute("data-cvkg-rect", &rect_attr)?;
            self.nodes[index].role = role.to_string();
            self.nodes[index].label = label.to_string();
            self.nodes[index].rect = rect;
            if self.nodes[index].parent != parent {
                self.nodes[index].parent = parent;
                let container = self.container.clone();
                Self::append_under(&container, &element, parent, &self.nodes)?;
            }
            return Ok(());
        }

        let element = self
            .document
            .create_element("div")?
            .dyn_into::<web_sys::HtmlElement>()
            .map_err(|_| JsValue::from_str("failed to create mirrored element"))?;
        element.set_attribute("style", OFFSCREEN_STYLE)?;
        element.set_attribute("role", role)?;
        element.set_attribute("aria-label", label)?;
        element.set_attribute("tabindex", "0")?;
        element.set_attribute("data-cvkg-node-id", &id.to_string())?;
        element.set_attribute("data-cvkg-rect", &rect_attr)?;
        Self::append_under(&self.container, &element, parent, &self.nodes)?;
        self.nodes.push(MirroredNode {
            id,
            parent,
            role: role.to_string(),
            label: label.to_string(),
            rect,
            element,
        });
        Ok(())
    }

    /// Remove a previously mirrored node and its DOM element.
    ///
    /// Returns an error if `id` is unknown. Descendants are **not** reparented
    /// or removed — they keep their old DOM location. This is a known spike
    /// limitation; callers should remove children first.
    pub fn remove_node(&mut self, id: u64) -> Result<(), JsValue> {
        let index = self
            .nodes
            .iter()
            .position(|node| node.id == id)
            .ok_or_else(|| JsValue::from_str("node id not found"))?;
        let node = self.nodes.remove(index);
        node.element.remove();
        Ok(())
    }

    /// Move DOM focus to a mirrored node, or clear focus when given `None`.
    ///
    /// Returns an error if `id` is unknown. This does not synthesise the
    /// canvas-side focus ring; the canvas must render that itself.
    pub fn set_focus(&self, id: Option<u64>) -> Result<(), JsValue> {
        match id {
            Some(id) => {
                let element = self
                    .nodes
                    .iter()
                    .find(|node| node.id == id)
                    .map(|node| node.element.clone())
                    .ok_or_else(|| JsValue::from_str("node id not found"))?;
                element.focus()?;
            }
            None => self.text_proxy.blur()?,
        }
        Ok(())
    }

    /// Update the accessible name (`aria-label`) of a mirrored node.
    ///
    /// Returns an error if `id` is unknown.
    pub fn set_node_label(&mut self, id: u64, label: &str) -> Result<(), JsValue> {
        let node = self
            .nodes
            .iter_mut()
            .find(|node| node.id == id)
            .ok_or_else(|| JsValue::from_str("node id not found"))?;
        node.element.set_attribute("aria-label", label)?;
        node.label = label.to_string();
        Ok(())
    }

    /// Post a polite announcement to the offscreen ARIA live region.
    ///
    /// Setting `textContent` is intentionally simple; repeated identical
    /// messages may not be re-announced by all screen readers.
    pub fn announce(&self, message: &str) -> Result<(), JsValue> {
        self.live_region.set_text_content(Some(message));
        Ok(())
    }

    /// Focus the hidden `<textarea>` so IME/keyboard input reaches the proxy.
    ///
    /// The canvas side should call this when a canvas text field gains focus.
    /// A real implementation would also position the proxy at the caret; this
    /// spike does not.
    pub fn focus_text_proxy(&self) -> Result<(), JsValue> {
        self.text_proxy.focus()
    }

    /// Clear the hidden `<textarea>` value (e.g. after a commit or cancel).
    pub fn clear_text_proxy(&self) -> Result<(), JsValue> {
        self.text_proxy.set_value("");
        Ok(())
    }

    /// Read text from `navigator.clipboard`, returning an error on any failure.
    ///
    /// This is a graceful fallback-free path: if the Clipboard API is
    /// unavailable or the user denies permission the returned error describes
    /// the problem, and nothing panics. A real product would add an
    /// `execCommand("paste")` fallback for non-secure contexts; that is
    /// deliberately out of scope for this spike.
    pub async fn read_clipboard(&self) -> Result<String, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no browser window"))?;
        let clipboard = window.navigator().clipboard();
        let promise = clipboard.read_text();
        let value = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|error| JsValue::from_str(&format!("clipboard read failed: {error:?}")))?;
        value
            .as_string()
            .ok_or_else(|| JsValue::from_str("clipboard read returned a non-string value"))
    }

    /// Write text to `navigator.clipboard`, returning an error on any failure.
    ///
    /// Mirrors [`A11yBridge::read_clipboard`]: no panic, no silent success, and
    /// no deprecated `execCommand` fallback in this spike.
    pub async fn write_clipboard(&self, text: &str) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no browser window"))?;
        let clipboard = window.navigator().clipboard();
        let promise = clipboard.write_text(text);
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map(|_| ())
            .map_err(|error| JsValue::from_str(&format!("clipboard write failed: {error:?}")))
    }

    /// Append `element` under `parent` (if it exists in `nodes`) or the bridge
    /// container, replacing any previous parent.
    fn append_under(
        container: &web_sys::HtmlElement,
        element: &web_sys::HtmlElement,
        parent: Option<u64>,
        nodes: &[MirroredNode],
    ) -> Result<(), JsValue> {
        match parent {
            Some(parent_id) => {
                let parent_element = nodes
                    .iter()
                    .find(|node| node.id == parent_id)
                    .map(|node| node.element.clone())
                    .ok_or_else(|| JsValue::from_str("parent node id not found"))?;
                parent_element.append_child(element)?;
            }
            None => {
                container.append_child(element)?;
            }
        }
        Ok(())
    }

    /// Install the five text-proxy DOM listeners. The closures capture an `Rc`
    /// clone of the callback slot, so `attach_text_proxy` may be called later.
    fn install_text_listeners(&mut self) -> Result<(), JsValue> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;

        // input -> full current value
        {
            let handler = self.event_handler.clone();
            let proxy = self.text_proxy.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let value = proxy.value();
                dispatch_event(&handler, AccessEvent::TextChanged(value));
            }) as Box<dyn FnMut(web_sys::Event)>);
            self.text_proxy
                .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            self.listeners.push(closure);
        }

        // compositionstart
        {
            let handler = self.event_handler.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                dispatch_event(&handler, AccessEvent::CompositionStarted);
            }) as Box<dyn FnMut(web_sys::Event)>);
            self.text_proxy.add_event_listener_with_callback(
                "compositionstart",
                closure.as_ref().unchecked_ref(),
            )?;
            self.listeners.push(closure);
        }

        // compositionupdate
        {
            let handler = self.event_handler.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let composition: web_sys::CompositionEvent = event.unchecked_into();
                let data = composition.data().unwrap_or_default();
                dispatch_event(&handler, AccessEvent::CompositionUpdated(data));
            }) as Box<dyn FnMut(web_sys::Event)>);
            self.text_proxy.add_event_listener_with_callback(
                "compositionupdate",
                closure.as_ref().unchecked_ref(),
            )?;
            self.listeners.push(closure);
        }

        // compositionend -> committed text + resulting value
        {
            let handler = self.event_handler.clone();
            let proxy = self.text_proxy.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let composition: web_sys::CompositionEvent = event.unchecked_into();
                let data = composition.data().unwrap_or_default();
                dispatch_event(&handler, AccessEvent::CompositionEnded(data));
                dispatch_event(&handler, AccessEvent::TextChanged(proxy.value()));
            }) as Box<dyn FnMut(web_sys::Event)>);
            self.text_proxy.add_event_listener_with_callback(
                "compositionend",
                closure.as_ref().unchecked_ref(),
            )?;
            self.listeners.push(closure);
        }

        // keydown -> Enter / Escape / Tab intents
        {
            let handler = self.event_handler.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let keyboard: web_sys::KeyboardEvent = event.unchecked_into();
                let access_event = match keyboard.key().as_str() {
                    "Enter" => Some(AccessEvent::Commit),
                    "Escape" => Some(AccessEvent::Cancel),
                    "Tab" => Some(AccessEvent::Tab {
                        shift: keyboard.shift_key(),
                    }),
                    _ => None,
                };
                if let Some(access_event) = access_event {
                    dispatch_event(&handler, access_event);
                }
            }) as Box<dyn FnMut(web_sys::Event)>);
            self.text_proxy
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
            self.listeners.push(closure);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Non-wasm stub: identical API, no DOM, no success path.
// ---------------------------------------------------------------------------

/// Error returned by every method of the non-wasm fallback implementation.
#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> JsValue {
    JsValue::from_str("A11yBridge is a wasm-only P1-3 spike; unsupported on this target")
}

/// Host (non-wasm) no-op stub for [`A11yBridge`].
///
/// The type and every method signature mirror the wasm implementation so the
/// rest of the crate compiles on the host. [`A11yBridge::new`] always returns
/// an `Unsupported` error and no instance is ever produced.
#[cfg(not(target_arch = "wasm32"))]
pub struct A11yBridge {
    /// Zero-sized marker so the stub is not an empty struct.
    _private: (),
}

#[cfg(not(target_arch = "wasm32"))]
impl A11yBridge {
    /// Always returns an `Unsupported` error on non-wasm targets.
    pub fn new(_canvas_id: &str) -> Result<Self, JsValue> {
        Err(unsupported())
    }

    /// No-op on non-wasm targets; returns an `Unsupported` error.
    pub fn attach_text_proxy<F>(&mut self, _handler: F) -> Result<(), JsValue>
    where
        F: FnMut(AccessEvent) + 'static,
    {
        Err(unsupported())
    }

    /// No-op on non-wasm targets; returns an `Unsupported` error.
    pub fn upsert_node(
        &mut self,
        _id: u64,
        _parent: Option<u64>,
        _role: &str,
        _label: &str,
        _rect: (f32, f32, f32, f32),
    ) -> Result<(), JsValue> {
        Err(unsupported())
    }

    /// No-op on non-wasm targets; returns an `Unsupported` error.
    pub fn remove_node(&mut self, _id: u64) -> Result<(), JsValue> {
        Err(unsupported())
    }

    /// No-op on non-wasm targets; returns an `Unsupported` error.
    pub fn set_focus(&self, _id: Option<u64>) -> Result<(), JsValue> {
        Err(unsupported())
    }

    /// No-op on non-wasm targets; returns an `Unsupported` error.
    pub fn set_node_label(&mut self, _id: u64, _label: &str) -> Result<(), JsValue> {
        Err(unsupported())
    }

    /// No-op on non-wasm targets; returns an `Unsupported` error.
    pub fn announce(&self, _message: &str) -> Result<(), JsValue> {
        Err(unsupported())
    }

    /// No-op on non-wasm targets; returns an `Unsupported` error.
    pub fn focus_text_proxy(&self) -> Result<(), JsValue> {
        Err(unsupported())
    }

    /// No-op on non-wasm targets; returns an `Unsupported` error.
    pub fn clear_text_proxy(&self) -> Result<(), JsValue> {
        Err(unsupported())
    }

    /// Always returns an `Unsupported` error on non-wasm targets.
    pub async fn read_clipboard(&self) -> Result<String, JsValue> {
        Err(unsupported())
    }

    /// Always returns an `Unsupported` error on non-wasm targets.
    pub async fn write_clipboard(&self, _text: &str) -> Result<(), JsValue> {
        Err(unsupported())
    }
}
