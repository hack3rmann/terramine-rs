pub mod glium_shader;
pub mod glium_texture;
pub mod camera_resource;
pub mod glium_mesh;
pub mod debug_visuals;
pub mod ui;
pub mod light;
pub mod surface;
pub mod failed_mesh;
pub mod failed_shader;
pub mod failed_texture;
pub mod mesh;
pub mod render_resource;
pub mod pipeline;
pub mod pass;
pub mod gpu_conversions;
pub mod material;
pub mod bind_group;
pub mod macros;
pub mod buffer;
pub mod texture;
pub mod sprite;
pub mod shader;
pub mod asset;

use {
    crate::prelude::*,
    failed_texture::Texture,
};



pub use {
    material::*, gpu_conversions::*, pass::*,
    bind_group::*, buffer::*, texture::*, window::Window,
    mesh::*, shader::*, asset::*, pipeline::RenderPipeline,

    wgpu::{
        SurfaceError, CommandEncoder, TextureUsages, Features,
        DeviceDescriptor, RequestAdapterOptions, Backends, InstanceDescriptor, Instance,
        BindGroupLayout, BindGroup, SurfaceConfiguration, Adapter, Surface, Queue,
        BindGroupEntry, BindGroupDescriptor, ShaderStages, BufferBindingType, BindingType,
        BindGroupLayoutDescriptor, BindGroupLayoutEntry, Device, BufferUsages,
        util::{DeviceExt, BufferInitDescriptor},
    },
    winit::window::Window as WinitWindow,
    imgui_wgpu::{Renderer as ImguiRenderer, RendererConfig as ImguiRendererConfig},
    winit::{event_loop::EventLoop, event::Event},
};



#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default, Pod, Zeroable)]
pub struct TestVertex {
    position: vec2,
    tex_coords: vec2,
}
assert_impl_all!(TestVertex: Send, Sync);

impl Vertex for TestVertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

const TEST_VERTICES: &[TestVertex] = &[
    TestVertex { position: vecf![-0.5, -0.5], tex_coords: vecf![0.0, 1.0] },
    TestVertex { position: vecf![ 0.5, -0.5], tex_coords: vecf![1.0, 1.0] },
    TestVertex { position: vecf![ 0.5,  0.5], tex_coords: vecf![1.0, 0.0] },
    TestVertex { position: vecf![-0.5, -0.5], tex_coords: vecf![0.0, 1.0] },
    TestVertex { position: vecf![ 0.5,  0.5], tex_coords: vecf![1.0, 0.0] },
    TestVertex { position: vecf![-0.5,  0.5], tex_coords: vecf![0.0, 0.0] },
];



#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CommonUniforms {
    pub screen_resolution: vec2,
    pub time: f32,
    pub _pad: u32,
}
assert_impl_all!(CommonUniforms: Send, Sync);



#[derive(Debug)]
pub struct CommonUniformsBuffer {
    pub bind_group_layout: Arc<BindGroupLayout>,
    pub bind_group: BindGroup,
    pub buffer: Buffer,
}
assert_impl_all!(CommonUniformsBuffer: Send, Sync);

impl CommonUniformsBuffer {
    pub fn new(device: &Device, initial_value: &CommonUniforms) -> Self {
        let buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("common_uniforms_buffer"),
                contents: bytemuck::bytes_of(initial_value),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            },
        );

        let layout = device.create_bind_group_layout(
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

        let bind_group = device.create_bind_group(
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

        Self { bind_group_layout: Arc::new(layout), bind_group, buffer: Buffer::from(buffer) }
    }

    pub fn update(&self, queue: &Queue, uniforms: CommonUniforms) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniforms]));
        queue.submit(std::iter::empty());
    }
}



#[derive(Debug)]
pub struct RenderContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub adapter: Adapter,
    pub surface: Surface,
}
assert_impl_all!(RenderContext: Send, Sync);



/// Graphics handler.
#[derive(Debug)]
pub struct Graphics {
    pub window: Window,

    pub context: RenderContext,
    pub imgui: ImGui,

    pub config: SurfaceConfiguration,
    pub common_uniforms: CommonUniformsBuffer,

    pub sandbox: World,
}
assert_impl_all!(Graphics: Send, Sync, Component);

