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
        texture::{
            Texture2d,
            DepthTexture2d,
            UncompressedFloatFormat,
            MipmapsOption,
            DepthFormat,
        },
        framebuffer::MultiOutputFrameBuffer,
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

#[derive(Debug)]
pub struct DeferredTextures {
    pub depth: DepthTexture2d,
    pub albedo: Texture2d,
    pub normal: Texture2d,
    pub position: Texture2d,
}

#[derive(Clone, Copy, Debug)]
pub struct QuadVertex {
    pub position: [f32; 4],
    pub texcoord: [f32; 2],
}

glium::implement_vertex!(QuadVertex, position, texcoord);

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
    pub imguiw: imgui_winit_support::WinitPlatform,
    pub imguir: ImguiRendererWrapper,

    /* Deferred rendering stuff */
    pub render_textures: Pin<Box<DeferredTextures>>,
    pub frame_buffer: Option<MultiOutputFrameBuffer<'static>>,
    pub quad_draw_resources: QuadDrawResources,
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

        let render_textures = Self::make_render_textures(&display, UInt2::from(DEFAULT_SIZES));

        let quad_draw_resources = {
            let vertices = glium::VertexBuffer::new(&display, &[
                QuadVertex { position: [-1.0, -1.0, 0.0, 1.0], texcoord: [0.0, 0.0] },
                QuadVertex { position: [-1.0,  1.0, 0.0, 1.0], texcoord: [0.0, 1.0] },
                QuadVertex { position: [ 1.0,  1.0, 0.0, 1.0], texcoord: [1.0, 1.0] },
                QuadVertex { position: [ 1.0, -1.0, 0.0, 1.0], texcoord: [1.0, 0.0] },
            ]).expect("failed to create vertex buffer");
    
            let indices = glium::IndexBuffer::new(
                &display,
                glium::index::PrimitiveType::TrianglesList,
                &[0_u16, 1, 2, 0, 2, 3],
            ).expect("failed to create index buffer");

            let shader = Shader::new("postprocessing", "postprocessing", &display);

            QuadDrawResources { shader, vertices, indices }
        };

        let mut graphics = Self {
            display: Box::pin(display),
            imguic: imgui_context,
            imguir: ImguiRendererWrapper(imgui_renderer),
            imguiw: winit_platform,
            event_loop: Some(event_loop),
            render_textures: Box::pin(render_textures),
            frame_buffer: None,
            quad_draw_resources,
        };
        graphics.apply_frame_buffer();

        Ok(graphics)
    }

    fn apply_frame_buffer(&mut self)
    where
        Self: 'static,
    {
        let display = self.display.as_ref().get_ref();
        let textures = self.render_textures.as_ref().get_ref();

        // * Safety:
        // * Safe because pointers are made of Pin<Box<T>> smart pointer
        // * so the pointers are valid. And data can't be moved due to Pin<T>.
        unsafe {
            self.frame_buffer = Some(Self::make_frame_buffer(
                display as *const glium::Display,
                textures as *const DeferredTextures,
            ));
        }
    }

    pub fn refresh_postprocessing_shaders(&mut self) {
        self.quad_draw_resources.shader = Shader::new("postprocessing", "postprocessing", self.display.as_ref().get_ref());
    }

    pub fn make_render_textures(facade: &dyn glium::backend::Facade, window_size: UInt2) -> DeferredTextures {
        let albedo = Texture2d::empty_with_format(
            facade,
            UncompressedFloatFormat::F11F11F10,
            MipmapsOption::NoMipmap,
            window_size.x,
            window_size.y,
        ).expect("failed to create albedo texture");

        let normal = Texture2d::empty_with_format(
            facade,
            UncompressedFloatFormat::F32F32F32,
            MipmapsOption::NoMipmap,
            window_size.x,
            window_size.y,
        ).expect("failed to create normal texture");

        let depth = DepthTexture2d::empty_with_format(
            facade,
            DepthFormat::F32,
            MipmapsOption::NoMipmap,
            window_size.x,
            window_size.y,
        ).expect("failed to create depth texture");

        let position = Texture2d::empty_with_format(
            facade,
            UncompressedFloatFormat::F32F32F32,
            MipmapsOption::NoMipmap,
            window_size.x,
            window_size.y,
        ).expect("failed to create position texture");

        DeferredTextures { depth, albedo, normal, position }
    }

    pub fn on_window_resize(&mut self, new_size: UInt2) {
        self.render_textures = Box::pin(
            Self::make_render_textures(self.display.as_ref().get_ref(), new_size)
        );

        self.apply_frame_buffer();
    }

    /// Makes frame buffer out of raw pointers.
    /// # Safety:
    /// * Pointers should be made out of safe valid references.
    /// * The data behind them should be valid for `'static`.
    /// * The data behind them cannot be moved.
    unsafe fn make_frame_buffer(
        display: *const glium::Display,
        textures: *const DeferredTextures
    ) -> MultiOutputFrameBuffer<'static> {
        let textures = textures.as_ref()
            .expect("textures should be non-null");
        let display = display.as_ref()
            .expect("display should be non-null");

        MultiOutputFrameBuffer::with_depth_buffer(
            display,
            [
                ("out_albedo",   &textures.albedo),
                ("out_normal",   &textures.normal),
                ("out_position", &textures.position),
            ],
            &textures.depth,
        ).expect("failed to create frame buffer")
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
        $graphics:expr,
        $make_target:expr,
        |mut $fb_name:ident $(:$FrameBufferType:ty|)?| $fb_draw_call:expr,
        |mut $target_name:ident $(:$FrameType:ty)?| $target_draw_call:expr
        $(
            ,
            uniform! {
                $($uniform_name:ident : $uniform_def:expr),+
                $(,)?
            }
        )?
        $(,)?
    ) => {{
        // TODO: check $FrameBufferType is actually a frame buffer.
        $(
            assert!(
                ::types::is_same::<::glium::Frame, $FrameType>(),
                "target type should be glium::Frame",
            );
        )?

        use $crate::app::utils::cfg::shader::{CLEAR_COLOR, CLEAR_DEPTH, CLEAR_STENCIL};

        let mut $fb_name = $graphics.frame_buffer.take().expect("frame buffer should be not taken at this point");
            $fb_name.clear_all(CLEAR_COLOR, CLEAR_DEPTH, CLEAR_STENCIL);
            let result1 = { $fb_draw_call };
        $graphics.frame_buffer = Some($fb_name);

        let mut $target_name = { $make_target };
            let quad_uniforms = uniform! {
                depth_texture:  &$graphics.render_textures.depth,
                albedo_texture: &$graphics.render_textures.albedo,
                normal_texture: &$graphics.render_textures.normal,
                position_texture: &$graphics.render_textures.position,
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
            let result2 = { $target_draw_call };
        $target_name.finish().expect("failed to finish target");

        (result1, result2)
    }};
}