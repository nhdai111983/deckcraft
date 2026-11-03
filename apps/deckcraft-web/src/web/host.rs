//! The `postMessage` bridge to a same-origin parent page, for pages that embed the app in an
//! `<iframe>`. Messages are plain objects with a `type`:
//!
//! - `deckcraft:ready` (to the parent): `{ version }`, the app is listening; send a file now.
//! - `deckcraft:open` (from the parent): `{ name, bytes }`, bytes an `ArrayBuffer` or
//!   `Uint8Array`. It opens like a presentation picked with File › Open or dropped on the window.
//! - `deckcraft:save` (to the parent): `{ name, bytes }` (a transferred `ArrayBuffer`) for every
//!   file the app would otherwise download: Save, Save As and the exports. The parent stores it
//!   and reports failures itself.
//! - `deckcraft:dirty` (to the parent): `{ dirty }` whenever unsaved changes appear or go away.
//! - `deckcraft:failed` (to the parent): `{ error }` when the app couldn't start.
//!
//! Only the parent window on the page's own origin is heard or answered.
//!
//! A file from the parent can't go straight into the inbox: every inbox event carries the id of
//! the operation that asked for it, and only the app can start one. So the listener parks the
//! file in [`Bridge::pending`] and the shell hands it over on the next pass, exactly as it does
//! for a dropped file.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use deckcraft_ui_egui::{Inbox, InboxEvent, InboxFile, Services, SlideApp};
use wasm_bindgen::JsCast as _;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

/// The name an opened presentation falls back to when the parent sends none.
const FALLBACK_NAME: &str = "Presentation.pptx";

/// Most files held from the parent while the app hasn't taken them yet. A page sending faster
/// than the app paints doesn't grow this without end.
const MAX_PENDING: usize = 8;

#[derive(Clone)]
pub struct Bridge {
    window: web_sys::Window,
    parent: web_sys::Window,
    origin: String,
    /// Files from the parent waiting for the shell to give them an operation id.
    pending: Rc<RefCell<Vec<(String, Vec<u8>)>>>,
    /// The unsaved-changes state last reported to the parent.
    reported: Rc<Cell<Option<bool>>>,
}

impl Bridge {
    /// `None` when the page isn't framed (there is no parent to talk to).
    pub fn new() -> Option<Self> {
        let window = web_sys::window()?;
        let parent = window.parent().ok().flatten()?;
        if js_sys::Object::is(&parent, &window) {
            return None;
        }
        let origin = window.location().origin().ok()?;
        Some(Self { window, parent, origin, pending: Rc::default(), reported: Rc::default() })
    }

    /// Send saved and exported files to the parent instead of downloading them, and hear files
    /// from it.
    pub fn connect(&self, services: &mut Services, ctx: egui::Context) {
        self.listen(ctx);

        let bridge = self.clone();
        services.write = Some(Box::new(move |name: &str, bytes: &[u8]| bridge.post_save(name, bytes)));

        let bridge = self.clone();
        services.download = Some(Box::new(move |name: &str, bytes: &[u8]| {
            if let Err(e) = bridge.post_save(name, bytes) {
                log::error!("couldn't hand {name} to the page: {e}");
            }
        }));
    }

    pub fn post_ready(&self) {
        self.send(&[("type", "deckcraft:ready".into()), ("version", env!("CARGO_PKG_VERSION").into())]);
    }

    pub fn post_failed(&self, error: &str) {
        self.send(&[("type", "deckcraft:failed".into()), ("error", error.into())]);
    }

    /// Each pass: give any file from the parent the operation id the app expects, and tell the
    /// parent when unsaved changes appear or go away.
    pub fn sync(&self, app: &mut SlideApp, inbox: &Inbox, ctx: &egui::Context) {
        let incoming = std::mem::take(&mut *self.pending.borrow_mut());
        for (name, bytes) in incoming {
            let id = app.begin_external_file("host");
            inbox.lock().unwrap_or_else(|e| e.into_inner()).push(InboxEvent::File(InboxFile { id, name, bytes }));
            ctx.request_repaint();
        }
        let dirty = app.session.is_dirty();
        if self.reported.get() != Some(dirty) && self.send(&[("type", "deckcraft:dirty".into()), ("dirty", dirty.into())]) {
            self.reported.set(Some(dirty));
        }
    }

    /// Hand the bytes to the parent. The buffer is transferred, so it is not copied again.
    fn post_save(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        let name = std::path::Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string());
        let buffer = js_sys::Uint8Array::from(bytes).buffer();
        let message = message(&[("type", "deckcraft:save".into()), ("name", name.as_str().into()), ("bytes", buffer.clone().into())])
            .ok_or_else(|| "couldn't build the message".to_string())?;
        self.parent
            .post_message_with_transfer(&message, &self.origin, &js_sys::Array::of1(&buffer))
            .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))
    }

    fn listen(&self, ctx: egui::Context) {
        let parent = self.parent.clone();
        let origin = self.origin.clone();
        let pending = self.pending.clone();
        // The event is read field by field rather than as a `web_sys::MessageEvent`, so the
        // crate needs no extra web-sys feature and the manifest stays as upstream has it.
        let on_message = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
            if string(&event, "origin").as_deref() != Some(origin.as_str()) {
                return;
            }
            if !js_sys::Reflect::get(&event, &JsValue::from_str("source")).is_ok_and(|s| js_sys::Object::is(&s, &parent)) {
                return;
            }
            let Ok(data) = js_sys::Reflect::get(&event, &JsValue::from_str("data")) else {
                return;
            };
            if string(&data, "type").as_deref() != Some("deckcraft:open") {
                return;
            }
            let name = string(&data, "name").filter(|n| !n.trim().is_empty()).unwrap_or_else(|| FALLBACK_NAME.to_string());
            let Some(bytes) = bytes(&data) else { return };
            let mut pending = pending.borrow_mut();
            if pending.len() >= MAX_PENDING {
                log::warn!("the page sent files faster than they could be opened; one was dropped");
                return;
            }
            pending.push((name, bytes));
            drop(pending);
            ctx.request_repaint();
        });
        if self.window.add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref()).is_ok() {
            // The listener lives as long as the page.
            on_message.forget();
        }
    }

    /// Post a message to the parent; false if it couldn't be sent.
    fn send(&self, fields: &[(&str, JsValue)]) -> bool {
        message(fields).is_some_and(|m| self.parent.post_message(&m, &self.origin).is_ok())
    }
}

fn message(fields: &[(&str, JsValue)]) -> Option<js_sys::Object> {
    let m = js_sys::Object::new();
    for (key, value) in fields {
        js_sys::Reflect::set(&m, &JsValue::from_str(key), value).ok()?;
    }
    Some(m)
}

fn string(data: &JsValue, key: &str) -> Option<String> {
    js_sys::Reflect::get(data, &JsValue::from_str(key)).ok()?.as_string()
}

/// `bytes` as an `ArrayBuffer` or `Uint8Array`.
fn bytes(data: &JsValue) -> Option<Vec<u8>> {
    let bytes = js_sys::Reflect::get(data, &JsValue::from_str("bytes")).ok()?;
    if let Some(array) = bytes.dyn_ref::<js_sys::Uint8Array>() {
        Some(array.to_vec())
    } else if bytes.is_instance_of::<js_sys::ArrayBuffer>() {
        Some(js_sys::Uint8Array::new(&bytes).to_vec())
    } else {
        None
    }
}