impl Graphics {
    /// Creates new [`Graphics`] that holds some renderer stuff.
    pub async fn new(event_loop: &EventLoop<()>) -> Result<Self, winit::error::OsError> {
        let _log_guard = logger::work("graphics", "initialization");

        let window = Window::from(event_loop, cfg::window::default::SIZES)?;



        // -----------< WGPU initialization >-----------

        // * # Safety
        // * 
        // * `Graphics` owns both the `window` and the `surface` so it's
        // * live as long as wgpu's `Surface`.
        let (context, config) = unsafe { Self::make_render_context(&window) }.await;



        // ------------ Renderng tests stuff ------------

        let common_uniforms = CommonUniformsBuffer::new(
            &context.device,
            &CommonUniforms {
                time: 0.0,
                screen_resolution: window.inner_size().to_vec2().into(),
                _pad: 0,
            },
        );

        let mut sandbox = World::new();

        let test_texture = Texture::load_from_file(
            Arc::clone(&context.device),
            Arc::clone(&context.queue),
            "TerramineIcon32p.png",
            "test_texture",
            0, 1,
        ).await
            .expect("failed to load an image");

        sandbox.insert_resource(test_texture);

        // let mesh = Mesh::new(
        //     MeshDescriptor {
        //         device: Arc::clone(&device),
        //         shader: Arc::new(shader),
        //         label: Arc::new(String::from("test mesh")),

        //         fragment_targets: Arc::new([Some(wgpu::ColorTargetState {
        //             // TODO: think about how transfer config.format everywhere.
        //             format: config.format,
        //             blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        //             write_mask: wgpu::ColorWrites::ALL,
        //         })]),

        //         primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        //         polygon_mode: wgpu::PolygonMode::Fill,

        //         // TODO: remind this ->
        //         bind_group_layouts: Arc::new([
        //             Arc::clone(&common_uniforms.bind_group_layout),
        //             Arc::clone(&test_texture.bind_group_layout),
        //         ]),
        //     },
        //     TEST_VERTICES,
        // );

        let mesh = Mesh::new(TEST_VERTICES.to_vec(), None, PrimitiveTopology::TriangleList);

        let Ok(gpu_mesh) = mesh.to_gpu(GpuMeshDescriptor {
            device: Arc::clone(&context.device),
            label: "test_mesh".into(),
            polygon_mode: default(),
        });

        sandbox.insert_resource(gpu_mesh);

        
        
        let imgui = ImGui::new(&context, &config, &window);



        Ok(Self {
            sandbox,
            window,
            context,
            config,
            common_uniforms,
            imgui,
        })
    }

