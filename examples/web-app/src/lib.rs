mod app;

/// The application factory is shared by native and browser hosts.
#[egui_web_sandbox::wasm_bindgen]
pub fn create_app() -> eframe::AppCreator<'static> {
    Box::new(|cc| Ok(Box::new(app::Notes::new(cc))))
}
