use crate::{
    prelude::*,
    graphics::{Device, Queue, Buffer, Sampler, TextureView, RenderPass},
};



pub use wgpu::{
    BindGroupLayoutEntry, BindGroupLayoutDescriptor, BindGroupDescriptor,
    BindGroupEntry, BindingResource,
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

    pub fn bind<'s>(&'s self, pass: &mut RenderPass<'s>, idx: u32) {
        pass.set_bind_group(idx, self, &[]);
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

    fn cache_key() -> &'static str;

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

    fn update(
        &self, device: &Device, queue: &Queue,
        bind_group: &mut PreparedBindGroup<Self::Data>,
    ) -> bool;

    fn bind_group_layout_entries(device: &Device) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized;
}



#[derive(Debug, Error)]
pub enum AsBindGroupError {
    #[error("failed to create unprepared bind group, try next frame")]
    RetryNextFrame,
}



#[derive(Debug)]
pub struct BindsCache {
    pub binds: HashMap<StaticStr, Box<dyn Any + Send + Sync>>,
}

impl BindsCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<B>(&mut self, bind: PreparedBindGroup<B>)
    where
        B: AsBindGroup + Send + Sync + 'static,
    {
        self.binds.insert(B::cache_key().into(), Box::new(bind));
    }

    pub fn get<B>(&self) -> Option<&PreparedBindGroup<B::Data>>
    where
        B: AsBindGroup + 'static,
    {
        self.binds.get(B::cache_key()).and_then(|value| value.downcast_ref())
    }

    pub fn get_mut<B>(&mut self) -> Option<&mut PreparedBindGroup<B::Data>>
    where
        B: AsBindGroup + 'static,
    {
        self.binds.get_mut(B::cache_key()).and_then(|value| value.downcast_mut())
    }
}

impl Default for BindsCache {
    fn default() -> Self {
        Self { binds: default() }
    }
}



#[cfg(test)]
mod tests {
    use super::*;



    #[derive(Debug, Clone, Copy)]
    pub struct BindableVector {
        pos: Vec3,
    }

    impl AsBindGroup for BindableVector {
        type Data = Vec3;

        fn label() -> Option<&'static str> {
            Some("bindable_vector")
        }

        fn cache_key() -> &'static str {
            "bindableVector"
        }

        fn update(
            &self, _: &Device, queue: &Queue,
            bind_group: &mut PreparedBindGroup<Self::Data>,
        ) -> bool {
            use crate::graphics::*;

            bind_group.unprepared.data = self.pos;

            for (index, resource) in bind_group.unprepared.bindings.iter() {
                if *index == 0 {
                    let OwnedBindingResource::Buffer(buffer)
                        = resource else { return false };

                    queue.write_buffer(
                        buffer, 0, bytemuck::cast_slice(&self.pos.to_array()),
                    );
                    queue.submit(None);
                }
            }

            true
        }

        fn unprepared_bind_group(
            &self, device: &Device, _: &BindGroupLayout,
        ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
            use crate::graphics::{BufferInitDescriptor, BufferUsages};

            let buffer = Buffer::new(device, &BufferInitDescriptor {
                label: Self::label(),
                contents: bytemuck::cast_slice(&self.pos.to_array()),
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