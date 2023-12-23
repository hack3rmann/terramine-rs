pub mod camera_resource;
pub mod debug_visuals;
pub mod ui;
pub mod light;
pub mod mesh;
pub mod render_resource;
pub mod pipeline;
pub mod pass;
pub mod gpu_conversions;
pub mod material;
pub mod bind_group;
pub mod buffer;
pub mod texture;
pub mod sprite;
pub mod shader;
pub mod asset;
pub mod image;

use crate::prelude::*;



pub use {
    material::*, gpu_conversions::*, pass::*,
    bind_group::*, buffer::*, texture::*, window::Window,
    mesh::*, shader::*, asset::*, pipeline::*, self::image::*,
    camera::*,

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



// FIXME: remove this
const TEST_VERTICES: &[TexturedVertex] = &[
    TexturedVertex::new(vecf!(-0.4, -0.5, 0.0), vecf!(0.0, 1.0)),
    TexturedVertex::new(vecf!( 0.4, -0.5, 0.0), vecf!(1.0, 1.0)),
    TexturedVertex::new(vecf!( 0.4,  0.5, 0.0), vecf!(1.0, 0.0)),
    TexturedVertex::new(vecf!(-0.4, -0.5, 0.0), vecf!(0.0, 1.0)),
    TexturedVertex::new(vecf!( 0.4,  0.5, 0.0), vecf!(1.0, 0.0)),
    TexturedVertex::new(vecf!(-0.4,  0.5, 0.0), vecf!(0.0, 0.0)),
];



#[derive(Debug)]
pub struct RenderContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub adapter: Adapter,
    pub surface: Surface,
}
assert_impl_all!(RenderContext: Send, Sync);



pub static SURFACE_CFG: RwLock<GlobalAsset<SurfaceConfiguration>> = const_default();



#[derive(Debug, Constructor)]
pub struct RenderStage {
    pub view: TextureView,
    pub depth: TextureView,
    pub encoder: CommandEncoder,
    pub output: SurfaceTexture,
}
assert_impl_all!(RenderStage: Send, Sync);



/// Graphics handler.
#[derive(Debug)]
pub struct Graphics {
    /// A [window][Window] handle.
    pub window: Window,

    /// Wraps [queue][Queue], [device][Device], [adapter][Adapter] and render [surface][Surface].
    pub context: RenderContext,

    /// Holds rendering information during render proccess.
    pub render_stage: Option<RenderStage>,

    /// Common shader uniforms buffer. Holds uniforms like time, screen resolution, etc.
    pub common_uniforms: CommonUniformsBuffer,

    /// `egui` handle.
    pub egui: Egui,

    pub sandbox: World,
}
assert_impl_all!(Graphics: Send, Sync, Component);

