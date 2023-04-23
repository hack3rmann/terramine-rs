use {
    std::pin::Pin,
    glium::{
        texture::{DepthTexture2d, Texture2d, TextureCreationError},
        framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer, ValidationError},
        backend::Facade,
    },
    math_linear::prelude::*,
    thiserror::Error,
};

#[derive(Debug)]
pub struct DeferredTextures {
    pub depth: DepthTexture2d,
    pub albedo: Texture2d,
    pub normal: Texture2d,
    pub position: Texture2d,
    pub light_depth: DepthTexture2d,
}

impl DeferredTextures {
    pub fn new(facade: &dyn Facade, window_size: UInt2) -> Result<Self, TextureCreationError> {
        use glium::texture::{UncompressedFloatFormat, DepthFormat, MipmapsOption};

        const MSAA_LEVEL: u32 = 1;
        const SHADOW_QUALITY_LEVEL: u32 = 4;

        let albedo = Texture2d::empty_with_format(
            facade,
            UncompressedFloatFormat::F11F11F10,
            MipmapsOption::NoMipmap,
            window_size.x * MSAA_LEVEL,
            window_size.y * MSAA_LEVEL,
        )?;

        let normal = Texture2d::empty_with_format(
            facade,
            UncompressedFloatFormat::F32F32F32,
            MipmapsOption::NoMipmap,
            window_size.x * MSAA_LEVEL,
            window_size.y * MSAA_LEVEL,
        )?;

        let depth = DepthTexture2d::empty_with_format(
            facade,
            DepthFormat::F32,
            MipmapsOption::NoMipmap,
            window_size.x * MSAA_LEVEL,
            window_size.y * MSAA_LEVEL,
        )?;

        let position = Texture2d::empty_with_format(
            facade,
            UncompressedFloatFormat::F32F32F32,
            MipmapsOption::NoMipmap,
            window_size.x * MSAA_LEVEL,
            window_size.y * MSAA_LEVEL,
        )?;

        let light_depth = DepthTexture2d::empty_with_format(
            facade,
            DepthFormat::F32,
            MipmapsOption::NoMipmap,
            window_size.x * SHADOW_QUALITY_LEVEL,
            window_size.y * SHADOW_QUALITY_LEVEL,
        )?;

        Ok(Self { depth, albedo, normal, position, light_depth })
    }
}

pub struct Surface<'s> {
    render_textures: Pin<Box<DeferredTextures>>,
    pub frame_buffer: MultiOutputFrameBuffer<'s>,
    pub shadow_buffer: SimpleFrameBuffer<'s>,
}

impl<'s> Surface<'s> {
    pub fn new(facade: &dyn Facade, window_size: UInt2) -> Result<Self, SurfaceError> {
        let textures = Box::pin(DeferredTextures::new(facade, window_size)?);

        // * Safety:
        // * Safe, because we own textures and no one can get mutabe access to them.
        // * On texture refresh we make new buffers with new references.
        // * Textures lives as long as buffers.
        let frame_buffer  = unsafe { Self::make_frame_buffer(textures.as_ref(), facade)? };
        let shadow_buffer = unsafe { Self::make_shadow_buffer(textures.as_ref(), facade)? };

        Ok(Self {
            render_textures: textures,
            frame_buffer,
            shadow_buffer,
        })
    }

    /// # Safety
    /// 
    /// `render_textures` should live as long as frame buffer and can not beeing modified.
    /// If textures are modified then it should be called again.
    pub unsafe fn make_frame_buffer<'b>(
        render_textures: Pin<&DeferredTextures>,
        facade: &dyn Facade,
    ) -> Result<MultiOutputFrameBuffer<'b>, ValidationError> {
        let textures = render_textures.get_ref() as *const DeferredTextures;
        let textures = textures.as_ref().unwrap_unchecked();

        MultiOutputFrameBuffer::with_depth_buffer(
            facade,
            [
                ("out_albedo",   &textures.albedo),
                ("out_normal",   &textures.normal),
                ("out_position", &textures.position),
            ],
            &textures.depth,
        )
    }

    /// # Safety
    /// 
    /// `render_textures` should live as long as frame buffer and can not beeing modified.
    /// If textures are modified then it should be called again.
    pub unsafe fn make_shadow_buffer<'b>(
        render_textures: Pin<&DeferredTextures>,
        facade: &dyn Facade,
    ) -> Result<SimpleFrameBuffer<'b>, ValidationError> {
        let texture = &render_textures.light_depth as *const DepthTexture2d;
        let texture = texture.as_ref().unwrap_unchecked();

        SimpleFrameBuffer::depth_only(facade, texture)
    }

    pub fn on_window_resize(&mut self, facade: &dyn Facade, new_size: UInt2) -> Result<(), SurfaceError> {
        self.render_textures.set(
            DeferredTextures::new(facade, new_size)?
        );
        let textures = self.render_textures.as_ref();

        // * Safety:
        // * Safe, because we own textures and no one can get mutabe access to them.
        // * On texture refresh we make new buffers with new references.
        // * Textures lives as long as buffers.
        unsafe {
            self.frame_buffer = Self::make_frame_buffer(textures, facade)?;
            self.shadow_buffer = Self::make_shadow_buffer(textures, facade)?;
        }

        Ok(())
    }

    pub fn get_textures(&self) -> &DeferredTextures {
        self.render_textures.as_ref().get_ref()
    }
}

#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error(transparent)]
    TextureCreation(#[from] TextureCreationError),
    #[error(transparent)]
    Validation(#[from] ValidationError),
}