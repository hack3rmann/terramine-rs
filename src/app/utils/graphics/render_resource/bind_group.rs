use {
    crate::{
        prelude::*,
        graphics::{TextureView, Sampler, Buffer},
        // prelude::Image,
        // render_asset::RenderAssets,
        // render_resource::{resource_macros::*, BindGroupLayout, Buffer, Sampler, TextureView},
        // texture::FallbackImage,
    },
    // encase::ShaderType,
};

crate::define_atomic_id!(BindGroupId);

/// Bind groups are responsible for binding render resources (e.g. buffers, textures, samplers)
/// to a [`TrackedRenderPass`](crate::render_phase::TrackedRenderPass).
/// This makes them accessible in the pipeline (shaders) as uniforms.
///
/// May be converted from and dereferences to a wgpu [`BindGroup`](wgpu::BindGroup).
/// Can be created via [`RenderDevice::create_bind_group`](crate::renderer::RenderDevice::create_bind_group).
#[derive(Clone, Debug, Deref)]
pub struct BindGroup {
    pub id: BindGroupId,
    #[deref]
    pub value: Arc<wgpu::BindGroup>,
}
assert_impl_all!(BindGroup: Send, Sync);

impl From<wgpu::BindGroup> for BindGroup {
    fn from(value: wgpu::BindGroup) -> Self {
        BindGroup {
            id: BindGroupId::new(),
            value: Arc::new(value),
        }
    }
}

// TODO:
// Converts a value to a [`BindGroup`] with a given [`BindGroupLayout`], which can then be used in Bevy shaders.
// This trait can be derived (and generally should be). Read on for details and examples.
//
// This is an opinionated trait that is intended to make it easy to generically
// convert a type into a [`BindGroup`]. It provides access to specific render resources,
// such as [`RenderAssets<Image>`] and [`FallbackImage`]. If a type has a [`Handle<Image>`](bevy_asset::Handle),
// these can be used to retrieve the corresponding [`Texture`](crate::render_resource::Texture) resource.
//
// [`AsBindGroup::as_bind_group`] is intended to be called once, then the result cached somewhere. It is generally
// ok to do "expensive" work here, such as creating a [`Buffer`] for a uniform.
//
// If for some reason a [`BindGroup`] cannot be created yet (for example, the [`Texture`](crate::render_resource::Texture)
// for an [`Image`] hasn't loaded yet), just return [`AsBindGroupError::RetryNextUpdate`], which signals that the caller
// should retry again later.
// pub trait AsBindGroup {
//     /// Data that will be stored alongside the "prepared" bind group.
//     type Data: Send + Sync;

//     /// Creates a bind group for `self` matching the layout defined in [`AsBindGroup::bind_group_layout`].
//     fn as_bind_group(
//         &self,
//         layout: &BindGroupLayout,
//         render_device: &RenderDevice,
//         images: &RenderAssets<Image>,
//         fallback_image: &FallbackImage,
//     ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError>;

//     /// Creates the bind group layout matching all bind groups returned by [`AsBindGroup::as_bind_group`]
//     fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout
//     where
//         Self: Sized;
// }

/// An error that occurs during [`AsBindGroup::as_bind_group`] calls.
#[derive(Error, Debug)]
pub enum AsBindGroupError {
    #[error("The bind group could not be generated. Try again next frame.")]
    RetryNextUpdate,
}
assert_impl_all!(AsBindGroupError: Send, Sync);

/// A prepared bind group returned as a result of [`AsBindGroup::as_bind_group`].
pub struct PreparedBindGroup<T> {
    pub bindings: Vec<OwnedBindingResource>,
    pub bind_group: BindGroup,
    pub data: T,
}
assert_impl_all!(PreparedBindGroup<u8>: Send, Sync);

/// An owned binding resource of any type (ex: a [`Buffer`], [`TextureView`], etc).
/// This is used by types like [`PreparedBindGroup`] to hold a single list of all
/// render resources used by bindings.
#[derive(Debug)]
pub enum OwnedBindingResource {
    Buffer(Buffer),
    TextureView(TextureView),
    Sampler(Sampler),
}
assert_impl_all!(OwnedBindingResource: Send, Sync);

impl OwnedBindingResource {
    pub fn get_binding(&self) -> wgpu::BindingResource<'_> {
        use OwnedBindingResource::*;
        match self {
            Buffer(buffer) => buffer.as_entire_binding(),
            TextureView(view) => wgpu::BindingResource::TextureView(view),
            Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
        }
    }
}

// Converts a value to a [`ShaderType`] for use in a bind group.
// This is automatically implemented for references that implement [`Into`].
// Generally normal [`Into`] / [`From`] impls should be preferred, but
// sometimes additional runtime metadata is required.
// This exists largely to make some [`AsBindGroup`] use cases easier.
// pub trait AsBindGroupShaderType<T: ShaderType> {
//     /// Return the `T` [`ShaderType`] for `self`. When used in [`AsBindGroup`]
//     /// derives, it is safe to assume that all images in `self` exist.
//     fn as_bind_group_shader_type(&self, images: &RenderAssets<Image>) -> T;
// }

// impl<T, U: ShaderType> AsBindGroupShaderType<U> for T
// where
//     for<'a> &'a T: Into<U>,
// {
//     fn as_bind_group_shader_type(&self, _: &RenderAssets<Image>) -> U {
//         self.into()
//     }
// }