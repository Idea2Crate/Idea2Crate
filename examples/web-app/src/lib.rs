#[cfg(target_arch = "wasm32")]
mod app;

/// The Web entry supplies only the ordinary App factory.
#[cfg(target_arch = "wasm32")]
#[egui_web_sandbox::wasm_bindgen]
fn create_app() -> eframe::AppCreator<'static> {
    Box::new(|cc| Ok(Box::new(app::Notes::new(cc))))
}
