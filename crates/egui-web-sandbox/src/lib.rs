//! Attach browser observations to an existing egui interface without replacing it.
//! Native services and application fixtures remain the application's responsibility.
use egui::{ColorImage, Context};
use std::sync::Arc;

pub mod desktop;
mod observe;
pub use observe::{Control, ControlRole, Snapshot, WindowSnapshot};
pub const PROTOCOL_VERSION: u32 = 1;
#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web;
#[cfg(feature = "launcher")]
pub use egui_web_sandbox_macros::wasm_bindgen;

#[cfg(all(feature = "launcher", target_arch = "wasm32"))]
#[doc(hidden)]
pub mod __private {
    pub use {js_sys, wasm_bindgen, wasm_bindgen_futures};
}

/// One observer and in-memory test environment per egui context.
#[derive(Clone)]
pub struct Sandbox(Arc<egui::mutex::Mutex<State>>);

#[derive(Default)]
struct State {
    image: Option<ColorImage>,
    text: Option<String>,
    snapshot: Snapshot,
    reset_enabled: bool,
    reset_requested: bool,
    generation: u32,
    // Optional custom-window observations; ordinary egui windows are discovered automatically.
    windows: Vec<WindowSnapshot>,
    focused_window: Option<Option<u64>>,
    clipboard_events: Vec<egui::Event>,
}

impl Sandbox {
    /// Attach once or return the existing instance. No per-frame registration is required.
    pub fn install(ctx: &Context) -> Self {
        if let Some(existing) = ctx.with_plugin::<observe::Observer, _>(|plugin| plugin.0.clone()) {
            return existing;
        }
        let sandbox = Self(Default::default());
        ctx.enable_accesskit();
        ctx.add_plugin(observe::Observer(sandbox.clone()));
        sandbox
    }

    /// Opt in before installing the web bridge, when the application handles `take_reset_request`.
    pub fn enable_reset(&self) {
        self.0.lock().reset_enabled = true;
    }

    pub fn image(&self) -> Option<ColorImage> {
        self.0.lock().image.clone()
    }
    pub fn has_image(&self) -> bool {
        self.0.lock().image.is_some()
    }
    pub fn set_image(&self, image: Option<ColorImage>) {
        let mut state = self.0.lock();
        state.image = image;
        state.text = None;
    }
    pub fn text(&self) -> Option<String> {
        self.0.lock().text.clone()
    }
    pub fn set_text(&self, text: String) {
        let mut state = self.0.lock();
        state.text = Some(text);
        state.image = None;
    }
    pub fn snapshot(&self) -> Snapshot {
        self.0.lock().snapshot.clone()
    }

    /// Coalesce pending requests. `None` means no application reset handler was enabled.
    pub fn request_reset(&self, ctx: &Context) -> Option<u32> {
        let mut state = self.0.lock();
        if !state.reset_enabled {
            return None;
        }
        state.reset_requested = true;
        let generation = state.generation.wrapping_add(1);
        drop(state);
        ctx.request_repaint();
        Some(generation)
    }
    pub fn take_reset_request(&self) -> bool {
        std::mem::take(&mut self.0.lock().reset_requested)
    }
    /// Acknowledge after restoring application state. Does not replace egui memory or plugins.
    pub fn reset(&self) {
        let mut state = self.0.lock();
        *state = State {
            reset_enabled: state.reset_enabled,
            generation: state.generation.wrapping_add(1),
            ..Default::default()
        };
    }
}