impl Graphics {
    /// Creates new [`Graphics`] that holds some renderer stuff.
    pub async fn new(event_loop: &EventLoop<()>) -> AnyResult<Self> {
        logger::scope!(from = "graphics", "new()");

        let window = Window::new(event_loop, cfg::window::default::SIZES)
            .context("failed to initialize a window")?;



        // -----------< WGPU initialization >-----------

        // * # Safety
        // * 
        // * `Graphics` owns both the `window` and the `surface` so it's
        // * live as long as wgpu's `Surface`.
        let (context, config) = unsafe { Self::make_render_context(&window) }.await;

        // * # Safety
        // * 
        // * We've got unique access to `SURFACE_CFG` so it is safe.
        unsafe { SURFACE_CFG.write().upload(config) };

        // * # Safety
        // * 
        // * This is graphics intialization so device does not exists anywhere before
        // * so assets can not been used before or been initialized.
        unsafe { asset::load_default_assets(&context.device) }.await;

        let common_uniforms = CommonUniformsBuffer::new(
            &context.device,
            &CommonUniforms {
                time: 0.0,
                screen_resolution: window.inner_size().to_vec2().into(),
                _padding: 0,
            },
        );



        // ------------ Renderng tests stuff ------------

        let mut sandbox = World::new();

        let image = Image::from_file("TerramineIcon32p.png")
            .await
            .expect("failed to load TerramineIcon32p.png image");

        let gpu_image = GpuImage::new(
            GpuImageDescriptor {
                device: &context.device,
                queue: &context.queue,
                image: &image,
                label: Some("test_image".into()),
            }
        );

        let gpu_image_bind_layout = GpuImage::bind_group_layout(&context.device);
        let gpu_image_bind_group = gpu_image.as_bind_group(&context.device, &gpu_image_bind_layout);

        let binds = Binds::from_iter([
            (gpu_image_bind_group, Some(gpu_image_bind_layout)),
        ]);

        let mesh = Mesh::new(
            TEST_VERTICES.to_vec(),
            None,
            PrimitiveTopology::TriangleList
        );

        let gpu_mesh = GpuMesh::new(
            GpuMeshDescriptor {
                mesh: &mesh,
                device: &context.device,
                label: Some("test_mesh".into()),
                polygon_mode: default(),
            },
        );

        let camera_uniform = camera::CameraUniformBuffer::new(
            &context.device, &default()
        );

        let layout = {
            let bind_group_layouts: Vec<_> = itertools::chain!(
                common_uniforms.binds.layouts(),
                binds.layouts(),
                camera_uniform.binds.layouts(),
            ).collect();

            PipelineLayout::new(
                &context.device,
                &PipelineLayoutDescriptor {
                    label: Some("test_pipeline_layout"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                },
            )
        };

        let material = {
            let source = ShaderSource::from_file("shader.wgsl")
                .await
                .expect("failed to load shader.wgsl from file");

            let shader = Shader::new(
                &context.device, source, vec![TexturedVertex::BUFFER_LAYOUT]
            );

            ShaderMaterial::from(shader).to_arc()
        };

        let pipeline = RenderPipeline::new(
            RenderPipelineDescriptor {
                device: &context.device,
                material: material.as_ref(),
                primitive_state: gpu_mesh.primitive_state,
                label: Some("test_render_pipeline".into()),
                layout: &layout,
            },
        );

        let binds_ref = BindsRef::from(binds);

        sandbox.spawn((gpu_mesh, pipeline, binds_ref, layout));
        sandbox.insert_resource(camera_uniform);



        // ----------------- Egui initialization -----------------

        let egui = Egui::new(
            &context.device,
            window.inner_size().to_vec2(),
            window.scale_factor(),
        );

        
        
        Ok(Self {
            egui,
            sandbox,
            window,
            context,
            common_uniforms,
            render_stage: None,
        })
    }

    /// Initializes a `wgpu` render context.
    /// 
    /// # Safety
    ///
    /// - `window` must be a valid object to create a surface upon.
    /// - `window` must remain valid until after
    /// the returned [`RenderContext`] is dropped.
    async unsafe fn make_render_context(window: &Window)
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
            .expect("context should be not WebGL2");

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
                    features: Features::empty(),
                    limits: default(),
                },
                None,
            )
            .await
            .expect("failed to create device");
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = *swapchain_capabilities.formats.get(0)
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
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        (
            RenderContext { device, queue, adapter, surface },
            config
        )
    }

    pub fn logo_pass(&mut self, cam: &CameraUniformBuffer) {
        let render_stage = self.render_stage.as_mut().unwrap();

        let mut query = self.sandbox.query::<(
            &BindsRef, &GpuMesh, &RenderPipeline
        )>();

        let mut pass = RenderPass::new(
            "logo_draw_pass",
            &mut render_stage.encoder,
            [&render_stage.view],
            Some(&render_stage.depth),
        );

        for (_entity, (binds, mesh, pipeline)) in query.into_iter() {
            Binds::bind_all(&mut pass, [
                &self.common_uniforms.binds,
                binds,
                &cam.binds,
            ]);
            
            let Ok(()) = mesh.render(pipeline, &mut pass);
        }
    }

    pub fn ui(world: &World, context: EguiContext) {
        use egui_dock::DockArea;

        let mut dock = context.dock.value.write();

        egui::SidePanel::right("right")
            .default_width(0.25 * context.ctx().available_rect().width())
            .min_width(0.2 * context.ctx().available_rect().width())
            .max_width(0.3 * context.ctx().available_rect().width())
            .show(context.ctx(), |ui| {
                DockArea::new(&mut dock)
                    .style(egui_dock::Style::from_egui(&context.ctx().style()))
                    .show_inside(ui, &mut EguiTabViewer { world });
            });
    }

    pub fn render_sandbox(&mut self) {
        // self.logo_pass(cam);
    }

    pub fn render_egui(&mut self, world: &World)
        -> Result<(), egui_wgpu_backend::BackendError>
    {
        let render_stage = self.render_stage.as_mut()
            .expect(
                "`render_sadbox` should be called after
                `begin_render` and before `finish_render`"
            );

        self.egui.begin_frame();

        Self::ui(world, self.egui.context());

        let full_output = self.egui.end_frame(&self.window);

        self.egui.render(
            &self.context.device,
            &self.context.queue,
            render_stage,
            full_output,
            self.window.scale_factor() as f32
        )?;

        Ok(())
    }

    pub fn render_with_sandbox<R>(
        &mut self, world: &World, render: R,
    ) -> AnyResult<()>
    where
        R: FnOnce(&Binds, &mut CommandEncoder, &TextureView, Option<&TextureView>, &World)
            -> AnyResult<()>,
    {
        let time = world.resource::<&Timer>()?.time();
        self.begin_render(time)?;

        {
            let render_stage = unsafe {
                self.render_stage.as_mut().unwrap_unchecked()
            };

            render(
                &self.common_uniforms.binds,
                &mut render_stage.encoder,
                &render_stage.view,
                Some(&render_stage.depth),
                world
            )?;
        }

        self.render_sandbox();
        self.render_egui(world)?;
        self.finish_render()?;

        Ok(())
    }

    pub fn render<R>(&mut self, time: Time, render: R) -> AnyResult<()>
    where
        R: FnOnce(&Binds, &mut CommandEncoder, &TextureView) -> AnyResult<()>,
    {
        self.begin_render(time)?;
        
        {
            // * Safety
            // * 
            // * Safe, because in `begin_render` `render_stage` is set to `Some`
            let render_stage = unsafe {
                self.render_stage.as_mut().unwrap_unchecked()
            };

            render(
                &self.common_uniforms.binds,
                &mut render_stage.encoder,
                &render_stage.view
            )?;
        }

        self.finish_render()?;

        Ok(())
    }

    /// Begins a render proccess.
    pub fn begin_render(&mut self, time: Time) -> Result<(), SurfaceError> {
        let size = self.window.inner_size();

        self.common_uniforms.update(&self.context.queue, CommonUniforms {
            time: time.as_secs_f32(),
            screen_resolution: vecf!(size.width, size.height),
            _padding: 0,
        });

        let depth = {
            let surface_cfg = SURFACE_CFG.read();

            let size = wgpu::Extent3d {
                width: surface_cfg.width,
                height: surface_cfg.height,
                depth_or_array_layers: 1,
            };

            let desc = wgpu::TextureDescriptor {
                label: Some("depth_texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };

            let texture = self.context.device.create_texture(&desc);

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            TextureView::from(view)
        };

        let output = self.context.surface.get_current_texture()?;

        let view = TextureView::from(
            output.texture.create_view(&default())
        );

        let mut encoder
            = self.context.device.create_command_encoder(&default());

        ClearPass::clear(
            &mut encoder, [&view], Some(&depth), cfg::shader::CLEAR_COLOR
        );

        self.render_stage = Some(
            RenderStage::new(view, depth, encoder, output.into())
        );

        Ok(())
    }

    /// Ends rendering proccess.
    pub fn finish_render(&mut self) -> AnyResult<()> {
        let render_stage = self.render_stage.take()
            .context(
                "`finish_render` should be called only once
                and `begin_render` should be called before"
            )?;

        self.context.queue.submit([render_stage.encoder.finish()]);

        Arc::into_inner(render_stage.output.inner)
            .expect("Render stage's output should be owned by `Graphics`")
            .present();

        Ok(())
    }

    /// Handles window resize event by [`Graphics`].
    pub fn on_window_resize(&mut self, new_size: UInt2) {
        if new_size.x > 0 && new_size.y > 0 {
            let mut config = SURFACE_CFG.write();
            (config.width, config.height) = (new_size.x, new_size.y);
            self.context.surface.configure(&self.context.device, &config);
        }
    }

    pub fn handle_event(world: &World, event: &Event<()>) -> AnyResult<()> {
        use winit::event::WindowEvent;

        let mut graphics = world.resource::<&mut Self>()?;

        graphics.egui.platform.handle_event(event);

        if let Event::WindowEvent { event: WindowEvent::Resized(new_size), .. }
            = event
        {
            let new_size = new_size.to_vec2();

            graphics.on_window_resize(new_size);

            let mut query = world.query::<&mut CameraComponent>();
            for (_entity, camera) in query.into_iter() {
                camera.on_window_resize(new_size);
            }
        }

        Ok(())
    }

    pub fn update(world: &World) -> AnyResult<()> {
        let graphics = world.resource::<&mut Self>()?;

        {
            let MainCamera(camera) = world.copy_resource::<MainCamera>()
                .context("main camera has to be set")?;
            let cam_uniform = camera.get_uniform(world)?;
            let uniform_buffer = world.resource::<&CameraUniformBuffer>()?;
            uniform_buffer.update(&graphics.context.queue, &cam_uniform);
        }

        keyboard::set_input_capture(graphics.egui.context().wants_keyboard_input());

        Ok(())
    }

    pub fn prepare_frame(&mut self, fps: f32) {
        self.window.set_title(&format!("Terramine: {fps:.0} FPS"));
    }
}



#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CommonUniforms {
    pub screen_resolution: vec2,
    pub time: f32,
    pub _padding: u32,
}
assert_impl_all!(CommonUniforms: Send, Sync);



