#[cfg(target_arch = "wasm32")]
mod app;

#[cfg(target_arch = "wasm32")]
use {
    egui_web_sandbox::web::{WebApp, launch},
    wasm_bindgen::prelude::*,
};

/// The Web entry supplies only the ordinary App factory.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start() -> Result<WebApp, JsValue> {
    launch(app::Notes::new).await
}
