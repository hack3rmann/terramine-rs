pub mod camera_resource;
pub mod debug_visuals;
pub mod ui;
pub mod light;
pub mod mesh;
pub mod render_resource;
pub mod pipeline;
pub mod pass;
pub mod gpu_conversions;
pub mod bind_group_old;
pub mod bind_group;
pub mod buffer;
pub mod texture;
pub mod sprite;
pub mod shader;
pub mod image;
pub mod asset;
pub mod material;
pub mod render_graph;
pub mod postproc;
pub mod gbuffer;

use crate::prelude::*;



pub use {
    material::*, gpu_conversions::*, pass::*,
    bind_group::*, buffer::*, texture::*, window::Window,
    mesh::*, shader::*, asset::*, pipeline::*, self::image::*,
    camera::*, render_graph::*,

    wgpu::{
        SurfaceError, CommandEncoder, Features,
        DeviceDescriptor, RequestAdapterOptions, Backends, InstanceDescriptor,
        Instance, SurfaceConfiguration, Adapter, Surface, Queue,
        BufferBindingType, BindingType, Device,
        util::{DeviceExt, BufferInitDescriptor}, Extent3d,
    },
    winit::{event_loop::EventLoop, event::Event},

    egui_wgpu_backend::{RenderPass as EguiRenderPass, ScreenDescriptor as EguiScreenDescriptor},
    ui::egui_util::{EguiContext, EguiDockState, Tab},
};



#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    async fn init(self, world: &mut World) -> AnyResult<()> {
        world.init_resource::<PipelineCache>();
        world.init_resource::<BindsCache>();
        world.init_resource::<RenderGraph>();
        world.insert_resource(CommonUniform::zeroed());

        let graphics = Graphics::new()
            .await
            .context("failed to create graphics")?;
        
        world.insert_resource(graphics);

        Ok(())
    }
}



#[derive(Debug)]
pub struct RenderContext<'window> {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub adapter: Adapter,
    pub surface: Surface<'window>,
}
assert_impl_all!(RenderContext: Send, Sync);

impl RenderContext<'_> {
    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}



pub static SURFACE_CFG: RwLock<GlobalAsset<SurfaceConfiguration>> = const_default();



#[derive(Debug)]
pub struct RenderStage {
    pub depth: GpuImage,
    pub encoder: Option<CommandEncoder>,
    pub surface_view: Option<TextureView>,
    pub surface_texture: Option<SurfaceTexture>,
}
assert_impl_all!(RenderStage: Send, Sync);

impl RenderStage {
    pub fn new(device: &Device) -> Self {
        let depth = Self::make_depth_image(device, {
            let cfg = SURFACE_CFG.read();
            UInt2::new(cfg.width, cfg.height)
        });

        Self {
            depth,
            surface_view: None,
            encoder: None,
            surface_texture: None,
        }
    }

    pub fn surface_view(&self) -> &TextureView {
        self.surface_view.as_ref()
            .expect("failed to get surface view")
    }

    pub fn encoder_mut(&mut self) -> &mut CommandEncoder {
        self.encoder.as_mut()
            .expect("failed to get encoder")
    }

    pub fn get_all(&mut self) -> (&TextureView, &TextureView, &mut CommandEncoder) {
        (
            self.surface_view.as_ref()
                .expect("failed to get surface view"),
            &self.depth.view,
            self.encoder.as_mut()
                .expect("failed to get command encoder"),
        )
    }

    pub fn encoder_and_view(&mut self) -> (&TextureView, &mut CommandEncoder) {
        let (surface_view, _, encoder) = self.get_all();
        (surface_view, encoder)
    }

    pub fn begin(
        &mut self, surface_texture: SurfaceTexture, encoder: CommandEncoder,
    ) {
        self.surface_view = Some(surface_texture.texture.create_view(&default()).into());
        self.surface_texture = Some(surface_texture);
        self.encoder = Some(encoder);
    }

    pub fn finish(&mut self) -> Option<(SurfaceTexture, CommandEncoder)> {
        self.surface_view = None;

        Some((
            self.surface_texture.take()?,
            self.encoder.take()?,
        ))
    }

