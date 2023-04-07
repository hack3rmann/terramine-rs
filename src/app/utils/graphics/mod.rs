pub mod shader;
pub mod texture;
pub mod vertex_buffer;
pub mod camera;
pub mod mesh;
pub mod debug_visuals;
pub mod ui;
pub mod light;
pub mod surface;

use {
    crate::app::utils::{logger, cfg},
    super::window::Window,
    shader::{Shader, ShaderError},
    vertex_buffer::VertexBuffer,
    surface::{Surface, SurfaceError},
    glium::{
        vertex::BufferCreationError as VertexCreationError,
        index::BufferCreationError as IndexCreationError,
        glutin::{
            event_loop::EventLoop,
            event::{
                Event,
                WindowEvent,
            },
            dpi,
        },
    },
    std::{path::PathBuf, pin::Pin},
    derive_deref_rs::Deref,
    math_linear::prelude::*,
    thiserror::Error,
    imgui_glium_renderer::{
        Renderer as ImguiRenderer,
        RendererError as ImguiRendererError,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct QuadVertex {
    pub position: [f32; 4],
    pub texcoord: [f32; 2],
}

glium::implement_vertex! { QuadVertex, position, texcoord }

#[derive(Debug)]
pub struct QuadDrawResources {
    pub shader: Shader,
    pub vertices: glium::VertexBuffer<QuadVertex>,
    pub indices: glium::IndexBuffer<u16>,
}

/// Struct that handles graphics.
pub struct Graphics {
    /* Gluim main struct */
    pub display: Pin<Box<glium::Display>>,

    /* OpenGL pipeline stuff */
    pub event_loop:	Option<EventLoop<()>>,

    /* ImGui stuff */
    pub imguic: imgui::Context,
    pub imguip: imgui_winit_support::WinitPlatform,
    pub imguir: ImguiRendererWrapper,

    /* Deferred rendering stuff */
    pub surface: Surface<'static>,
    pub quad_draw_resources: QuadDrawResources,
}

impl Graphics {
    /// Creates new [`Graphics`] that holds some renderer stuff.
    pub fn new() -> Result<Self, GraphicsError> {
        logger::log!(Info, "graphics", "start initialization");

        /* Glutin event loop */
        let event_loop = EventLoop::new();

        const DEFAULT_SIZES: USize2 = cfg::window::default::SIZES;

        /* Window creation */
        let window = Window::from(&event_loop, DEFAULT_SIZES).take_window();

        /* Create ImGui context ant set settings file name. */
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(Some(PathBuf::from(r"src/imgui_settings.ini")));

        /* Bound ImGui to winit. */
        let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
        winit_platform.attach_window(imgui_context.io_mut(), window.window(), imgui_winit_support::HiDpiMode::Rounded);

        /* Bad start size fix */
        let dummy_event: Event<()> = Event::WindowEvent {
            window_id: window.window().id(),
            event: WindowEvent::Resized(dpi::PhysicalSize::new(DEFAULT_SIZES.x as u32, DEFAULT_SIZES.y as u32))
        };
        winit_platform.handle_event(imgui_context.io_mut(), window.window(), &dummy_event);

        /* Style configuration. */
        imgui_context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;
        imgui_context.style_mut().window_rounding = 16.0;

        /* Glium setup. */
        let display = Box::pin(glium::Display::from_gl_window(window)?);

        /* ImGui glium renderer setup. */
        let imgui_renderer = ImguiRenderer::init(&mut imgui_context, display.as_ref().get_ref())?;

        let quad_draw_resources = {
            let vertices = glium::VertexBuffer::new(display.as_ref().get_ref(), &[
                QuadVertex { position: [-1.0, -1.0, 0.0, 1.0], texcoord: [0.0, 0.0] },
                QuadVertex { position: [-1.0,  1.0, 0.0, 1.0], texcoord: [0.0, 1.0] },
                QuadVertex { position: [ 1.0,  1.0, 0.0, 1.0], texcoord: [1.0, 1.0] },
                QuadVertex { position: [ 1.0, -1.0, 0.0, 1.0], texcoord: [1.0, 0.0] },
            ]).map_err(GraphicsError::VertexBufferCreation)?;
    
            let indices = glium::IndexBuffer::new(
                display.as_ref().get_ref(),
                glium::index::PrimitiveType::TrianglesList,
                &[0_u16, 1, 2, 0, 2, 3],
            ).map_err(GraphicsError::IndexBuffferCreation)?;

            let shader = Shader::new("postprocessing", "postprocessing", display.as_ref().get_ref())?;

            QuadDrawResources { shader, vertices, indices }
        };

        let surface = Surface::new(display.as_ref().get_ref(), UInt2::from(DEFAULT_SIZES))?;

        logger::log!(Info, "graphics", "end initialization");

        Ok(Self {
            display,
            imguic: imgui_context,
            imguir: ImguiRendererWrapper(imgui_renderer),
            imguip: winit_platform,
            event_loop: Some(event_loop),
            surface,
            quad_draw_resources,
        })
    }

    pub fn refresh_postprocessing_shaders(&mut self) -> Result<(), ShaderError> {
        self.quad_draw_resources.shader =
            Shader::new("postprocessing", "postprocessing", self.display.as_ref().get_ref())?;

        Ok(())
    }

    pub fn on_window_resize(&mut self, new_size: UInt2) -> Result<(), SurfaceError> {
        self.surface.on_window_resize(self.display.as_ref().get_ref(), new_size)
    }

    /// Gives event_loop and removes it from graphics struct.
    pub fn take_event_loop(&mut self) -> EventLoop<()> {
        self.event_loop.take()
            .expect("event loop can't be taken twice")
    }
}

#[derive(Debug, Error)]
pub enum GraphicsError {
    #[error("failed to initialize imgui glium renderer: {0}")]
    GliumRenderer(#[from] ImguiRendererError),

    #[error("opengl should be compatible: {0}")]
    IncompatibleOpenGl(#[from] glium::IncompatibleOpenGl),

    #[error("failed to create quad vertex buffer: {0}")]
    VertexBufferCreation(VertexCreationError),

    #[error("failed to create quad index buffer: {0}")]
    IndexBuffferCreation(IndexCreationError),

    #[error("failed to create shader: {0}")]
    ShaderCreation(#[from] ShaderError),

    #[error("failed to create surface: {0}")]
    Surface(#[from] SurfaceError),
}

#[derive(Deref)]
pub struct ImguiRendererWrapper(pub ImguiRenderer);

impl std::fmt::Debug for ImguiRendererWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "imgui_glium_renderer::Renderer {{...}}")
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