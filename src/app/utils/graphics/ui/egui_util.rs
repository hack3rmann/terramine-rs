use crate::{prelude::*, transform::Transform};



pub type WindowBuilder = fn(&mut egui::Ui);

pub static WINDOW_BUILDERS: Mutex<SmallVec<[WindowBuilder; 64]>>
    = const_default();

/// Adds a function to window builders list without aquireing [`Mutex`]'s lock.
pub fn push_window_builder(builder: WindowBuilder) {
    WINDOW_BUILDERS.lock().push(builder)
}

/// Adds a function to window builders list without aquireing [`Mutex`]'s lock.
/// 
/// # Safety
/// 
/// - should be called on main thread.
/// - there's no threads pushing update functions.
pub unsafe fn push_window_builder_lock_free(builder: WindowBuilder) {
    WINDOW_BUILDERS
        .data_ptr()
        .as_mut()
        .unwrap_unchecked()
        .push(builder);
}

/// Applies builders to `ui`.
pub fn use_each_window_builder(ui: &mut egui::Ui) {
    for build in WINDOW_BUILDERS.lock().iter() {
        build(ui)
    }
}



pub trait ShowDebugUi {
    fn show_debug_ui(&mut self, ctx: &mut egui::Context);
}
assert_obj_safe!(ShowDebugUi);



#[derive(Debug, Clone, Default, PartialEq)]
pub enum Tab {
    #[default]
    Properties,
    Profiler,
    Other {
        name: String,
    },
}
assert_impl_all!(Tab: Send, Sync);

impl Tab {
    pub fn new<'n>(name: impl Into<StrView<'n>>) -> Self {
        let name = name.into();

        match name.deref() {
            "Properties" => Self::Properties,
            "Profiler" => Self::Profiler,
            _ => Self::Other { name: name.to_string() },
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Properties => "Properties",
            Self::Profiler => "Profiler",
            Self::Other { name } => name,
        }
    }
}



#[derive(Clone, Debug)]
pub struct EguiDockState {
    pub(crate) value: Arc<RwLock<egui_dock::DockState<Tab>>>,
}
assert_impl_all!(EguiDockState: Send, Sync);

impl EguiDockState {
    pub fn new(value: egui_dock::DockState<Tab>) -> Self {
        Self { value: Arc::new(RwLock::new(value)) }
    }
}

impl Default for EguiDockState {
    fn default() -> Self {
        use egui_dock::DockState;

        Self::new(DockState::new(vec![]))
    }
}



#[derive(Debug, Clone)]
pub struct EguiContexts {
    pub(crate) ctx: egui::Context,
    pub(crate) dock: EguiDockState,
}
assert_impl_all!(EguiContexts: Send, Sync);

impl EguiContexts {
    pub fn new(ctx: egui::Context, dock: EguiDockState) -> Self {
        Self { ctx, dock }
    }

    pub fn ctx(&self) -> &egui::Context {
        &self.ctx
    }

    pub fn ctx_mut(&mut self) -> &mut egui::Context {
        &mut self.ctx
    }
}



#[derive(PartialEq, Debug)]
pub struct TransformWidget<'t> {
    pub transform: &'t mut Transform,
}
assert_impl_all!(TransformWidget: Send, Sync);

impl<'t> TransformWidget<'t> {
    pub fn new(transform: &'t mut Transform) -> Self {
        Self { transform }
    }
}

impl egui::Widget for TransformWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let transform_response = ui.horizontal(|ui| {
            ui.label("Position")
            | ui.add(egui::DragValue::new(&mut self.transform.translation.x).prefix("x: "))
            | ui.add(egui::DragValue::new(&mut self.transform.translation.y).prefix("y: "))
            | ui.add(egui::DragValue::new(&mut self.transform.translation.z).prefix("z: "))
        }).inner;
    
        let rotation_response = ui.horizontal(|ui| {
            ui.label("Rotation")
            | ui.add(egui::DragValue::new(&mut self.transform.rotation.angles.x).prefix("roll: "))
            | ui.add(egui::DragValue::new(&mut self.transform.rotation.angles.y).prefix("pitch: "))
            | ui.add(egui::DragValue::new(&mut self.transform.rotation.angles.z).prefix("yaw: "))
        }).inner;

        let scaling_response = ui.horizontal(|ui| {
            ui.label("Scaling")
            | ui.add(egui::DragValue::new(&mut self.transform.scaling.amount.x).prefix("x: "))
            | ui.add(egui::DragValue::new(&mut self.transform.scaling.amount.y).prefix("y: "))
            | ui.add(egui::DragValue::new(&mut self.transform.scaling.amount.z).prefix("z: "))
        }).inner;

        transform_response | rotation_response | scaling_response
    }
}