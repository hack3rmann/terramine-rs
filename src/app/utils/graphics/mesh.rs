use {
    crate::{
        prelude::*,
        graphics::{ToGpu, Device, Buffer, RenderPipeline, RenderPass},
    },
    std::{hash::Hash, fmt::Debug},
};



pub use wgpu::{
    PrimitiveTopology, PrimitiveState, PolygonMode, VertexStepMode, VertexAttribute, vertex_attr_array,
    VertexBufferLayout,
};



/// CPU-side mesh containing list of vertices and
/// possibly indices with specified [topology][PrimitiveTopology].
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Mesh<V> {
    pub vertices: Vec<V>,
    pub indices: Option<Indices>,
    pub primitive_topology: PrimitiveTopology,
}
assert_impl_all!(Mesh<f32>: Send, Sync, Component);

impl<V: Vertex> Mesh<V> {
    pub fn new(vertices: Vec<V>, indices: Option<Indices>, primitive_topology: PrimitiveTopology) -> Self {
        Self { vertices, indices, primitive_topology }
    }

    pub fn new_empty(primitive_topology: PrimitiveTopology) -> Self {
        Self::new(vec![], None, primitive_topology)
    }
}

impl<V: Vertex> ToGpu for Mesh<V> {
    type Descriptor = GpuMeshDescriptor;
    type GpuType = GpuMesh;
    type Error = !;
    
    /// Creates new [`GpuMesh`] instance from [`Mesh`].
    fn to_gpu(&self, desc: GpuMeshDescriptor) -> Result<GpuMesh, !> {
        use crate::graphics::{
            BufferUsages, IndexFormat, FrontFace, Face,
            DeviceExt, BufferInitDescriptor,
        };

        let vertices = desc.device.create_buffer_init(
            &BufferInitDescriptor {
                label: desc.label.as_deref(),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            },
        ).into();

        let (indices, idx_format) = match self.indices {
            Some(ref indices) => {
                let (contents, idx_format) = match indices {
                    Indices::U16(indices) => (bytemuck::cast_slice(indices), IndexFormat::Uint16),
                    Indices::U32(indices) => (bytemuck::cast_slice(indices), IndexFormat::Uint32),
                };

                let indices = GpuIndices::Indexed(desc.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: desc.label.as_deref(),
                        contents,
                        usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                    },
                ).into());

                (indices, Some(idx_format))
            },

            None => (GpuIndices::Unindexed, None),
        };

        let primitive_state = PrimitiveState {
            topology: self.primitive_topology,
            strip_index_format: idx_format,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: desc.polygon_mode,
            conservative: false,

        };

        Ok(GpuMesh {
            is_enabled: AtomicBool::from(true),
            buffer: vertices,
            n_vertices: self.vertices.len(),
            indices,
            label: desc.label,
            primitive_state,
        })
    }
}



#[derive(Debug, TypeUuid)]
#[uuid = "a529a8b9-4e2b-40ee-b689-654a26066acd"]
pub struct GpuMeshDescriptor {
    pub device: Arc<Device>,
    pub label: Option<StaticStr>,
    pub polygon_mode: PolygonMode,
}
assert_impl_all!(GpuMeshDescriptor: Send, Sync);



/// CPU-side index buffer containing indices for [mesh][Mesh].
#[derive(Debug, Clone, Hash, PartialEq, Eq, SmartDefault, TypeUuid)]
#[uuid = "2a840fc0-e39a-11ed-b9fb-0800200c9a66"]
pub enum Indices {
    #[default]
    U16(Vec<u16>),
    U32(Vec<u32>),
}
assert_impl_all!(Indices: Send, Sync, Component);

impl From<Vec<u16>> for Indices {
    fn from(value: Vec<u16>) -> Self {
        Self::U16(value)
    }
}

impl From<Vec<u32>> for Indices {
    fn from(value: Vec<u32>) -> Self {
        Self::U32(value)
    }
}



/// GPU-side [mesh][Mesh] containing information of it's [material][Material] and [gpu-buffer][wgpu::Buffer].
#[derive(Debug, TypeUuid)]
#[uuid = "286ff010-e38a-11ed-b9fb-0800200c9a66"]
pub struct GpuMesh {
    pub is_enabled: AtomicBool,

    pub buffer: Buffer,
    pub n_vertices: usize,

    pub indices: GpuIndices,

    pub label: Option<StaticStr>,
    pub primitive_state: PrimitiveState,
}
assert_impl_all!(GpuMesh: Send, Sync, Component);

impl GpuMesh {
    pub fn is_empty(&self) -> bool {
        self.n_vertices == 0
    }

    pub fn switch_visibility(&self) {
        let _ = self.is_enabled.fetch_update(AcqRel, Relaxed, |old| Some(!old));
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled.load(Acquire)
    }

    pub fn enable(&self) {
        self.is_enabled.store(true, Release);
    }

    pub fn disable(&self) {
        self.is_enabled.store(false, Release);
    }
}

impl Renderable for GpuMesh {
    type Error = !;

    fn render<'rp, 's: 'rp>(
        &'s self, pipeline: &'rp RenderPipeline, render_pass: &mut RenderPass<'rp>,
    ) -> Result<(), Self::Error> {
        if self.is_empty() { return Ok(()) }

        render_pass.set_pipeline(pipeline);
        render_pass.set_vertex_buffer(0, *self.buffer.slice(..));
        render_pass.draw(0..self.n_vertices as u32, 0..1);

        Ok(())
    }
}



#[derive(Debug, TypeUuid)]
#[uuid = "abf45a60-e39a-11ed-b9fb-0800200c9a66"]
pub enum GpuIndices {
    Unindexed,
    Indexed(Buffer),
}
assert_impl_all!(GpuIndices: Send, Sync, Component);

impl From<Buffer> for GpuIndices {
    fn from(buffer: Buffer) -> Self {
        Self::Indexed(buffer)
    }
}



/// Trait that all vertices should satisfy to allow usage on GPU.
pub trait Vertex: Pod + PartialEq {
    const ATTRIBUTES: &'static [VertexAttribute];
    const STEP_MODE: VertexStepMode;

    const BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as u64,
        step_mode: Self::STEP_MODE,
        attributes: Self::ATTRIBUTES,
    };
}



#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq, Default)]
pub struct DefaultVertex {
    pub position: vec3,
}
assert_impl_all!(DefaultVertex: Send, Sync);

impl DefaultVertex {
    pub const fn new(position: vec3) -> Self {
        Self { position }
    }
}

impl From<vec3> for DefaultVertex {
    fn from(value: vec3) -> Self {
        Self::new(value)
    }
}

impl Vertex for DefaultVertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![0 => Float32x3];

    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}



#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq, Default)]
pub struct TexturedVertex {
    pub position: vec3,
    pub uv: vec2,
}
assert_impl_all!(TexturedVertex: Send, Sync);

impl TexturedVertex {
    pub const fn new(position: vec3, uv: vec2) -> Self {
        Self { position, uv }
    }
}

impl Vertex for TexturedVertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}



pub trait Renderable {
    type Error: std::error::Error;

    fn render<'rp, 's: 'rp>(
        &'s self, pipeline: &'rp RenderPipeline, render_pass: &mut RenderPass<'rp>,
    ) -> Result<(), Self::Error>;
}
assert_obj_safe!(Renderable<Error = ()>);