    fn on_window_resize(&mut self, device: &Device, new_size: UInt2) {
        self.depth = Self::make_depth_image(device, new_size);
    }

    fn make_depth_image(device: &Device, size: UInt2) -> GpuImage {
        let size = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&default());

        let sampler = Sampler::new(device, &default());

        GpuImage {
            texture: texture.into(),
            sampler,
            view: view.into(),
            label: Some("depth".into()),
        }
    }
}



/// Graphics handler.
#[derive(Debug)]
pub struct Graphics {
    /// Wraps [queue][Queue], [device][Device], [adapter][Adapter] and render [surface][Surface].
    pub context: RenderContext<'static>,
    
    /// A [window][Window] handle.
    pub window: Window,

    /// Holds rendering information during rendering proccess.
    pub render_stage: RenderStage,

    /// `egui` handle.
    pub egui: Egui,
}
assert_impl_all!(Graphics: Send, Sync, Component);

impl Graphics {
    pub fn get_device(&self) -> Arc<Device> {
        Arc::clone(&self.context.device)
    }

    pub fn get_queue(&self) -> Arc<Queue> {
        Arc::clone(&self.context.queue)
    }

    pub fn device(&self) -> &Device {
        self.context.device()
    }

    pub fn queue(&self) -> &Queue {
        self.context.queue()
    }

    /// Creates new [`Graphics`] that holds some renderer stuff.
    pub async fn new() -> AnyResult<Self> {
        logger::scope!(from = "graphics", "new()");

        let window = Window::new(cfg::window::default::SIZES)
            .context("failed to initialize a window")?;

        let (context, config) = Self::make_render_context(&window).await;

        // # Safety
        // 
        // `Graphics` owns both the `window` and the `surface` so it's
        // live as long as wgpu's `Surface`.        
        let context: RenderContext<'static> = unsafe { std::mem::transmute(context) };

        // # Safety
        // 
        // We have unique access to `SURFACE_CFG` so it is safe.
        unsafe { SURFACE_CFG.write().upload(config) };

        let egui = Egui::new(
            context.device(),
            window.inner_size().to_vec2(),
            window.scale_factor(),
        );
        
        Ok(Self {
            egui,
            window,
            render_stage: RenderStage::new(&context.device),
            context,
        })
    }

