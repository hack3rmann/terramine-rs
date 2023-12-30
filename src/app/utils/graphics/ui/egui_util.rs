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
    Console,
    Inspector,
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
            "Console" => Self::Console,
            "Inspector" => Self::Inspector,
            _ => Self::Other { name: name.to_string() },
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Properties => "Properties",
            Self::Profiler => "Profiler",
            Self::Console => "Console",
            Self::Inspector => "Inspector",
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
        use egui_dock::{DockState, SurfaceIndex, NodeIndex};

        let mut result = DockState::new(vec![Tab::Properties, Tab::Profiler, Tab::Inspector]);

        let tree = result.get_surface_mut(SurfaceIndex::main())
            .expect("main surface should exist")
            .node_tree_mut()
            .expect("node tree borrowed only once");

        tree.split_below(NodeIndex::root(), 0.6, vec![Tab::Console]);

        Self::new(result)
    }
}



#[derive(Debug, Clone, Deref)]
pub struct EguiContext {
    #[deref]
    pub(crate) ctx: egui::Context,
    pub(crate) dock: EguiDockState,
}
assert_impl_all!(EguiContext: Send, Sync);

impl EguiContext {
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



pub struct DragValue3<'src, 'str, T> {
    src: &'src mut T,
    label: egui::WidgetText,
    x_prefix: StrView<'str>,
    y_prefix: StrView<'str>,
    z_prefix: StrView<'str>,
}
assert_impl_all!(DragValue3<i32>: Send, Sync);

impl<'src, 'str, T> DragValue3<'src, 'str, T> {
    pub fn new(src: &'src mut T) -> Self {
        Self {
            src,
            label: "".into(),
            x_prefix: "x: ".into(),
            y_prefix: "y: ".into(),
            z_prefix: "z: ".into(),
        }
    }

    pub fn label(mut self, label: impl Into<egui::WidgetText>) -> Self {
        self.label = label.into();
        self
    }

    pub fn x_prefix(mut self, value: impl Into<StrView<'str>>) -> Self {
        self.x_prefix = value.into();
        self
    }

    pub fn y_prefix(mut self, value: impl Into<StrView<'str>>) -> Self {
        self.y_prefix = value.into();
        self
    }

    pub fn z_prefix(mut self, value: impl Into<StrView<'str>>) -> Self {
        self.z_prefix = value.into();
        self
    }

    pub fn prefixes(
        mut self,
        x_prefix: impl Into<StrView<'str>>,
        y_prefix: impl Into<StrView<'str>>,
        z_prefix: impl Into<StrView<'str>>,
    ) -> Self {
        self = self.x_prefix(x_prefix);
        self = self.y_prefix(y_prefix);
        self = self.z_prefix(z_prefix);
        self
    }
}

impl egui::Widget for DragValue3<'_, '_, vec3> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label(self.label)
                | ui.add(egui::DragValue::new(&mut self.src.x).prefix(self.x_prefix))
                | ui.add(egui::DragValue::new(&mut self.src.y).prefix(self.y_prefix))
                | ui.add(egui::DragValue::new(&mut self.src.z).prefix(self.z_prefix))
        }).inner
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
        let transform_response = ui.add(
            DragValue3::new(&mut self.transform.translation.position).label("Position")
        );
    
        let rotation_response = ui.add(
            DragValue3::new(&mut self.transform.rotation.angles)
                .label("Rotation")
                .prefixes("roll: ", "pitch: ", "yaw: ")
        );

        let scaling_response = ui.add(
            DragValue3::new(&mut self.transform.scaling.amount).label("Scaling")
        );

        transform_response | rotation_response | scaling_response
    }
}



#[profile]
pub fn run_inspector(world: &World, ui: &mut egui::Ui) {
    use crate::{
        components::Name,
        graphics::mesh::{Mesh, GpuMesh},
        physics::PhysicalComponent,
        camera::CameraComponent,
    };
    
    let mut name_query = world.query::<(
        Option<&mut Name>,
        Option<&mut Transform>,
        Option<&mut Mesh>,
        Option<&mut PhysicalComponent>,
        Option<&mut Timer>,
    )>();

    for (entity, (name, transform, mesh, phys, timer)) in name_query.iter() {
        let heading = match name {
            None => format!("entity {}", entity.id()),
            Some(name) => format!("{} (entity {})", name.value, entity.id()),
        };

        ui.collapsing(heading, |ui| {
            if let Ok(mut camera) = world.get::<&mut CameraComponent>(entity) {
                ui.collapsing("CameraComponent", |ui| camera.ui(ui));
            }

            if let Some(transform) = transform {
                ui.collapsing("Transform", |ui| {
                    ui.add(TransformWidget::new(transform));
                });
            }

            if let Some(mesh) = mesh {
                ui.collapsing(format!("Mesh ({})", Bytes(mesh.size())), |ui| {
                    ui.label(egui::RichText::new(
                        format!("Mesh: {mesh:#?}")
                    ).monospace());
                });
            }

            if let Ok(mesh) = world.get::<&mut GpuMesh>(entity) {
                ui.collapsing("GpuMesh", |ui| {
                    ui.label(egui::RichText::new(
                        format!("{:#?}", mesh)
                    ).monospace());
                });
            }

            if let Ok(mut loader) = world.get::<&mut AssetLoader>(entity) {
                ui.collapsing("AssetManager", |ui| {
                    static NAME: RwLock<String> = const_default();
                    let mut name = NAME.write();

                    if ui.text_edit_singleline(name.deref_mut()).lost_focus()
                        && ui.input(|s| s.key_pressed(egui::Key::Enter))
                    {
                        loader.start_loading(
                            name.deref(),
                            |bytes| Ok(String::from_utf8(bytes)?),
                        );
                    }

                    ui.label("Loading:");

                    for path in loader.unloaded.keys() {
                        ui.label(path.as_os_str().to_string_lossy());
                    }

                    ui.separator();

                    ui.label("Finished");

                    for (path, asset) in loader.loaded.iter() {
                        ui.label(format!("{asset:?}"));

                        ui.collapsing(path.as_os_str().to_string_lossy(), |ui| {
                            if let Some(string) = asset.get::<String>() {
                                ui.label(egui::RichText::new(string).monospace());
                            }
                        });
                    }
                });
            }

            if let Some(phys) = phys {
                ui.collapsing("PhysicalComponent", |ui| {
                    ui.label(egui::RichText::new(
                        format!("{phys:#?}")
                    ).monospace());
                });
            }

            if let Some(timer) = timer {
                ui.collapsing("Timer", |ui| {
                    ui.label(egui::RichText::new(
                        timer.to_string()
                    ).monospace());
                });
            }
        });
    }
}



pub trait ShowUi {
    fn ui(&mut self, ui: &mut egui::Ui);
}
assert_obj_safe!(ShowUi);