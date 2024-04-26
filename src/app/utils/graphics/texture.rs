use crate::{
    prelude::*,
    graphics::{Device, Queue, Image},
};


pub use wgpu::{
    TextureDescriptor, TextureViewDescriptor, ImageCopyTexture, ImageDataLayout, Origin2d, Origin3d,
    TextureAspect, TextureSampleType, TextureViewDimension, SamplerBindingType, TextureDimension,
    SamplerDescriptor, FilterMode, TextureUsages, SurfaceTexture,
};



macros::define_atomic_id!(TextureId);

/// A GPU-accessible texture.
///
/// May be converted from and dereferences to a wgpu [`Texture`](wgpu::Texture).
/// Can be created via [`RenderDevice::create_texture`](crate::renderer::RenderDevice::create_texture).
#[derive(Clone, Debug)]
pub struct Texture {
    pub inner: Arc<wgpu::Texture>,
    pub id: TextureId,
}
assert_impl_all!(Texture: Send, Sync);

impl Texture {
    /// Makes new [texture][Texture].
    pub fn new_empty(device: &Device, desc: &TextureDescriptor) -> Self {
        let texture = device.create_texture(desc);
        Self::from(texture)
    }

    /// Makes new [texture][Texture] with data.
    pub fn new(device: &Device, queue: &Queue, image: &Image, desc: &TextureDescriptor) -> Self {
        let texture = Self::new_empty(device, desc);
        texture.write(queue, image);
        texture
    }

    /// Writes data to the [texture][Texture] from loaded [image][Image].
    pub fn write(&self, queue: &Queue, image: &Image) {
        let (width, height) = image.dimensions().as_tuple();
        let size = image.extent_size();

        queue.write_texture(
            ImageCopyTexture {
                texture: &self.inner,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            image,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * mem::size_of::<f32>() as u32),
                rows_per_image: Some(height),
            },
            size,
        );
    }

    /// Creates a [view][TextureView] of this [texture][Texture].
    pub fn create_view(&self, desc: &TextureViewDescriptor) -> TextureView {
        TextureView::from(self.inner.create_view(desc))
    }
}

impl From<wgpu::Texture> for Texture {
    fn from(value: wgpu::Texture) -> Self {
        Self {
            id: TextureId::new(),
            inner: Arc::new(value),
        }
    }
}

impl Deref for Texture {
    type Target = wgpu::Texture;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}



macros::define_atomic_id!(TextureViewId);

/// Describes a [`Texture`] with its associated metadata required by a pipeline or [`BindGroup`](super::BindGroup).
#[derive(Clone, Debug)]
pub struct TextureView {
    pub id: TextureViewId,
    pub inner: Arc<wgpu::TextureView>,
}
assert_impl_all!(TextureView: Send, Sync);

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        Self {
            id: TextureViewId::new(),
            inner: Arc::new(value),
        }
    }
}

impl Deref for TextureView {
    type Target = wgpu::TextureView;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}



macros::define_atomic_id!(SamplerId);

/// A Sampler defines how a pipeline will sample from a [`TextureView`].
/// They define image filters (including anisotropy) and address (wrapping) modes, among other things.
///
/// May be converted from and dereferences to a wgpu [`Sampler`](wgpu::Sampler).
/// Can be created via [`RenderDevice::create_sampler`](crate::renderer::RenderDevice::create_sampler).
#[derive(Clone, Debug)]
pub struct Sampler {
    pub id: SamplerId,
    pub inner: Arc<wgpu::Sampler>,
}
assert_impl_all!(Sampler: Send, Sync);

impl Sampler {
    pub fn new(device: &Device, desc: &SamplerDescriptor) -> Self {
        let sampler = device.create_sampler(desc);
        Self::from(sampler)
    }
}

impl From<wgpu::Sampler> for Sampler {
    fn from(value: wgpu::Sampler) -> Self {
        Self {
            id: SamplerId::new(),
            inner: Arc::new(value),
        }
    }
}

impl Deref for Sampler {
    type Target = wgpu::Sampler;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}