    /// Initializes a `wgpu` render context.
    /// 
    /// # Safety
    ///
    /// - `window` must be a valid object to create a surface upon.
    /// - `window` must remain valid until after
    /// the returned [`RenderContext`] is dropped.
    async fn make_render_context(window: &Window)
        -> (RenderContext, SurfaceConfiguration)
    {
        let wgpu_instance = Instance::new(
            InstanceDescriptor {
                backends: Backends::DX12 | Backends::VULKAN,
                dx12_shader_compiler: default(),
                // TODO: choose specific flags
                flags: wgpu::InstanceFlags::all(),
                // TODO: specify minor version
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            }
        );

        let surface = wgpu_instance.create_surface(&window.inner)
            .expect("context should not be WebGL2");

        let adapter = wgpu_instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface)
            })
            .await
            .expect("failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: default(),
                },
                None,
            )
            .await
            .expect("failed to create device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = *swapchain_capabilities.formats.first()
            .expect(
                "failed to get swap chain format 0: the
                surface is incompatible with the adapter"
            );

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: swapchain_capabilities.present_modes[0],
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            // FIXME: figure out what is this
            desired_maximum_frame_latency: 0,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        (
            RenderContext { device, queue, adapter, surface },
            config
        )
    }

    pub fn ui(world: &World, context: EguiContext) {
        use egui_dock::DockArea;

        let mut dock = context.dock.value.write();
        let width = context.ctx().available_rect().width();

        egui::SidePanel::right("right")
            .default_width(0.25 * width)
            .min_width(0.2 * width)
            .max_width(0.3 * width)
            .show(context.ctx(), |ui| {
                DockArea::new(&mut dock)
                    .style(egui_dock::Style::from_egui(&context.ctx().style()))
                    .show_inside(ui, &mut EguiTabViewer { world });
            });
    }

    pub fn render_egui(&mut self, world: &World)
        -> Result<(), egui_wgpu_backend::BackendError>
    {
        let (surface_view, encoder) = self.render_stage.encoder_and_view();

        self.egui.begin_frame();

        Self::ui(world, self.egui.context());

        let full_output = self.egui.end_frame(&self.window);

        self.egui.render(
            &self.context.device,
            &self.context.queue,
            encoder,
            surface_view,
            full_output,
            self.window.scale_factor() as f32
        )?;

        Ok(())
    }

    pub fn render(&mut self, world: &World) -> AnyResult<()> {
        self.begin_render()?;

        if let Ok(graph) = world.resource::<&RenderGraph>() {
            let (view, depth, encoder) = self.render_stage.get_all();
            
            graph.run(world, encoder, &[view.clone()], Some(depth));
        } else {
            logger::error!(
                from = "graphics",
                "failed to draw: render graph not found",
            );
        }

        self.render_egui(world)?;

        self.finish_render()?;

        Ok(())
    }

    /// Begins a render proccess.
    pub fn begin_render(&mut self) -> Result<(), SurfaceError> {
        let output = self.context.surface.get_current_texture()?;

        let encoder = self.context.device.create_command_encoder(&default());

        self.render_stage.begin(output, encoder);

        let (surface_view, depth, encoder) = self.render_stage.get_all();

        ClearPass::clear(
            encoder,
            [surface_view],
            Some(depth),
            cfg::shader::CLEAR_COLOR,
            Some(1.0),
        );

        Ok(())
    }

    /// Ends rendering proccess.
    pub fn finish_render(&mut self) -> AnyResult<()> {
        let (surface_texture, encoder) = self.render_stage.finish()
            .context(
                "`finish_render` should be called only once
                and `begin_render` should be called before"
            )?;

        self.context.queue.submit([encoder.finish()]);

        surface_texture.present();

        Ok(())
    }

    /// Handles window resize event by [`Graphics`].
    pub fn on_window_resize(&mut self, new_size: UInt2) {
        if new_size.x > 0 && new_size.y > 0 {
            let mut config = SURFACE_CFG.write();
            (config.width, config.height) = (new_size.x, new_size.y);
            self.context.surface.configure(&self.context.device, &config);
        }

        self.render_stage.on_window_resize(&self.context.device, new_size);
    }

    pub fn handle_event(world: &World, event: &Event<()>) -> AnyResult<()> {
        use winit::event::WindowEvent;

        let mut graphics = world.resource::<&mut Self>()?;

        graphics.egui.platform.handle_event(event);

        if let Event::WindowEvent {
            event: WindowEvent::Resized(new_size), ..
        } = event {
            let new_size = new_size.to_vec2();

            graphics.on_window_resize(new_size);

            let mut query = world.query::<&mut CameraComponent>();
            for (_entity, camera) in query.into_iter() {
                camera.on_window_resize(new_size);
            }
        }

        Ok(())
    }

    pub fn update(world: &mut World) -> AnyResult<()> {
        mesh::GpuMesh::make_renderable(world);

        let graphics = world.resource::<&mut Self>()?;
        let timer = world.resource::<&Timer>()?;

        keyboard::set_input_capture(graphics.egui.context().wants_keyboard_input());

        if let Ok(mut common_uniform) = world.resource::<&mut CommonUniform>() {
            common_uniform.screen_resolution
                = graphics.window.inner_size().to_vec2().into();

            common_uniform.time = timer.time().as_secs_f32();

            let mut cache = world.resource::<&mut BindsCache>()
                .expect("failed to found binds cache");

            if let Some(common) = cache.get_mut::<CommonUniform>() {
                AsBindGroup::update(
                    common_uniform.deref(),
                    graphics.device(),
                    graphics.queue(),
                    common,
                );
            } else {
                let entries = CommonUniform::bind_group_layout_entries(graphics.device());
                let layout = CommonUniform::bind_group_layout(graphics.device(), &entries);
                let bind_group = common_uniform.as_bind_group(graphics.device(), &layout)?;

                cache.add(bind_group);
            }
        }

        Ok(())
    }

    pub fn prepare_frame(&mut self, fps: f32) {
        self.window.set_title(&format!("Terramine: {fps:.0} FPS"));
    }
}



