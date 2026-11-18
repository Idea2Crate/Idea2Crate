//! Browser regression fixture for a failing application constructor.
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start() -> Result<egui_web_sandbox::web::WebApp, JsValue> {
    egui_web_sandbox::web::try_launch(|_| Err("fixture constructor failed".into())).await
}
