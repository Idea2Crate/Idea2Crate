//! An ordinary egui application: no sandbox imports, window registration or test-only buttons.
use eframe::egui;

struct Document {
    title: String,
    notes: String,
    reviewed: bool,
    priority: u32,
    published: bool,
    revisions: u32,
}

pub struct Workbench {
    documents: [Document; 2],
    inspector: bool,
    nodes: [egui::Vec2; 2],
}

impl Default for Workbench {
    fn default() -> Self {
        Self {
            documents: [
                Document {
                    title: "Product brief".into(),
                    notes: "A small workspace for a new idea.".into(),
                    reviewed: false,
                    priority: 3,
                    published: false,
                    revisions: 0,
                },
                Document {
                    title: "Release checklist".into(),
                    notes: "Review changes, test, then publish.".into(),
                    reviewed: true,
                    priority: 5,
                    published: false,
                    revisions: 0,
                },
            ],
            inspector: false,
            nodes: [egui::vec2(30.0, 45.0), egui::vec2(340.0, 70.0)],
        }
    }
}

impl eframe::App for Workbench {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.painter()
            .rect_filled(ui.max_rect(), 0, ui.visuals().panel_fill);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.heading("Workspace Lab");
            ui.separator();
            ui.label("An ordinary egui application, observed in the browser");
            ui.checkbox(&mut self.inspector, "Inspector");
        });
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label("Edit documents, copy text, move windows and drag pipeline nodes. Each page is independent.");
        });
        let document_width = ((ui.ctx().viewport_rect().width() - 108.0) / 2.0).clamp(240.0, 420.0);
        for (index, document) in self.documents.iter_mut().enumerate() {
            let name = ["Project brief", "Release notes"][index];
            egui::Window::new(name)
                .id(egui::Id::unique(("document", index)))
                .default_pos(egui::pos2(
                    36.0 + index as f32 * (document_width + 50.0),
                    100.0,
                ))
                .default_width(document_width)
                .default_height(300.0)
                .show(ui.ctx(), |ui| {
                    ui.label("DOCUMENT")
                        .on_hover_text("Two windows deliberately share the same control names.");
                    let title = ui.label("Title");
                    ui.add(
                        egui::TextEdit::singleline(&mut document.title)
                            .desired_width(f32::INFINITY),
                    )
                    .labelled_by(title.id);
                    let notes = ui.label("Notes");
                    ui.add(
                        egui::TextEdit::multiline(&mut document.notes)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    )
                    .labelled_by(notes.id);
                    ui.add_space(8.0);
                    ui.checkbox(&mut document.reviewed, "Reviewed");
                    ui.add(egui::Slider::new(&mut document.priority, 1..=10).text("Priority"));
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut document.published, false, "Draft");
                        ui.radio_value(&mut document.published, true, "Published");
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !document.title.trim().is_empty(),
                                egui::Button::new("Save"),
                            )
                            .clicked()
                        {
                            document.revisions += 1;
                        }
                        ui.label(format!("Saved revision: {}", document.revisions));
                    });
                });
        }

        let viewport = ui.ctx().viewport_rect();
        let expanded_pipeline = viewport.height() >= 740.0;
        egui::Window::new("Pipeline")
            .default_pos(egui::pos2(
                36.0,
                if expanded_pipeline {
                    475.0
                } else {
                    viewport.height() - 40.0
                },
            ))
            .default_open(expanded_pipeline)
            .default_height(210.0)
            .default_width((ui.ctx().viewport_rect().width() - 72.0).min(890.0))
            .show(ui.ctx(), |ui| {
                ui.label("Custom drawing with standard egui drag responses and accessible names");
                let (canvas, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 170.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(canvas, 8, ui.visuals().extreme_bg_color);
                let node_size = egui::vec2(150.0, 68.0);
                ui.painter().line_segment(
                    [
                        canvas.min + self.nodes[0] + node_size / 2.0,
                        canvas.min + self.nodes[1] + node_size / 2.0,
                    ],
                    egui::Stroke::new(2.0, ui.visuals().selection.stroke.color),
                );
                for (index, position) in self.nodes.iter_mut().enumerate() {
                    let name = ["Input node", "Output node"][index];
                    let rect = egui::Rect::from_min_size(canvas.min + *position, node_size);
                    let response = ui
                        .interact(
                            rect,
                            ui.make_persistent_id(("node", index)),
                            egui::Sense::click_and_drag(),
                        )
                        .on_hover_cursor(egui::CursorIcon::Grab);
                    if response.dragged() {
                        *position = (*position + response.drag_delta())
                            .clamp(egui::Vec2::ZERO, canvas.size() - node_size);
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    }
                    response
                        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
                    ui.painter()
                        .rect_filled(rect, 8, ui.visuals().widgets.inactive.bg_fill);
                    ui.painter().rect_stroke(
                        rect,
                        8,
                        ui.visuals().widgets.inactive.bg_stroke,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        name,
                        egui::FontId::proportional(16.0),
                        ui.visuals().text_color(),
                    );
                }
            });

        if self.inspector {
            let summary = self
                .documents
                .iter()
                .map(|document| format!("{} · revision {}", document.title, document.revisions))
                .collect::<Vec<_>>();
            ui.ctx().show_viewport_deferred(
                egui::ViewportId::from_hash_of("inspector"),
                egui::ViewportBuilder::default()
                    .with_title("Inspector")
                    .with_inner_size([320.0, 160.0])
                    .with_position([970.0, 100.0]),
                move |ui, _class| {
                    ui.heading("Live document summary");
                    for line in &summary {
                        ui.label(line);
                    }
                    ui.label("Uses egui's built-in embedded viewport on the web.");
                },
            );
        }
    }
}
