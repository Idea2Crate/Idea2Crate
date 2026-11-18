//! Browser entry only: the application's UI has no sandbox dependency.
#[cfg(target_arch = "wasm32")]
#[path = "basic/workbench.rs"]
mod workbench;

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::workbench::Workbench;
    use eframe::egui;
    use egui_web_sandbox::{
        Sandbox,
        web::{WebApp, launch},
    };
    use wasm_bindgen::prelude::*;

    struct Demo {
        app: Workbench,
        sandbox: Sandbox,
    }
    impl eframe::App for Demo {
        fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
            if self.sandbox.take_reset_request() {
                self.app = Workbench::default();
                ui.ctx().memory_mut(|memory| *memory = Default::default());
                self.sandbox.reset();
            }
            self.app.ui(ui, frame);
        }
    }

    #[wasm_bindgen]
    pub async fn start() -> Result<WebApp, JsValue> {
        let query = web_sys::window()
            .ok_or_else(|| JsValue::from_str("window unavailable"))?
            .location()
            .search()?;
        let resettable = !web_sys::UrlSearchParams::new_with_str(&query)?.has("observe");
        launch(move |cc| {
            let sandbox = Sandbox::install(&cc.egui_ctx);
            if resettable {
                sandbox.enable_reset();
            }
            Demo {
                app: Workbench::default(),
                sandbox,
            }
        })
        .await
    }
}
