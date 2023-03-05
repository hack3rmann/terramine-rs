pub mod shader;
pub mod texture;
pub mod vertex_buffer;
pub mod camera;
pub mod mesh;
pub mod debug_visuals;

use {
    crate::app::utils::{
        cfg,
        werror::prelude::*,
    },
    super::window::Window,
    shader::Shader,
    vertex_buffer::VertexBuffer,
    glium::{
        self as gl,
        backend::Facade,
        glutin::{
            event_loop::EventLoop,
            event::{
                Event,
                WindowEvent,
            },
            dpi,
        },
        Surface,
    },
    std::{
        sync::atomic::{
            AtomicBool, Ordering
        },
        path::PathBuf,
        error::Error,
        rc::Rc,
    },
    derive_deref_rs::Deref,
    math_linear::prelude::*,
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
    /// Graphics initialize function. Can be called once.
    /// If you call it again it will panic.
    pub fn initialize() -> Result<Self, &'static str> {
        /* Validating initialization */
        Self::validate()?;

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
        let display = glium::Display::from_gl_window(window).wunwrap();

        /* ImGui glium renderer setup. */
        let imgui_renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display).wunwrap();

        Ok (
            Graphics {
                display,
                imguic: imgui_context,
                imguir: ImguiRendererWrapper(imgui_renderer),
                imguiw: winit_platform,
                event_loop: Some(event_loop),
            }
        )
    }

    /// Validates initialization.
    fn validate() -> Result<(), &'static str> {
        /* Checks if struct is already initialized */
        static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
        if IS_INITIALIZED.load(Ordering::Acquire) {
            return Err("Attempting to initialize graphics twice! Graphics is already initialized!");
        } else {
            Ok(IS_INITIALIZED.store(true, Ordering::Release))
        }
    }

    /// Gives event_loop and removes it from graphics struct.
    pub fn take_event_loop(&mut self) -> glium::glutin::event_loop::EventLoop<()> {
        self.event_loop.take()
            .expect("graphics.event_loop should be initialized!")
    }

    pub fn get_context(&self) -> &Rc<gl::backend::Context> {
        self.display.get_context()
    }

    #[allow(dead_code)]
    pub fn draw<WriteFn>(display: &gl::Display, write_fn: WriteFn) -> Result<(), Box<dyn Error>>
    where
        WriteFn: FnOnce(&mut gl::Frame) -> Result<(), Box<dyn Error>>,
    {
        let mut target = display.draw();
        target.clear_all(cfg::shader::CLEAR_COLOR, cfg::shader::CLEAR_DEPTH, cfg::shader::CLEAR_STENCIL);
        write_fn(&mut target)?;
        target.finish()?;
        Ok(())
    }
}

#[derive(Deref)]
pub struct ImguiRendererWrapper(pub imgui_glium_renderer::Renderer);

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