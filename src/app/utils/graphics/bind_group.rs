#![allow(dead_code)]

use crate::{prelude::*, graphics::{Buffer, TextureView, Sampler}};



crate::define_atomic_id!(BindGroupId);

/// Handle to a binding group.
///
/// A `BindGroup` represents the set of resources bound to the bindings described by a
/// [`BindGroupLayout`]. It can be created with [`Device::create_bind_group`]. A `BindGroup` can
/// be bound to a particular [`RenderPass`] with [`RenderPass::set_bind_group`], or to a
/// [`ComputePass`] with [`ComputePass::set_bind_group`].
///
/// Corresponds to [WebGPU `GPUBindGroup`](https://gpuweb.github.io/gpuweb/#gpubindgroup).
#[derive(Debug, Deref, TypeUuid)]
#[uuid = "c07bcb74-294c-4f38-bf44-1de9fd0ca1bd"]
pub struct BindGroup {
    #[deref]
    pub inner: Arc<wgpu::BindGroup>,
    pub id: BindGroupId,
}
assert_impl_all!(BindGroup: Send, Sync);

impl From<wgpu::BindGroup> for BindGroup {
    fn from(value: wgpu::BindGroup) -> Self {
        Self {
            inner: Arc::new(value),
            id: BindGroupId::new(),
        }
    }
}

impl From<&Arc<wgpu::BindGroup>> for BindGroup {
    fn from(value: &Arc<wgpu::BindGroup>) -> Self {
        Self {
            inner: Arc::clone(value),
            id: BindGroupId::new(),
        }
    }
}



/// Owned resource that can be bound to a pipeline.
///
/// Corresponds to [WebGPU `GPUBindingResource`](
/// https://gpuweb.github.io/gpuweb/#typedefdef-gpubindingresource).
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "23de1ecd-7d2c-44e0-a777-dd92d468f20a"]
pub enum OwnedBindingResource {
    Buffer(Buffer),
    Sampler(Sampler),
    TextureView(TextureView),
}
assert_impl_all!(OwnedBindingResource: Send, Sync);

impl OwnedBindingResource {
    fn as_binding_resource(&self) -> wgpu::BindingResource<'_> {
        use OwnedBindingResource::*;
        match self {
            Buffer(buffer) => buffer.as_entire_binding(),
            Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
            TextureView(view) => wgpu::BindingResource::TextureView(view),
        }
    }
}

impl From<Buffer> for OwnedBindingResource {
    fn from(value: Buffer) -> Self {
        Self::Buffer(value)
    }
}

impl From<Sampler> for OwnedBindingResource {
    fn from(value: Sampler) -> Self {
        Self::Sampler(value)
    }
}

impl From<TextureView> for OwnedBindingResource {
    fn from(value: TextureView) -> Self {
        Self::TextureView(value)
    }
}



#[derive(Debug)]
pub struct PreparedBindGroup<T> {
    pub resources: Vec<OwnedBindingResource>,
    pub bind_group: BindGroup,
    pub data: T,
}
assert_impl_all!(PreparedBindGroup<u32>: Send, Sync);



pub trait AsBindGroup: Send + Sync + 'static {
    type Data: Send + Sync + 'static;

    fn as_bind_group(
        &self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout
    ) -> PreparedBindGroup<Self::Data>;

    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
    where
        Self: Sized;
}
assert_obj_safe!(AsBindGroup<Data = vec3>);