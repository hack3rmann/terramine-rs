use crate::{
    prelude::*,
    graphics::{Device, Buffer, Sampler, TextureView, BindingResource},
};



pub use wgpu::{
    BindGroupLayoutEntry, BindGroupLayoutDescriptor, BindGroupDescriptor,
    BindGroupEntry,
};



#[derive(Clone, Debug)]
pub struct BindGroupLayout {
    value: Arc<wgpu::BindGroupLayout>,
}

impl BindGroupLayout {
    pub fn new(device: &Device, desc: &BindGroupLayoutDescriptor) -> Self {
        Self::from(device.create_bind_group_layout(desc))
    }
}

impl From<wgpu::BindGroupLayout> for BindGroupLayout {
    fn from(value: wgpu::BindGroupLayout) -> Self {
        Self { value: Arc::new(value) }
    }
}

impl Deref for BindGroupLayout {
    type Target = wgpu::BindGroupLayout;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}



#[derive(Clone, Debug)]
pub struct BindGroup {
    pub(crate) value: Arc<wgpu::BindGroup>,
}
assert_impl_all!(BindGroup: Send, Sync);

impl BindGroup {
    pub fn new(device: &Device, desc: &BindGroupDescriptor) -> Self {
        Self { value: Arc::new(device.create_bind_group(desc)) }
    }
}

impl From<wgpu::BindGroup> for BindGroup {
    fn from(value: wgpu::BindGroup) -> Self {
        Self { value: Arc::new(value) }
    }
}

impl Deref for BindGroup {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}



#[derive(Debug)]
pub struct UnpreparedBindGroup<T> {
    pub bindings: Vec<(u32, OwnedBindingResource)>,
    pub data: T,
}



#[derive(Debug)]
pub struct PreparedBindGroup<T> {
    pub unprepared: UnpreparedBindGroup<T>,
    pub bind_group: BindGroup,
}



#[derive(Debug)]
pub enum OwnedBindingResource {
    Buffer(Buffer),
    TextureView(TextureView),
    Sampler(Sampler),
}

impl OwnedBindingResource {
    pub fn get_binding(&self) -> BindingResource<'_> {
        match self {
            Self::Buffer(buffer) => buffer.as_entire_binding(),
            Self::Sampler(sampler) => BindingResource::Sampler(sampler),
            Self::TextureView(view) => BindingResource::TextureView(view),
        }
    }
}



pub trait AsBindGroup {
    type Data: Send + Sync;

    fn label() -> Option<&'static str> {
        None
    }

    fn as_bind_group(
        &self, device: &Device, layout: &BindGroupLayout,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let unprepared = self.unprepared_bind_group(device, layout)?;

        let entries = unprepared.bindings.iter()
            .map(|(index, binding)| BindGroupEntry {
                binding: *index,
                resource: binding.get_binding(),
            })
            .collect_vec();

        let bind_group = BindGroup::new(device, &BindGroupDescriptor {
            label: Self::label(),
            entries: &entries,
            layout,
        });

        Ok(PreparedBindGroup { bind_group, unprepared })
    }

    fn unprepared_bind_group(
        &self, device: &Device, layout: &BindGroupLayout,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError>;

    fn bind_group_layout(device: &Device, entries: &[BindGroupLayoutEntry]) -> BindGroupLayout
    where
        Self: Sized,
    {
        BindGroupLayout::new(device, &BindGroupLayoutDescriptor {
            label: Self::label(),
            entries,
        })
    }

    fn bind_group_layout_entries(device: &Device) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized;
}



#[derive(Debug, Error)]
pub enum AsBindGroupError {
    #[error("failed to create unprepared bind group, try next frame")]
    RetryNextFrame,
}



#[cfg(test)]
mod tests {
    use super::*;



    #[derive(Debug, Clone, Copy)]
    pub struct BindableVector {
        pos: vec3,
    }

    impl AsBindGroup for BindableVector {
        type Data = vec3;

        fn unprepared_bind_group(
            &self, device: &Device, _: &BindGroupLayout,
        ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
            use crate::graphics::{BufferInitDescriptor, BufferUsages};

            let buffer = Buffer::new(device, &BufferInitDescriptor {
                label: Self::label(),
                contents: bytemuck::cast_slice(&self.pos.as_array()),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            Ok(UnpreparedBindGroup {
                bindings: vec![(0, OwnedBindingResource::Buffer(buffer))],
                data: self.pos,
            })
        }

        fn bind_group_layout_entries(_: &Device) -> Vec<BindGroupLayoutEntry>
        where
            Self: Sized,
        {
            use crate::graphics::{ShaderStages, BindingType, BufferBindingType};

            vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ]
        }
    }
}