pub mod shader;
pub mod texture;
pub mod vertex_buffer;
pub mod camera;
pub mod mesh;
pub mod debug_visuals;
pub mod ui;
pub mod light;

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
        framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer},
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
    pub light_depth: DepthTexture2d,
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
    pub imguip: imgui_winit_support::WinitPlatform,
    pub imguir: ImguiRendererWrapper,

    /* Deferred rendering stuff */
    pub render_textures: Pin<Box<DeferredTextures>>,
    pub frame_buffer: MultiOutputFrameBuffer<'static>,
    pub shadow_buffer: SimpleFrameBuffer<'static>,
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
        let display = Box::pin(glium::Display::from_gl_window(window)?);

        /* ImGui glium renderer setup. */
        let imgui_renderer = ImguiRenderer::init(&mut imgui_context, display.as_ref().get_ref())?;

        let render_textures = Box::pin(
            Self::make_render_textures(display.as_ref().get_ref(), UInt2::from(DEFAULT_SIZES))
        );

        let quad_draw_resources = {
            let vertices = glium::VertexBuffer::new(display.as_ref().get_ref(), &[
                QuadVertex { position: [-1.0, -1.0, 0.0, 1.0], texcoord: [0.0, 0.0] },
                QuadVertex { position: [-1.0,  1.0, 0.0, 1.0], texcoord: [0.0, 1.0] },
                QuadVertex { position: [ 1.0,  1.0, 0.0, 1.0], texcoord: [1.0, 1.0] },
                QuadVertex { position: [ 1.0, -1.0, 0.0, 1.0], texcoord: [1.0, 0.0] },
            ]).expect("failed to create vertex buffer");
    
            let indices = glium::IndexBuffer::new(
                display.as_ref().get_ref(),
                glium::index::PrimitiveType::TrianglesList,
                &[0_u16, 1, 2, 0, 2, 3],
            ).expect("failed to create index buffer");

            let shader = Shader::new("postprocessing", "postprocessing", display.as_ref().get_ref());

            QuadDrawResources { shader, vertices, indices }
        };

        let frame_buffer = Self::make_frame_buffer(render_textures.as_ref(), display.as_ref().get_ref());
        let shadow_buffer = Self::make_shadow_buffer(render_textures.as_ref(), display.as_ref().get_ref());

        Ok(Self {
            display,
            imguic: imgui_context,
            imguir: ImguiRendererWrapper(imgui_renderer),
            imguip: winit_platform,
            event_loop: Some(event_loop),
            render_textures,
            frame_buffer,
            shadow_buffer,
            quad_draw_resources,
        })
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

        let light_depth = DepthTexture2d::empty_with_format(
            facade,
            DepthFormat::F32,
            MipmapsOption::NoMipmap,
            window_size.x * 2,
            window_size.y * 2,
        ).expect("failed to make light depth texture");

        DeferredTextures { depth, albedo, normal, position, light_depth }
    }

    pub fn make_frame_buffer<'t, 'b>(
        render_textures: Pin<&'t DeferredTextures>,
        facade: &dyn glium::backend::Facade,
    ) -> MultiOutputFrameBuffer<'b> {
        let textures = render_textures.get_ref() as *const DeferredTextures;

        // FIXME: add safety arg
        let textures = unsafe { textures.as_ref().unwrap_unchecked() };

        MultiOutputFrameBuffer::with_depth_buffer(
            facade,
            [
                ("out_albedo",   &textures.albedo),
                ("out_normal",   &textures.normal),
                ("out_position", &textures.position),
            ],
            &textures.depth,
        ).expect("failed to create frame buffer")
    }

    pub fn make_shadow_buffer<'t, 'b>(
        render_textures: Pin<&'t DeferredTextures>,
        facade: &dyn glium::backend::Facade,
    ) -> SimpleFrameBuffer<'b> {
        
        let texture = &render_textures.light_depth as *const DepthTexture2d;

        // FIXME: add safety arg
        let texture = unsafe { texture.as_ref().unwrap_unchecked() };

        SimpleFrameBuffer::depth_only(facade, texture)
            .expect("failed to create frame buffer")
    }

    pub fn on_window_resize(&mut self, new_size: UInt2) {
        let display = self.display.as_ref().get_ref();
        
        // TODO: make struct that allows that automatically.
        
        self.render_textures.set(
            Self::make_render_textures(display, new_size)
        );
        
        let textures = self.render_textures.as_ref();
        self.frame_buffer = Self::make_frame_buffer(textures, display);
        self.shadow_buffer = Self::make_shadow_buffer(textures, display);
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
        $uniforms_name:ident,
        |&mut $fb_name:ident $(:$FrameBufferType:ty|)?| $fb_draw_call:expr,
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

        $graphics.shadow_buffer.clear_all(CLEAR_COLOR, CLEAR_DEPTH, CLEAR_STENCIL);
        let result1 = {
            let $uniforms_name = $uniforms_name.add("is_shadow_pass", true);
            let $fb_name = &mut $graphics.shadow_buffer;
            $fb_draw_call
        };
        
        $graphics.frame_buffer.clear_all(CLEAR_COLOR, CLEAR_DEPTH, CLEAR_STENCIL);
        let result2 = {
            let $uniforms_name = $uniforms_name.add("is_shadow_pass", false);
            let $fb_name = &mut $graphics.frame_buffer;
            $fb_draw_call
        };

        let mut $target_name = { $make_target };
            let quad_uniforms = uniform! {
                depth_texture:  &$graphics.render_textures.depth,
                albedo_texture: &$graphics.render_textures.albedo,
                normal_texture: &$graphics.render_textures.normal,
                light_depth_texture: &$graphics.render_textures.light_depth,
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
            let result3 = { $target_draw_call };
        $target_name.finish().expect("failed to finish target");

        (result1, result2, result3)
    }};
}