#[cfg(target_arch = "wasm32")]
mod app;

#[cfg(target_arch = "wasm32")]
use {
    egui_web_sandbox::web::{WebApp, launch},
    wasm_bindgen::prelude::*,
};

/// Replace the factory with your existing App constructor.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start() -> Result<WebApp, JsValue> {
    launch(Box::new(|cc| Ok(Box::new(app::Notes::new(cc))))).await
}
