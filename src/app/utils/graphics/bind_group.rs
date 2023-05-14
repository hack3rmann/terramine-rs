#![allow(dead_code)]

use crate::{
    prelude::*,
    graphics::{Buffer, TextureView, Sampler, Device, RenderPass},
};



pub use wgpu::{
    BindingResource, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BindGroupEntry, BindGroupDescriptor,
};



crate::define_atomic_id!(BindGroupLayoutId);

/// Handle to a binding group layout.
///
/// A `BindGroupLayout` is a handle to the GPU-side layout of a binding group. It can be used to
/// create a [`BindGroupDescriptor`] object, which in turn can be used to create a [`BindGroup`]
/// object with [`Device::create_bind_group`]. A series of `BindGroupLayout`s can also be used to
/// create a [`PipelineLayoutDescriptor`], which can be used to create a [`PipelineLayout`].
///
/// It can be created with [`Device::create_bind_group_layout`].
///
/// Corresponds to [WebGPU `GPUBindGroupLayout`](
/// https://gpuweb.github.io/gpuweb/#gpubindgrouplayout).
#[derive(Clone, Debug)]
pub struct BindGroupLayout {
    pub inner: Arc<wgpu::BindGroupLayout>,
    pub id: BindGroupLayoutId,
}

impl From<wgpu::BindGroupLayout> for BindGroupLayout {
    fn from(value: wgpu::BindGroupLayout) -> Self {
        Self { inner: Arc::new(value), id: BindGroupLayoutId::new() }
    }
}

impl Deref for BindGroupLayout {
    type Target = wgpu::BindGroupLayout;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}



crate::define_atomic_id!(BindGroupId);

/// Handle to a binding group.
///
/// A `BindGroup` represents the set of resources bound to the bindings described by a
/// [`BindGroupLayout`]. It can be created with [`Device::create_bind_group`]. A `BindGroup` can
/// be bound to a particular [`RenderPass`] with [`RenderPass::set_bind_group`], or to a
/// [`ComputePass`] with [`ComputePass::set_bind_group`].
///
/// Corresponds to [WebGPU `GPUBindGroup`](https://gpuweb.github.io/gpuweb/#gpubindgroup).
#[derive(Debug, Clone)]
pub struct BindGroup {
    pub inner: Arc<wgpu::BindGroup>,
    pub id: BindGroupId,
}
assert_impl_all!(BindGroup: Send, Sync);

impl BindGroup {
    pub fn bind<'s>(&'s self, pass: &mut RenderPass<'s>, idx: u32) {
        pass.set_bind_group(idx, self, &[]);
    }
}

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

impl Deref for BindGroup {
    type Target = wgpu::BindGroup;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}



/// Owned resource that can be bound to a pipeline.
///
/// Corresponds to [WebGPU `GPUBindingResource`](
/// https://gpuweb.github.io/gpuweb/#typedefdef-gpubindingresource).
#[derive(Debug, Clone)]
pub enum OwnedBindingResource {
    Buffer(Buffer),
    Sampler(Sampler),
    TextureView(TextureView),
}
assert_impl_all!(OwnedBindingResource: Send, Sync);

impl OwnedBindingResource {
    fn as_binding_resource(&self) -> BindingResource<'_> {
        use OwnedBindingResource::*;
        match self {
            Buffer(buffer) => buffer.as_entire_binding(),
            Sampler(sampler) => BindingResource::Sampler(sampler),
            TextureView(view) => BindingResource::TextureView(view),
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



pub trait AsBindGroup: Send + Sync + 'static {
    fn as_bind_group(
        &self, device: &Device, layout: &BindGroupLayout
    ) -> BindGroup;

    fn bind_group_layout(device: &Device) -> BindGroupLayout
    where
        Self: Sized;
}
assert_obj_safe!(AsBindGroup);



#[derive(Debug, Default)]
pub struct Binds {
    pub bind_groups: Vec<BindGroup>,
    pub layouts: Vec<Option<BindGroupLayout>>,
}
assert_impl_all!(Binds: Send, Sync, Component);

impl Binds {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            bind_groups: Vec::with_capacity(cap),
            layouts: Vec::with_capacity(cap),
        }
    }

    pub fn layouts(&self) -> impl Iterator<Item = &wgpu::BindGroupLayout> + '_ {
        self.layouts.iter().flat_map(Option::as_deref)
    }

    pub fn new() -> Self {
        default()
    }

    pub fn push(&mut self, bind_group: BindGroup, layout: Option<BindGroupLayout>) {
        self.bind_groups.push(bind_group);
        self.layouts.push(layout);
    }

    pub fn bind<'s>(&'s self, pass: &mut RenderPass<'s>, start_idx: u32) {
        for (bind_group, i) in self.bind_groups.iter().zip(0..) {
            bind_group.bind(pass, start_idx + i);
        }
    }

    pub fn count(&self) -> usize {
        self.bind_groups.len()
    }

    pub fn join(&mut self, mut other: Self) -> &mut Self {
        self.bind_groups.append(&mut other.bind_groups);
        self.layouts.append(&mut other.layouts);
        self
    }
}

impl FromIterator<(BindGroup, Option<BindGroupLayout>)> for Binds {
    fn from_iter<T: IntoIterator<Item = (BindGroup, Option<BindGroupLayout>)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (bind_groups, layouts) = iter.unzip();
        Self { bind_groups, layouts }
    }
}



crate::define_atomic_id!(BindsId);

#[derive(Clone, Debug)]
pub struct BindsRef {
    pub id: BindsId,
    pub inner: Arc<Binds>,
}

impl From<Binds> for BindsRef {
    fn from(value: Binds) -> Self {
        Self { inner: Arc::new(value), id: BindsId::new() }
    }
}

impl Deref for BindsRef {
    type Target = Binds;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}
