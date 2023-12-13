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

    // FIXME:
    egui_wgpu_backend::{RenderPass as EguiRenderPass, ScreenDescriptor as EguiScreenDescriptor},
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
    pub encoder: CommandEncoder,
    pub output: SurfaceTexture,
}
assert_impl_all!(RenderStage: Send, Sync);



#[derive(Deref)]
pub struct DemoWindowsUnsafe {
    pub inner: egui_demo_lib::DemoWindows,
}

// FIXME: !Send + !Sync
unsafe impl Send for DemoWindowsUnsafe { }
unsafe impl Sync for DemoWindowsUnsafe { }



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

        sandbox.insert_resource(
            EguiRenderPass::new(&context.device, SURFACE_CFG.read().format, 1)
        );

        let egui_platform = egui_winit_platform::Platform::new(
            egui_winit_platform::PlatformDescriptor {
                physical_width: window.inner_size().width,
                physical_height: window.inner_size().height,
                scale_factor: window.scale_factor(),
                font_definitions: default(),
                style: default(),
            }
        );

        let demo_window = egui_demo_lib::DemoWindows::default();

        sandbox.insert_resource(DemoWindowsUnsafe { inner: demo_window });

        sandbox.insert_resource(egui_platform);

        
        
        Ok(Self {
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

    pub fn render_sandbox(&mut self, cam: &CameraUniformBuffer) {
        let render_stage = self.render_stage.as_mut()
            .expect(
                "`render_sadbox` should be called after
                `begin_render` and before `finish_render`"
            );

        {
            let mut query = self.sandbox.query::<(
                &BindsRef, &GpuMesh, &RenderPipeline
            )>();

            let mut pass = RenderPass::new(
                "logo_draw_pass",
                &mut render_stage.encoder,
                [&render_stage.view]
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

        {
            let mut platform = self.sandbox
                .resource::<&mut egui_winit_platform::Platform>()
                .unwrap();

            let mut demo_window = self.sandbox
                .resource::<&mut DemoWindowsUnsafe>()
                .unwrap();

            let mut pass = self.sandbox
                .resource::<&mut egui_wgpu_backend::RenderPass>()
                .unwrap();

            platform.begin_frame();

            demo_window.ui(&platform.context());

            let full_output = platform.end_frame(Some(&self.window));
            let paint_jobs = platform.context().tessellate(full_output.shapes);

            let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
                physical_width: SURFACE_CFG.read().width,
                physical_height: SURFACE_CFG.read().height,
                scale_factor: self.window.scale_factor() as f32,
            };
            let tdelta: egui::TexturesDelta = full_output.textures_delta;
            pass.add_textures(&self.context.device, &self.context.queue, &tdelta)
                .expect("add texture ok");
            pass.update_buffers(&self.context.device, &self.context.queue, &paint_jobs, &screen_descriptor);

            // Record all render passes.
            pass.execute(
                &mut render_stage.encoder,
                &render_stage.view,
                &paint_jobs,
                &screen_descriptor,
                Some(wgpu::Color::BLACK),
            ).unwrap();
        }
    }

    pub fn render_with_sandbox(&mut self, time: Time, world: &World)
        -> AnyResult<()>
    {
        self.begin_render(time)?;

        {
            let cam = world.resource::<&CameraUniformBuffer>().unwrap();
            self.render_sandbox(&cam);
        }

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

        let output = self.context.surface.get_current_texture()?;

        let view = TextureView::from(
            output.texture.create_view(&default())
        );

        let mut encoder
            = self.context.device.create_command_encoder(&default());

        ClearPass::clear(&mut encoder, [&view], cfg::shader::CLEAR_COLOR);

        self.render_stage = Some(
            RenderStage::new(view, encoder, output.into())
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

        graphics.sandbox.resource::<&mut egui_winit_platform::Platform>()
            .unwrap()
            .handle_event(event);

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

    pub async fn update(world: &World) -> AnyResult<()> {
        let graphics = world.resource::<&mut Self>()?;

        {
            let MainCamera(camera) = world.copy_resource::<MainCamera>()
                .context("main camera has to be set")?;
            let cam_uniform = camera.get_uniform(world)?;
            let uniform_buffer = world.resource::<&CameraUniformBuffer>()?;
            uniform_buffer.update(&graphics.context.queue, &cam_uniform);
        }

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
