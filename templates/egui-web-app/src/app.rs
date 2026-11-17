//! Ordinary application code: eframe owns the UI and logic lifecycle.
use eframe::egui;

pub struct Notes {
    draft: String,
    saved: Option<String>,
    save_requested: bool,
}

impl Notes {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            draft: "First note".into(),
            saved: None,
            save_requested: false,
        }
    }
}

impl eframe::App for Notes {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if std::mem::take(&mut self.save_requested) {
            self.saved = Some(self.draft.clone());
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Notes");
            let label = ui.label("Title");
            ui.text_edit_singleline(&mut self.draft)
                .labelled_by(label.id);
            if ui.button("Save").clicked() {
                self.save_requested = true;
                ui.ctx().request_repaint();
            }
            ui.label(match &self.saved {
                Some(title) => format!("Saved: {title}"),
                None => "No saved note".into(),
            });
        });
    }
}
