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
    pub async fn start(resettable: bool) -> Result<WebApp, JsValue> {
        launch(Box::new(move |cc| {
            let sandbox = Sandbox::install(&cc.egui_ctx);
            if resettable {
                sandbox.enable_reset();
            }
            Ok(Box::new(Demo {
                app: Workbench::default(),
                sandbox,
            }))
        }))
        .await
    }
}
