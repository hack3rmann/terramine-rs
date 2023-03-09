pub mod shader;
pub mod texture;
pub mod vertex_buffer;
pub mod camera;
pub mod mesh;
pub mod debug_visuals;

use {
    crate::app::utils::{
        cfg,
    },
    super::window::Window,
    shader::Shader,
    vertex_buffer::VertexBuffer,
    glium::{
        glutin::{
            event_loop::EventLoop,
            event::{
                Event,
                WindowEvent,
            },
            dpi,
        },
    },
    std::{
        path::PathBuf,
    },
    derive_deref_rs::Deref,
    math_linear::prelude::*,
    thiserror::Error,
    imgui_glium_renderer::{
        Renderer as ImguiRenderer,
        RendererError as ImguiRendererError,
    },
};

/// Struct that handles graphics.
#[derive(Debug)]
pub struct Graphics {
    /* Gluim main struct */
    pub display: glium::Display,

    /* OpenGL pipeline stuff */
    pub event_loop:	Option<EventLoop<()>>,

    /* ImGui stuff */
    pub imguic: imgui::Context,
    pub imguiw: imgui_winit_support::WinitPlatform,
    pub imguir: ImguiRendererWrapper,
}


impl Graphics {
    /// Creates new [`Graphics`] that holds some renderer stuff.
    pub fn new() -> Result<Self, GraphicsError> {
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
        let display = glium::Display::from_gl_window(window)?;

        /* ImGui glium renderer setup. */
        let imgui_renderer = ImguiRenderer::init(&mut imgui_context, &display)?;

        Ok(Graphics {
            display,
            imguic: imgui_context,
            imguir: ImguiRendererWrapper(imgui_renderer),
            imguiw: winit_platform,
            event_loop: Some(event_loop),
        })
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
        $target_src:expr,
        |mut $target_name:ident $(:$FrameType:ty)?| $draw_call:expr
        $(,)?
    ) => {{
        $(
            assert!(
                ::types::is_same::<::glium::Frame, $FrameType>(),
                "target type should be glium::Frame",
            );
        )?

        use $crate::app::utils::cfg::shader::{
            CLEAR_COLOR as __CLEAR_COLOR,
            CLEAR_DEPTH as __CLEAR_DEPTH,
            CLEAR_STENCIL as __CLEAR_STENCIL,
        };

        let mut $target_name = { $target_src }; 
        $target_name.clear_all(__CLEAR_COLOR, __CLEAR_DEPTH, __CLEAR_STENCIL);
        let __result = { $draw_call };
        $target_name.finish().expect("failed to finish target");

        __result
    }};
}