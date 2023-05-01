use crate::prelude::*;

crate::define_atomic_id!(TextureId);

/// A GPU-accessible texture.
///
/// May be converted from and dereferences to a wgpu [`Texture`](wgpu::Texture).
/// Can be created via [`RenderDevice::create_texture`](crate::renderer::RenderDevice::create_texture).
#[derive(Clone, Debug, Deref)]
pub struct Texture {
    #[deref]
    pub inner: Arc<wgpu::Texture>,
    pub id: TextureId,
}

impl Texture {
    /// Creates a view of this texture.
    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor) -> TextureView {
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

crate::define_atomic_id!(TextureViewId);

/// Describes a [`Texture`] with its associated metadata required by a pipeline or [`BindGroup`](super::BindGroup).
#[derive(Clone, Debug, Deref)]
pub struct TextureView {
    pub id: TextureViewId,
    #[deref]
    pub inner: Arc<wgpu::TextureView>,
}

#[derive(Clone, Debug, Deref)]
pub struct SurfaceTexture {
    pub inner: Arc<wgpu::SurfaceTexture>,
}

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        Self {
            id: TextureViewId::new(),
            inner: Arc::new(value),
        }
    }
}

impl From<wgpu::SurfaceTexture> for SurfaceTexture {
    fn from(value: wgpu::SurfaceTexture) -> Self {
        Self { inner: Arc::new(value) }
    }
}

crate::define_atomic_id!(SamplerId);

/// A Sampler defines how a pipeline will sample from a [`TextureView`].
/// They define image filters (including anisotropy) and address (wrapping) modes, among other things.
///
/// May be converted from and dereferences to a wgpu [`Sampler`](wgpu::Sampler).
/// Can be created via [`RenderDevice::create_sampler`](crate::renderer::RenderDevice::create_sampler).
#[derive(Clone, Debug, Deref)]
pub struct Sampler {
    pub id: SamplerId,
    #[deref]
    pub inner: Arc<wgpu::Sampler>,
}

impl From<wgpu::Sampler> for Sampler {
    fn from(value: wgpu::Sampler) -> Self {
        Self {
            id: SamplerId::new(),
            inner: Arc::new(value),
        }
    }
}