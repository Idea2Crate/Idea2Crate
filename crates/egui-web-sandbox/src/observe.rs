use super::*;
use egui::Rect;
use serde::Serialize;
use std::collections::HashMap;

/// All rectangles use egui points, [left, top, right, bottom]. No image bytes.
#[derive(Clone, Default, Serialize)]
#[non_exhaustive]
pub struct Snapshot {
    pub schema_version: u32,
    pub generation: u32,
    pub frame: u64,
    pub bounds: [f32; 4],
    pub pixels_per_point: f32,
    pub zoom_factor: f32,
    pub focused_window: Option<String>,
    pub windows: Vec<WindowSnapshot>,
    pub controls: Vec<Control>,
    pub clipboard_image_size: Option<[usize; 2]>,
    pub clipboard_has_text: bool,
    pub ready: bool,
    pub reset_supported: bool,
    pub focused_control: Option<String>,
}

#[derive(Clone, Serialize)]
#[non_exhaustive]
pub struct WindowSnapshot {
    #[serde(serialize_with = "serialize_id")]
    pub id: u64,
    pub title: String,
    pub bounds: [f32; 4],
}
impl WindowSnapshot {
    pub(crate) fn rect(&self) -> Rect {
        Rect::from_min_max(
            egui::pos2(self.bounds[0], self.bounds[1]),
            egui::pos2(self.bounds[2], self.bounds[3]),
        )
    }
}

#[derive(Clone, Serialize)]
#[non_exhaustive]
pub struct Control {
    pub id: String,
    pub role: ControlRole,
    pub label: String,
    pub value: Option<String>,
    pub bounds: [f64; 4],
    pub disabled: bool,
    pub parent: Option<String>,
    pub window: Option<String>,
    pub focused: bool,
    pub checked: Option<bool>,
    pub mixed: bool,
    pub selected: Option<bool>,
    pub expanded: Option<bool>,
    pub numeric_value: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

// These names belong to our protocol, rather than AccessKit's Debug implementation.
macro_rules! roles {
    ($($role:ident),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
        #[non_exhaustive]
        pub enum ControlRole { $($role,)* Other }
        impl From<egui::accesskit::Role> for ControlRole {
            fn from(role: egui::accesskit::Role) -> Self {
                match role {
                    $(egui::accesskit::Role::$role => Self::$role,)*
                    _ => Self::Other,
                }
            }
        }
    };
}
roles!(
    Button,
    DefaultButton,
    Label,
    Image,
    Link,
    TextInput,
    MultilineTextInput,
    SearchInput,
    PasswordInput,
    NumberInput,
    CheckBox,
    RadioButton,
    Switch,
    Slider,
    SpinButton,
    ComboBox,
    ListBox,
    ListBoxOption,
    List,
    ListItem,
    Menu,
    MenuItem,
    Tab,
    TabList,
    TabPanel,
    Window,
    Dialog,
    Group,
    ScrollView,
    Splitter,
    Table,
    Row,
    Cell,
    ProgressIndicator,
    Canvas,
    DisclosureTriangle
);

pub(crate) fn rect_array(rect: Rect) -> [f32; 4] {
    [rect.min.x, rect.min.y, rect.max.x, rect.max.y]
}
fn serialize_id<S: serde::Serializer>(id: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&id.to_string())
}

