//! Optional simulation for applications with custom native window commands.
//! Ordinary egui windows already provide their own movement and resizing.
use crate::{Sandbox, WindowSnapshot};
use egui::{Context, LayerId, Pos2, Rect, ViewportCommand};
use std::sync::Arc;

#[derive(Clone)]
pub struct Desktop(Arc<egui::mutex::Mutex<State>>, Sandbox);

#[derive(Default)]
struct State {
    focused: Option<u64>,
    gesture: Option<Gesture>,
    windows: Vec<ObservedWindow>,
}

struct ObservedWindow {
    view: WindowSnapshot,
    layer: LayerId,
    focusable: bool,
}

struct Gesture {
    window: u64,
    origin: Pos2,
    rect: Rect,
    resize: Option<egui::ResizeDirection>,
    step: f32,
}

impl Desktop {
    pub fn new(sandbox: Sandbox) -> Self {
        Self(Default::default(), sandbox)
    }

    pub fn sandbox(&self) -> &Sandbox {
        &self.1
    }

    /// Start a UI pass. Modal ownership is supplied by the application.
    pub fn begin_frame(&self, ctx: &Context, blocked: bool) {
        let (focused, pressed, pointer) = ctx.input(|input| {
            (
                input.focused,
                input.pointer.any_pressed(),
                input.pointer.latest_pos(),
            )
        });
        let hit = pointer.and_then(|point| ctx.layer_id_at(point).map(|layer| (point, layer)));
        let mut state = self.0.lock();
        if !focused {
            state.focused = None;
        } else if pressed && !blocked {
            state.focused = hit.and_then(|(point, layer)| {
                state
                    .windows
                    .iter()
                    .rev()
                    .find(|window| {
                        window.focusable
                            && window.layer == layer
                            && window.view.rect().contains(point)
                    })
                    .map(|window| window.view.id)
            });
        }
        if !focused || blocked {
            state.gesture = None;
        }
        state.windows.clear();
        let mut host = self.1.0.lock();
        host.windows.clear();
        host.focused_window = Some(state.focused);
    }

    pub fn observe_window(
        &self,
        id: u64,
        title: impl Into<String>,
        rect: Rect,
        layer: LayerId,
        focusable: bool,
    ) {
        let title = title.into();
        self.0.lock().windows.push(ObservedWindow {
            view: WindowSnapshot {
                id,
                title: title.clone(),
                bounds: crate::observe::rect_array(rect),
            },
            layer,
            focusable,
        });
        self.1.0.lock().windows.push(WindowSnapshot {
            id,
            title,
            bounds: crate::observe::rect_array(rect),
        });
    }

    pub fn focused(&self) -> Option<u64> {
        self.0.lock().focused
    }
    pub fn focus(&self, id: Option<u64>) {
        self.0.lock().focused = id;
        self.1.0.lock().focused_window = Some(id);
    }
    pub fn forget_window(&self, id: u64) {
        self.cancel_gesture(id);
        let mut state = self.0.lock();
        state.windows.retain(|window| window.view.id != id);
        if state.focused == Some(id) {
            state.focused = None;
        }
        let mut host = self.1.0.lock();
        host.windows.retain(|window| window.id != id);
        host.focused_window = Some(state.focused);
    }
    pub fn cancel_gesture(&self, id: u64) {
        let mut state = self.0.lock();
        if state
            .gesture
            .as_ref()
            .is_some_and(|gesture| gesture.window == id)
        {
            state.gesture = None;
        }
    }

    /// Apply only delivered pointer input to the application's current rectangle.
    pub fn update_window(&self, ctx: &Context, id: u64, rect: &mut Rect, enabled: bool) {
        let (focused, down, pointer) = ctx.input(|input| {
            (
                input.focused,
                input.pointer.primary_down(),
                input.pointer.interact_pos(),
            )
        });
        let mut state = self.0.lock();
        let Some(gesture) = state
            .gesture
            .as_ref()
            .filter(|gesture| gesture.window == id)
        else {
            return;
        };
        if enabled
            && focused
            && let Some(pointer) = pointer
        {
            *rect = gesture.rect_at(pointer);
        }
        if !enabled || !focused || !down {
            state.gesture = None;
        }
    }

    /// Translate a native-style drag/resize request into a virtual gesture.
    pub fn window_command(
        &self,
        ctx: &Context,
        id: u64,
        rect: Rect,
        command: ViewportCommand,
        enabled: bool,
    ) {
        if !enabled {
            self.cancel_gesture(id);
            return;
        }
        let resize = match command {
            ViewportCommand::StartDrag => None,
            ViewportCommand::BeginResize(direction) => Some(direction),
            _ => return,
        };
        let Some(origin) = ctx.input(|input| {
            (input.focused && input.pointer.primary_down())
                .then(|| input.pointer.press_origin())
                .flatten()
        }) else {
            return;
        };
        self.0.lock().gesture = Some(Gesture {
            window: id,
            origin,
            rect,
            resize,
            step: ctx.pixels_per_point().recip(),
        });
    }

    /// Clear virtual window input; application and sandbox state have separate lifetimes.
    pub fn reset(&self) {
        *self.0.lock() = State::default();
        let mut host = self.1.0.lock();
        host.windows.clear();
        host.focused_window = None;
    }
}

impl Gesture {
    fn rect_at(&self, pointer: Pos2) -> Rect {
        let delta = pointer - self.origin;
        let Some(direction) = self.resize else {
            return self.rect.translate(delta);
        };
        use egui::ResizeDirection::*;
        let axes = match direction {
            West => [-1, 0],
            East => [1, 0],
            North => [0, -1],
            South => [0, 1],
            NorthWest => [-1, -1],
            NorthEast => [1, -1],
            SouthWest => [-1, 1],
            SouthEast => [1, 1],
        };
        let mut rect = self.rect;
        for axis in 0..2 {
            if axes[axis] < 0 {
                rect.min[axis] = (rect.min[axis] + delta[axis]).min(rect.max[axis] - self.step);
            }
            if axes[axis] > 0 {
                rect.max[axis] = (rect.max[axis] + delta[axis]).max(rect.min[axis] + self.step);
            }
        }
        rect
    }
}