    /// Initializes a `wgpu` render context.
    /// 
    /// # Safety
    ///
    /// - `window` must be a valid object to create a surface upon.
    /// - `window` must remain valid until after the returned [`RenderContext`] is
    ///   dropped.
    async unsafe fn make_render_context(window: &Window) -> (RenderContext, SurfaceConfiguration) {
        let wgpu_instance = Instance::new(
            InstanceDescriptor {
                backends: Backends::DX12 | Backends::VULKAN,
                dx12_shader_compiler: default(),
            }
        );

        let surface = wgpu_instance.create_surface(window.deref())
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
            .expect("failed to get swap chain format 0: the surface is incompatible with the adapter");
        
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

    pub async fn refresh_test_shader(&mut self) {
        // FIXME:

        // let shader = ShaderSource::load_from_file(
        //     Arc::clone(&self.context.device),
        //     "test_shader",
        //     "shader.wgsl",
        // ).await;

        // match shader {
        //     Ok(shader) => self.test_mesh.reload_shader(Arc::new(shader)),
        //     Err(err) => logger::log!(Error, from = "graphics", "failed to reload test shader: {err}"),
        // }
    }

    pub fn render_sandbox(&mut self, encoder: &mut CommandEncoder, view: TextureView) {
        let texture = self.sandbox.resource::<&Texture>().unwrap();
        let mesh = self.sandbox.resource::<&GpuMesh>().unwrap();
        let pipeline = self.sandbox.resource::<&RenderPipeline>().unwrap();

        let mut pass = RenderPass::new(encoder, "logo_draw_pass", [&view]);

        pass.set_bind_group(0, &self.common_uniforms.bind_group, &[]);
        pass.set_bind_group(1, &texture.bind_group, &[]);
        
        let Ok(()) = mesh.render(&pipeline, &mut pass);
    }

    pub fn render<UseUi: FnOnce(&mut imgui::Ui)>(
        &mut self, desc: RenderDescriptor<UseUi>,
    ) -> Result<(), SurfaceError> {
        let size = self.window.inner_size();
        self.common_uniforms.update(&self.context.queue, CommonUniforms {
            time: desc.time,
            screen_resolution: vecf!(size.width, size.height),
            _pad: 0,
        });

        let output = self.context.surface.get_current_texture()?;
        let view = TextureView::from(output.texture.create_view(&default()));
        let mut encoder = self.context.device.create_command_encoder(&default());

        ClearPass::clear(&mut encoder, [&view]);

        self.render_sandbox(&mut encoder, view.clone());

        {
            let mut render_pass = RenderPass::new(&mut encoder, "imgui_draw_pass", [&view]);

            let ui = self.imgui.context.new_frame();

            (desc.use_imgui_ui)(ui);
            for build in self.imgui.window_builders.iter() {
                build(ui);
            }

            self.imgui.platform.prepare_render(ui, &self.window);

            let draw_data = self.imgui.context.render();
            self.imgui.renderer.render(draw_data, &self.context.queue, &self.context.device, &mut render_pass)
                .expect("failed to render imgui");
        }
    
        self.context.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }

    /// Handles window resize event by [`Graphics`].
    pub fn on_window_resize(&mut self, new_size: UInt2) {
        if new_size.x > 0 && new_size.y > 0 {
            (self.config.width, self.config.height) = (new_size.x, new_size.y);
            self.context.surface.configure(&self.context.device, &self.config);
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        use winit::event::WindowEvent;

        if let Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } = event {
            self.on_window_resize(new_size.to_vec2());
        }

        self.imgui.platform.handle_event(
            self.imgui.context.io_mut(),
            &self.window,
            event,
        );
    }

    pub async fn update(&mut self, dt: Duration) {
        if keyboard::just_pressed(cfg::key_bindings::RELOAD_RESOURCES) {
            self.refresh_test_shader().await;
        }

        self.imgui.context
            .io_mut()
            .update_delta_time(dt);
    }

    pub fn prepare_frame(&mut self, fps: f32) -> Result<(), winit::error::ExternalError> {
        self.window.set_title(&format!("Terramine: {fps:.0} FPS"));

        self.imgui.platform
            .prepare_frame(self.imgui.context.io_mut(), &self.window)
    }
}



/// `imgui-wgpu` crate uses only `wgpu` stuff that are [`Send`] _and_ [`Sync`]
/// but `imgui` context is not [`Send`] nor [`Sync`]. Use carefully.
#[derive(Debug)]
pub struct ImGui {
    /// ImGui context.
    pub context: imgui::Context,

    /// ImGui winit support.
    pub platform: imgui_winit_support::WinitPlatform,

    /// ImGui WGPU renderer.
    pub renderer: ImGuiRendererWrapper,

    /// Always used windows.
    pub window_builders: Vec<fn(&imgui::Ui)>,
}

impl ImGui {
    pub fn new(
        render_context: &RenderContext,
        surface_config: &SurfaceConfiguration,
        window: &Window,
    ) -> Self {
        // Create ImGui context and set `.ini` file name.
        let mut context = imgui::Context::create();
        context.set_ini_filename(Some(PathBuf::from("src/imgui_settings.ini")));

        // Bind ImGui to winit.
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
        platform.attach_window(context.io_mut(), window.deref(), imgui_winit_support::HiDpiMode::Rounded);

        // Style configuration.
        context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.io_mut().font_global_scale = (1.0 / platform.hidpi_factor()) as f32;
        context.style_mut().window_rounding = 16.0;

        // Create ImGui renderer.
        let renderer = ImguiRenderer::new(
            &mut context,
            &render_context.device,
            &render_context.queue,
            ImguiRendererConfig {
                texture_format: surface_config.format,
                ..default()
            },
        );

        Self {
            context,
            platform,
            renderer: ImGuiRendererWrapper(renderer),
            window_builders: default(),
        }
    }

    pub fn add_window_builder(&mut self, builder: fn(&imgui::Ui)) {
        self.window_builders.push(builder);
    }

    pub fn add_window_builder_bunch(
        &mut self,
        builders: impl IntoIterator<Item = fn(&imgui::Ui)>,
    ) {
        self.window_builders.extend(builders.into_iter());
    }
}

// imgui-wgpu uses only wgpu stuff that are Send and Sync
// but imgui context is not Send nor Sync. Use carefully.
unsafe impl Send for ImGui { }
unsafe impl Sync for ImGui { }



#[derive(Deref)]
pub struct ImGuiRendererWrapper(ImguiRenderer);

impl std::fmt::Debug for ImGuiRendererWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "imgui_Renderer {{ ... }}")
    }
}

#[derive(Debug)]
pub struct RenderDescriptor<UseImguiUi> {
    pub use_imgui_ui: UseImguiUi,
    pub time: f32,
}