#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CommonUniform {
    pub screen_resolution: vec2,
    pub time: f32,
    pub _padding: u32,
}
assert_impl_all!(CommonUniform: Send, Sync);

impl AsBindGroup for CommonUniform {
    type Data = Self;

    fn label() -> Option<&'static str> {
        Some("common_uniform")
    }

    fn cache_key() -> &'static str {
        "common"
    }

    fn update(
        &self, _: &Device, queue: &Queue,
        bind_group: &mut PreparedBindGroup<Self::Data>,
    ) -> bool {
        use crate::graphics::*;

        bind_group.unprepared.data = *self;

        for (index, resource) in bind_group.unprepared.bindings.iter() {
            if *index == 0 {
                let OwnedBindingResource::Buffer(buffer)
                    = resource else { return false };
                
                queue.write_buffer(buffer, 0, bytemuck::bytes_of(self));
                queue.submit(None);
            }
        }

        true
    }

    fn bind_group_layout_entries(_: &Device) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]
    }

    fn unprepared_bind_group(
        &self, device: &Device, _: &BindGroupLayout,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        let buffer = Buffer::new(device, &BufferInitDescriptor {
            label: Self::label(),
            contents: bytemuck::bytes_of(self),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Ok(UnpreparedBindGroup {
            data: *self,
            bindings: vec![(0, OwnedBindingResource::Buffer(buffer))]
        })
    }
}



pub struct Egui {
    render_pass: egui_wgpu_backend::RenderPass,
    platform: egui_winit_platform::Platform,
    dock_state: EguiDockState,
}
assert_impl_all!(Egui: Send, Sync);

impl Egui {
    pub fn new(device: &Device, window_sizes: UInt2, scale_factor: f64) -> Self {
        Self {
            render_pass: egui_wgpu_backend::RenderPass::new(
                device, SURFACE_CFG.read().format, 1
            ),
            platform: egui_winit_platform::Platform::new(
                egui_winit_platform::PlatformDescriptor {
                    physical_width: window_sizes.x,
                    physical_height: window_sizes.y,
                    scale_factor,
                    font_definitions: default(),
                    style: default(),
                },
            ),
            dock_state: default(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.platform.begin_frame();
    }

    pub fn end_frame(&mut self, window: &Window) -> egui::FullOutput {
        // `end_frame` changes cursor visibility to change its icon.
        // So we can't provide vindow during capturing.
        self.platform.end_frame(
            (!mouse::is_captured()).then_some(window)
        )
    }

    pub fn context(&self) -> EguiContext {
        EguiContext::new(self.platform.context(), self.dock_state.clone())
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        surface_view: &TextureView,
        full_output: egui::FullOutput,
        scale_factor: f32,
    ) -> Result<(), egui_wgpu_backend::BackendError> {
        let paint_jobs = self.context().tessellate(full_output.shapes, full_output.pixels_per_point);

        let (width, height) = {
            let cfg = SURFACE_CFG.read();
            (cfg.width, cfg.height)
        };

        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };

        self.render_pass.add_textures(device, queue, &full_output.textures_delta)?;
        self.render_pass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        self.render_pass.execute(
            encoder, surface_view, &paint_jobs, &screen_descriptor, None,
        )?;

        Ok(())
    }
}

impl std::fmt::Debug for Egui {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Egui {{ dock_state: {:?}, .. }}", self.dock_state)
    }
}



#[derive(Clone, Copy, Debug)]
struct EguiTabViewer<'w> {
    world: &'w World,
}

impl egui_dock::TabViewer for EguiTabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Properties => {
                ui::egui_util::use_each_window_builder(ui);

                camera::CameraHandle::spawn_control_window(self.world, ui);
            },

            Tab::Profiler => {
                let time_step = self.world.resource::<&Timer>()
                    .expect("failed to find the timer")
                    .time_step();

                crate::profiler::update_and_build_window(time_step, ui);
            }

            Tab::Console => crate::logger::build_window(ui),

            Tab::Inspector => ui::egui_util::run_inspector(self.world, ui),

            Tab::Other { .. } => (),
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        matches!(tab, Tab::Other { .. })
    }
}