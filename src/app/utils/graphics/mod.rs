pub mod glium_shader;
pub mod texture;
pub mod camera;
pub mod glium_mesh;
pub mod debug_visuals;
pub mod ui;
pub mod light;
pub mod surface;
pub mod mesh;
pub mod shader;

use {
    crate::{
        prelude::*,
        window::Window,
    },
    mesh::{Mesh, Bufferizable},
    shader::Shader,
    wgpu::{
        Device, Queue, Adapter, Instance as WgpuInstance,
    },
    winit::event_loop::EventLoop,
    std::path::PathBuf,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default, Pod, Zeroable)]
pub struct TestVertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Bufferizable for TestVertex {
    const ATTRS: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

    const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &Self::ATTRS,
    };
}

const TEST_VERTICES: [TestVertex; 3] = [
    TestVertex { position: [ 0.0,  0.5], color: [1.0, 0.0, 0.0] },
    TestVertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
    TestVertex { position: [ 0.5, -0.5], color: [0.0, 0.0, 1.0] },
];

/// Graphics handler.
pub struct Graphics {
    pub window: Window,
    pub surface: wgpu::Surface,
    pub adapter: Adapter,
    pub device: Arc<Device>,
    pub queue: Queue,
    pub config: wgpu::SurfaceConfiguration,

    pub test_mesh: Mesh<TestVertex>,

    pub event_loop:	Option<EventLoop<()>>,

    pub imgui: ImGui,
}

impl Graphics {
    /// Creates new [`Graphics`] that holds some renderer stuff.
    pub async fn new() -> Result<Self, winit::error::OsError> {
        let _log_guard = logger::work("graphics", "initialization");

        const DEFAULT_SIZES: USize2 = cfg::window::default::SIZES;
        
        // Window creation
        let event_loop = EventLoop::new();
        let window = Window::from(&event_loop, DEFAULT_SIZES)?;

        // ------------ WGPU initialization ------------

        let wgpu_instance = WgpuInstance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::DX12 | wgpu::Backends::VULKAN,
                dx12_shader_compiler: Default::default(),
            }
        );

        // # Safety
        //
        // `Graphics` owns both the `window` and the `surface` so it
        // lives as long as wgpu's `Surface`.
        let surface = unsafe {
            wgpu_instance.create_surface(&*window)
                .expect("context should be not WebGL2")
        };

        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface)
            })
            .await
            .expect("failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            }, None)
            .await
            .expect("failed to create device");
        let device = Arc::new(device);

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = *swapchain_capabilities.formats.get(0)
            .expect("failed to get swap chain format 0: the surface is incompatible with the adapter");
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: DEFAULT_SIZES.x as u32,
            height: DEFAULT_SIZES.y as u32,
            present_mode: swapchain_capabilities.present_modes[0],
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let shader = Shader::load_from_file(Arc::clone(&device), "triangle shader", "shader.wgsl")
            .await
            .expect("failed to load shader from file");

        let mesh = Mesh::new(
            Arc::clone(&device),
            &TEST_VERTICES,
            Arc::new(shader),
            "test mesh",
            &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Fill,
        );

        // ------------ Dear ImGui initialization ------------

        // Create ImGui context and set `.ini` file name.
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(Some(PathBuf::from("src/imgui_settings.ini")));

        // Bound ImGui to winit.
        let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
        winit_platform.attach_window(imgui_context.io_mut(), &window, imgui_winit_support::HiDpiMode::Rounded);

        // Style configuration.
        imgui_context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;
        imgui_context.style_mut().window_rounding = 16.0;

        // Create ImGui renderer.
        let imgui_renderer = imgui_wgpu::Renderer::new(
            &mut imgui_context,
            &device,
            &queue,
            imgui_wgpu::RendererConfig {
                texture_format: config.format,
                ..Default::default()
            },
        );

        Ok(Self {
            event_loop: Some(event_loop),
            test_mesh: mesh,
            window,
            surface,
            adapter,
            device,
            queue,
            config,
            imgui: ImGui {
                context: imgui_context,
                platform: winit_platform,
                renderer: ImGuiRendererWrapper(imgui_renderer),
            },
        })
    }

    // pub fn refresh_postprocessing_shaders(&mut self) -> Result<(), ShaderError> {
    //     self.quad_draw_resources.shader =
    //         Shader::new("postprocessing", "postprocessing", self.display.as_ref().get_ref())?;

    //     Ok(())
    // }

    pub fn render(
        &mut self,
        use_ui: impl FnOnce(&mut imgui::Ui),
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            },
        );

        {
            let (r, g, b, a) = cfg::shader::CLEAR_COLOR;
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.test_mesh.render(&mut render_pass);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ImGui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let ui = self.imgui.context.new_frame();
            use_ui(ui);

            self.imgui.platform.prepare_render(&ui, &self.window);

            let draw_data = self.imgui.context.render();
            self.imgui.renderer.render(draw_data, &self.queue, &self.device, &mut render_pass)
                .expect("failed to render imgui");
        }
    
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn on_window_resize(&mut self, new_size: UInt2) {
        if new_size.x > 0 && new_size.y > 0 {
            (self.config.width, self.config.height) = (new_size.x, new_size.y);
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Gives event_loop and removes it from graphics struct.
    pub fn take_event_loop(&mut self) -> EventLoop<()> {
        self.event_loop.take()
            .expect("event loop can't be taken twice")
    }
}

#[derive(Debug)]
pub struct ImGui {
    // ImGui context.
    pub context: imgui::Context,

    // ImGui winit support.
    pub platform: imgui_winit_support::WinitPlatform,

    // ImGui WGPU renderer.
    pub renderer: ImGuiRendererWrapper,
}

#[derive(Deref)]
pub struct ImGuiRendererWrapper(imgui_wgpu::Renderer);

impl std::fmt::Debug for ImGuiRendererWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "imgui_wgpu::Renderer {{ ... }}")
    }
}

