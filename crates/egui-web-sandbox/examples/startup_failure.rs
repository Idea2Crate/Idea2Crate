//! Browser regression fixture for a failing application constructor.
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = "export function failure_prefix() { return 'fixture '; }")]
extern "C" {
    fn failure_prefix() -> String;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/examples/js/nested/failure.mjs")]
extern "C" {
    fn failure_suffix() -> String;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start() -> Result<egui_web_sandbox::web::WebApp, JsValue> {
    egui_web_sandbox::web::launch(Box::new(|_| {
        Err(format!("{}{}", failure_prefix(), failure_suffix()).into())
    }))
    .await
}
