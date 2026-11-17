//! Browser entry only: the application's UI has no sandbox dependency.
#[cfg(target_arch = "wasm32")]
#[path = "basic/workbench.rs"]
mod workbench;

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::workbench::Workbench;
    use eframe::egui;
    use egui_web_sandbox::{Sandbox, web::AgentBridge};
    use wasm_bindgen::{JsCast, prelude::*};

    struct Demo {
        app: Workbench,
        sandbox: Sandbox,
        _bridge: AgentBridge,
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
    pub async fn start(resettable: bool) -> Result<(), JsValue> {
        let canvas = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("canvas"))
            .ok_or_else(|| JsValue::from_str("canvas unavailable"))?
            .dyn_into()?;
        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |cc| {
                    let sandbox = Sandbox::install(&cc.egui_ctx);
                    if resettable {
                        sandbox.enable_reset();
                    }
                    let bridge = AgentBridge::install(&cc.egui_ctx, &sandbox)
                        .map_err(|error| format!("agent bridge: {error:?}"))?;
                    // The launcher and an outer adapter may attach independently.
                    // Both obtain the same observer/environment for this context.
                    Ok(Box::new(Demo {
                        app: Workbench::default(),
                        sandbox: Sandbox::install(&cc.egui_ctx),
                        _bridge: bridge,
                    }))
                }),
            )
            .await
    }
}