pub use crate::draw;

/// Draw to target macro.
/// It will execute draw calls passed it after clearing target and before finishing it.
#[macro_export]
macro_rules! draw {
    (
        render_shadows: $render_shadows:expr,
        $graphics:expr,
        $make_target:expr,
        let $uniforms_name:ident = {
            $(
                $($uniform_name:ident : $uniform_def:expr),+
                $(,)?
            )?
        },
        |&mut $fb_name:ident| $fb_draw_call:expr,
        |mut $target_name:ident| $target_draw_call:expr
        $(,)?
    ) => {{
        use $crate::app::utils::cfg::shader::{CLEAR_COLOR, CLEAR_DEPTH, CLEAR_STENCIL};

        let $uniforms_name = ::glium::uniform! {
            render_shadows: $render_shadows,
            $(
                $($uniform_name : $uniform_def,)+
            )?
        };

        let result1 = if $render_shadows {
            $graphics.surface.shadow_buffer.clear_all(CLEAR_COLOR, CLEAR_DEPTH, CLEAR_STENCIL);
            {
                let $uniforms_name = $uniforms_name.add("is_shadow_pass", true);
                let $fb_name = &mut $graphics.surface.shadow_buffer;
                $fb_draw_call
            }
        } else { };
        
        $graphics.surface.frame_buffer.clear_all(CLEAR_COLOR, CLEAR_DEPTH, CLEAR_STENCIL);
        let result2 = {
            let $uniforms_name = $uniforms_name.add("is_shadow_pass", false);
            let $fb_name = &mut $graphics.surface.frame_buffer;
            $fb_draw_call
        };

        let mut $target_name = { $make_target };
            let quad_uniforms = ::glium::uniform! {
                render_shadows: $render_shadows,
                depth_texture:       &$graphics.surface.get_textures().depth,
                albedo_texture:      &$graphics.surface.get_textures().albedo,
                normal_texture:      &$graphics.surface.get_textures().normal,
                light_depth_texture: &$graphics.surface.get_textures().light_depth,
                position_texture:    &$graphics.surface.get_textures().position,
                $(
                    $($uniform_name : $uniform_def,)+
                )?
            };

            $target_name.draw(
                &$graphics.quad_draw_resources.vertices,
                &$graphics.quad_draw_resources.indices,
                &$graphics.quad_draw_resources.shader.program,
                &quad_uniforms,
                &Default::default()
            ).expect("failed to draw to target");
            let result3 = { $target_draw_call };
        $target_name.finish().expect("failed to finish target");

        (result1, result2, result3)
    }};
}