fn accessible_name(
    id: u64,
    nodes: &HashMap<u64, &egui::accesskit::Node>,
    visited: &mut Vec<u64>,
) -> String {
    if visited.contains(&id) {
        return String::new();
    }
    visited.push(id);
    let Some(node) = nodes.get(&id) else {
        return String::new();
    };
    if let Some(label) = node.label() {
        return label.to_owned();
    }
    if node.role() == egui::accesskit::Role::Label {
        return node.value().unwrap_or_default().to_owned();
    }
    node.labelled_by()
        .iter()
        .map(|id| accessible_name(id.0, nodes, visited))
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) struct Observer(pub Sandbox);
impl egui::plugin::Plugin for Observer {
    fn debug_name(&self) -> &'static str {
        "egui-web-sandbox"
    }
    fn input_hook(&mut self, _ctx: &Context, input: &mut egui::RawInput) {
        // The web bridge suppresses the browser's clipboard default action. Keep
        // real key events for application shortcuts and add standard egui events.
        if !input.focused {
            self.0.0.lock().clipboard_events.clear();
            return;
        }
        input
            .events
            .extend(std::mem::take(&mut self.0.0.lock().clipboard_events));
        let mut clipboard = Vec::new();
        for event in &input.events {
            if let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
                && (modifiers.command || modifiers.ctrl)
                && !modifiers.alt
                && !modifiers.shift
            {
                match key {
                    egui::Key::C
                        if !input
                            .events
                            .iter()
                            .any(|event| matches!(event, egui::Event::Copy)) =>
                    {
                        clipboard.push(egui::Event::Copy)
                    }
                    egui::Key::X
                        if !input
                            .events
                            .iter()
                            .any(|event| matches!(event, egui::Event::Cut)) =>
                    {
                        clipboard.push(egui::Event::Cut)
                    }
                    egui::Key::V
                        if !input.events.iter().any(|event| {
                            matches!(event, egui::Event::Paste(_) | egui::Event::PasteImage(_))
                        }) =>
                    {
                        if let Some(text) = self.0.text() {
                            clipboard.push(egui::Event::Paste(text));
                        }
                    }
                    _ => {}
                }
            }
        }
        input.events.extend(clipboard);
    }
    fn output_hook(&mut self, ctx: &Context, output: &mut egui::FullOutput) {
        let mut state = self.0.0.lock();
        // Copy commands stay in this instance's memory clipboard.
        output.platform_output.commands.retain(|command| {
            match command {
                egui::OutputCommand::CopyText(text) => {
                    state.text = Some(text.clone());
                    state.image = None;
                }
                egui::OutputCommand::CopyImage(image) => {
                    state.image = Some(image.clone());
                    state.text = None;
                }
                _ => return true,
            }
            false
        });
        if let Some(tree) = &output.platform_output.accesskit_update {
            let parents = tree
                .nodes
                .iter()
                .flat_map(|(id, node)| node.children().iter().map(move |child| (child.0, id.0)))
                .collect::<HashMap<_, _>>();
            let nodes = tree
                .nodes
                .iter()
                .map(|(id, node)| (id.0, node))
                .collect::<HashMap<_, _>>();
            let window_of = |mut id: u64| {
                for _ in 0..=nodes.len() {
                    if nodes.get(&id).is_some_and(|node| {
                        node.bounds().is_some()
                            && matches!(
                                node.role(),
                                egui::accesskit::Role::Window | egui::accesskit::Role::Dialog
                            )
                    }) {
                        return Some(id);
                    }
                    id = *parents.get(&id)?;
                }
                None
            };
            state.snapshot.controls = tree
                .nodes
                .iter()
                .filter_map(|(id, node)| {
                    let bounds = node.bounds()?;
                    if node.is_hidden() || node.role() == egui::accesskit::Role::TextRun {
                        return None;
                    }
                    Some(Control {
                        id: id.0.to_string(),
                        role: node.role().into(),
                        label: accessible_name(id.0, &nodes, &mut Vec::new()),
                        value: node.value().map(str::to_owned),
                        bounds: [bounds.x0, bounds.y0, bounds.x1, bounds.y1],
                        disabled: node.is_disabled(),
                        parent: parents.get(&id.0).map(u64::to_string),
                        window: window_of(id.0).map(|id| id.to_string()),
                        focused: tree.focus == *id,
                        checked: node.toggled().and_then(|value| match value {
                            egui::accesskit::Toggled::True => Some(true),
                            egui::accesskit::Toggled::False => Some(false),
                            egui::accesskit::Toggled::Mixed => None,
                        }),
                        mixed: node.toggled() == Some(egui::accesskit::Toggled::Mixed),
                        selected: node.is_selected(),
                        expanded: node.is_expanded(),
                        numeric_value: node.numeric_value(),
                        min: node.min_numeric_value(),
                        max: node.max_numeric_value(),
                    })
                })
                .collect();
            state.snapshot.controls.sort_by(|a, b| a.id.cmp(&b.id));
            state.snapshot.focused_control = Some(tree.focus.0.to_string());
            state.snapshot.focused_window = state
                .focused_window
                .unwrap_or_else(|| window_of(tree.focus.0))
                .map(|id| id.to_string());
            state.snapshot.windows = tree
                .nodes
                .iter()
                .filter_map(|(id, node)| {
                    matches!(
                        node.role(),
                        egui::accesskit::Role::Window | egui::accesskit::Role::Dialog
                    )
                    .then_some(())?;
                    let bounds = node.bounds()?;
                    Some(WindowSnapshot {
                        id: id.0,
                        title: node.label().unwrap_or_default().to_owned(),
                        bounds: [
                            bounds.x0 as f32,
                            bounds.y0 as f32,
                            bounds.x1 as f32,
                            bounds.y1 as f32,
                        ],
                    })
                })
                .collect();
        }
        let windows = state.windows.clone();
        state.snapshot.windows.extend(windows);
        state.snapshot.windows.sort_by_key(|window| window.id);
        state.snapshot.bounds = rect_array(ctx.viewport_rect());
        state.snapshot.frame = ctx.cumulative_frame_nr();
        state.snapshot.schema_version = PROTOCOL_VERSION;
        state.snapshot.generation = state.generation;
        state.snapshot.pixels_per_point = ctx.pixels_per_point();
        state.snapshot.zoom_factor = ctx.zoom_factor();
        state.snapshot.clipboard_image_size = state
            .image
            .as_ref()
            .map(|image| [image.width(), image.height()]);
        state.snapshot.clipboard_has_text = state.text.is_some();
        state.snapshot.ready = true;
        state.snapshot.reset_supported = state.reset_enabled;
    }
}
