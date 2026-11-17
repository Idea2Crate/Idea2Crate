//! Own the renderer and observation bridge without wrapping the application's App.
use super::{AgentBridge, available_document};
use crate::Sandbox;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};

/// A running browser app. Keep the returned JavaScript object alive until unmount.
/// `destroy()` and the generated `free()` both release renderer and bridge resources.
#[wasm_bindgen]
pub struct WebApp {
    runner: eframe::WebRunner,
    bridge: Rc<RefCell<Option<AgentBridge>>>,
}

#[wasm_bindgen]
impl WebApp {
    /// Stop this app. Repeated calls are harmless, including after another app starts.
    pub fn destroy(&self) {
        self.runner.destroy();
        self.bridge.borrow_mut().take();
    }

    pub fn has_panicked(&self) -> bool {
        self.runner.has_panicked()
    }
}

impl Drop for WebApp {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// Start an App on the template's `#canvas` using eframe's default WebOptions.
///
/// The factory receives the actual CreationContext. All App hooks remain owned by
/// eframe; the launcher does not proxy or replace them. Native services and optional
/// fixture reset handling remain the application's responsibility.
pub async fn launch(app_creator: eframe::AppCreator<'static>) -> Result<WebApp, JsValue> {
    let canvas = available_document()?
        .get_element_by_id("canvas")
        .ok_or_else(|| JsValue::from_str("canvas #canvas unavailable"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("element #canvas is not a canvas"))?;
    launch_on(canvas, eframe::WebOptions::default(), app_creator).await
}

/// Start on an existing canvas with project-specific renderer options.
pub async fn launch_on(
    canvas: web_sys::HtmlCanvasElement,
    options: eframe::WebOptions,
    app_creator: eframe::AppCreator<'static>,
) -> Result<WebApp, JsValue> {
    available_document()?;
    let app = WebApp {
        runner: eframe::WebRunner::new(),
        bridge: Rc::new(RefCell::new(None)),
    };
    let bridge = app.bridge.clone();
    app.runner
        .start(
            canvas,
            options,
            Box::new(move |cc| {
                let sandbox = Sandbox::install(&cc.egui_ctx);
                let instance = app_creator(cc)?;
                *bridge.borrow_mut() = Some(
                    AgentBridge::install(&cc.egui_ctx, &sandbox)
                        .map_err(|error| format!("sandbox bridge: {error:?}"))?,
                );
                Ok(instance)
            }),
        )
        .await?;
    Ok(app)
}