#[derive(Debug)]
pub struct CommonUniformsBuffer {
    pub buffer: Buffer,
    pub binds: Binds,
}
assert_impl_all!(CommonUniformsBuffer: Send, Sync);

impl CommonUniformsBuffer {
    pub fn new(device: &Device, initial_value: &CommonUniforms) -> Self {
        let buffer = Buffer::new(
            device,
            &BufferInitDescriptor {
                label: Some("common_uniforms_buffer"),
                contents: bytemuck::bytes_of(initial_value),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            },
        );

        let layout = BindGroupLayout::new(
            device,
            &BindGroupLayoutDescriptor {
                label: Some("common_uniforms_bind_group_layout"),
                entries: &[
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
                ],
            },
        );

        let bind_group = BindGroup::new(
            device,
            &BindGroupDescriptor {
                label: Some("common_uniforms_bind_group"),
                layout: &layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            },
        );

        let binds = Binds::from_iter([
            (bind_group, Some(layout)),
        ]);

        Self { binds, buffer }
    }

    pub fn update(&self, queue: &Queue, uniforms: CommonUniforms) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniforms]));
        queue.submit(None);
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
        render_stage: &mut RenderStage,
        full_output: egui::FullOutput,
        scale_factor: f32,
    ) -> Result<(), egui_wgpu_backend::BackendError> {
        let paint_jobs = self.context().tessellate(full_output.shapes);

        let (width, height) = {
            let cfg = SURFACE_CFG.read();
            (cfg.width, cfg.height)
        };

        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };

        self.render_pass.add_textures(
            device, queue, &full_output.textures_delta,
        )?;

        self.render_pass.update_buffers(
            device, queue, &paint_jobs, &screen_descriptor,
        );

        self.render_pass.execute(
            &mut render_stage.encoder,
            &render_stage.view,
            &paint_jobs,
            &screen_descriptor,
            None
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
            
                {
                    use crate::terrain::chunk::chunk_array_old::ChunkArray;

                    ChunkArray::spawn_control_window(self.world, ui);
                }

                camera::CameraHandle::spawn_control_window(self.world, ui);
            },

            Tab::Profiler => {
                let time_step = self.world.resource::<&Timer>()
                    .expect("failed to find the timer")
                    .time_step();

                crate::profiler::update_and_build_window(time_step, ui);
            }

            Tab::Console => crate::logger::build_window(ui),

            Tab::Other { .. } => (),
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        matches!(tab, Tab::Other { .. })
    }
}