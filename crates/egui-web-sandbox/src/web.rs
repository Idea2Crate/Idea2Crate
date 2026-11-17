//! Browser automation observes state and sends normal DOM mouse/keyboard input.
use super::*;
use wasm_bindgen::{JsCast, prelude::*};
type ClipboardHandler = Closure<dyn Fn(web_sys::ClipboardEvent)>;

/// Owns `window.eguiSandbox` for this page. Dropping it removes the bridge.
pub struct AgentBridge {
    _snapshot: Closure<dyn Fn() -> JsValue>,
    _reset: Closure<dyn Fn() -> Result<u32, JsValue>>,
    document: web_sys::Document,
    key: Closure<dyn Fn(web_sys::KeyboardEvent)>,
    clipboard: Vec<(&'static str, ClipboardHandler)>,
}
impl AgentBridge {
    pub fn install(ctx: &Context, sandbox: &Sandbox) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
        if js_sys::Reflect::has(&window, &"eguiSandbox".into())? {
            return Err(JsValue::from_str("this page already has a sandbox bridge"));
        }
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("document unavailable"))?;
        let observed = sandbox.clone();
        let snapshot = Closure::wrap(Box::new(move || {
            serde_json::to_string(&observed.snapshot())
                .ok()
                .and_then(|json| js_sys::JSON::parse(&json).ok())
                .unwrap_or(JsValue::NULL)
        }) as Box<dyn Fn() -> JsValue>);
        let reset_host = sandbox.clone();
        let repaint = ctx.clone();
        let reset = Closure::wrap(Box::new(move || {
            reset_host
                .request_reset(&repaint)
                .ok_or_else(|| JsValue::from_str("application reset is not enabled"))
        }) as Box<dyn Fn() -> Result<u32, JsValue>>);
        let bridge = js_sys::Object::new();
        js_sys::Reflect::set(&bridge, &"version".into(), &JsValue::from(PROTOCOL_VERSION))?;
        js_sys::Reflect::set(
            &bridge,
            &"snapshot".into(),
            snapshot.as_ref().unchecked_ref(),
        )?;
        if sandbox.0.lock().reset_enabled {
            js_sys::Reflect::set(&bridge, &"reset".into(), reset.as_ref().unchecked_ref())?;
        }
        js_sys::Reflect::set(
            &window,
            &"eguiSandbox".into(),
            &js_sys::Object::freeze(&bridge),
        )?;
        let focused = ctx.clone();
        let key = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if focused.input(|input| input.focused)
                && (event.ctrl_key() || event.meta_key())
                && !event.alt_key()
                && !event.shift_key()
                && matches!(event.key().to_ascii_lowercase().as_str(), "c" | "x" | "v")
            {
                // Let eframe receive the key, while preventing OS clipboard access.
                event.prevent_default();
            }
        }) as Box<dyn Fn(web_sys::KeyboardEvent)>);
        let mut bridge = Self {
            _snapshot: snapshot,
            _reset: reset,
            document,
            key,
            clipboard: Vec::new(),
        };
        bridge.document.add_event_listener_with_callback_and_bool(
            "keydown",
            bridge.key.as_ref().unchecked_ref(),
            true,
        )?;
        for name in ["copy", "cut", "paste"] {
            let host = sandbox.clone();
            let focused = ctx.clone();
            let handler = Closure::wrap(Box::new(move |event: web_sys::ClipboardEvent| {
                if !focused.input(|input| input.focused) {
                    return;
                }
                event.prevent_default();
                event.stop_immediate_propagation();
                let input = match name {
                    "copy" => Some(egui::Event::Copy),
                    "cut" => Some(egui::Event::Cut),
                    _ => host.text().map(egui::Event::Paste),
                };
                if let Some(input) = input {
                    host.0.lock().clipboard_events.push(input);
                    focused.request_repaint();
                }
            }) as Box<dyn Fn(web_sys::ClipboardEvent)>);
            bridge.document.add_event_listener_with_callback_and_bool(
                name,
                handler.as_ref().unchecked_ref(),
                true,
            )?;
            bridge.clipboard.push((name, handler));
        }
        Ok(bridge)
    }
}
impl Drop for AgentBridge {
    fn drop(&mut self) {
        let _ = self.document.remove_event_listener_with_callback_and_bool(
            "keydown",
            self.key.as_ref().unchecked_ref(),
            true,
        );
        for (name, handler) in &self.clipboard {
            let _ = self.document.remove_event_listener_with_callback_and_bool(
                name,
                handler.as_ref().unchecked_ref(),
                true,
            );
        }
        if let Some(window) = web_sys::window() {
            let _ = js_sys::Reflect::delete_property(&window, &"eguiSandbox".into());
        }
    }
}
