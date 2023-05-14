use {
    crate::{
        prelude::*,
        graphics::{Device, Buffer, RenderPipeline, RenderPass},
    },
    std::{hash::Hash, fmt::Debug, any::TypeId},
};



pub use wgpu::{
    PrimitiveTopology, PrimitiveState, PolygonMode, VertexStepMode, VertexAttribute, vertex_attr_array,
    VertexBufferLayout, BufferView, IndexFormat,
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

    pub fn connect(meshes: impl IntoIterator<Item = Self>) -> Self {
        let mut meshes = meshes.into_iter();
        
        let first = match meshes.next() {
            Some(mesh) => mesh,
            None => return default(),
        };

        let mut vertices = first.vertices;
        let mut indices = first.indices;
        let primitive_topology = first.primitive_topology;

        for mut mesh in meshes {
            vertices.append(&mut mesh.vertices);

            match (&mut indices, &mut mesh.indices) {
                (Some(to), Some(from)) => to.append(from),
                (None, None) => (),
                _ => panic!("can not connect with different indices containment"),
            }

            assert_eq!(
                mesh.primitive_topology,
                primitive_topology,
                "all primitive topologies should be the same to connect meshes",
            )
        }

        Self::new(vertices, indices, primitive_topology)
    }
}

impl<V: Vertex> FromIterator<Mesh<V>> for Mesh<V> {
    fn from_iter<T: IntoIterator<Item = Mesh<V>>>(iter: T) -> Self {
        Self::connect(iter)
    }
}



/// CPU-side index buffer containing indices for [mesh][Mesh].
#[derive(Debug, Clone, Hash, PartialEq, Eq, SmartDefault, TypeUuid)]
#[uuid = "2a840fc0-e39a-11ed-b9fb-0800200c9a66"]
pub enum Indices {
    #[default]
    U16(Vec<u16>),
    U32(Vec<u32>),
}
assert_impl_all!(Indices: Send, Sync, Component);

impl Indices {
    pub fn format(&self) -> IndexFormat {
        match self {
            Self::U16(_) => IndexFormat::Uint16,
            Self::U32(_) => IndexFormat::Uint32,
        }
    }

    pub fn append(&mut self, from: &mut Self) {
        match (self, from) {
            (Self::U16(to), Self::U16(from)) => to.append(from),
            (Self::U32(to), Self::U32(from)) => to.append(from),
            _ => panic!("appending different types if indices is unsupported"),
        }
    }
}

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



#[derive(Debug)]
pub struct GpuMeshDescriptor<'s, V> {
    pub device: &'s Device,
    pub label: Option<StaticStr>,
    pub polygon_mode: PolygonMode,
    pub mesh: &'s Mesh<V>,
}
assert_impl_all!(GpuMeshDescriptor<f32>: Send, Sync);



/// GPU-side [mesh][Mesh] containing information of it's [material][Material] and [gpu-buffer][wgpu::Buffer].
#[derive(Debug)]
pub struct GpuMesh {
    pub is_enabled: AtomicBool,

    pub buffer: Buffer,
    pub n_vertices: usize,

    pub indices: GpuIndices,

    pub label: Option<StaticStr>,
    pub primitive_state: PrimitiveState,

    pub vertex_type_id: TypeId,
}
assert_impl_all!(GpuMesh: Send, Sync, Component);

impl GpuMesh {
    pub fn new<V: Vertex>(desc: GpuMeshDescriptor<'_, V>) -> Self {
        use crate::graphics::{
            BufferUsages, FrontFace, Face,
            DeviceExt, BufferInitDescriptor,
        };

        let vertices = desc.device.create_buffer_init(
            &BufferInitDescriptor {
                label: desc.label.as_deref(),
                contents: bytemuck::cast_slice(&desc.mesh.vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            },
        ).into();

        let indices = match desc.mesh.indices {
            Some(ref indices) => {
                let (contents, idx_format) = match indices {
                    Indices::U16(indices) => (bytemuck::cast_slice(indices), IndexFormat::Uint16),
                    Indices::U32(indices) => (bytemuck::cast_slice(indices), IndexFormat::Uint32),
                };

                GpuIndices::Indexed(desc.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: desc.label.as_deref(),
                        contents,
                        usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                    },
                ).into(), idx_format)
            },

            None => GpuIndices::Unindexed,
        };

        let primitive_state = PrimitiveState {
            topology: desc.mesh.primitive_topology,
            strip_index_format: indices.format(),
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: desc.polygon_mode,
            conservative: false,

        };

        Self {
            vertex_type_id: TypeId::of::<V>(),
            is_enabled: AtomicBool::from(true),
            buffer: vertices,
            n_vertices: desc.mesh.vertices.len(),
            indices,
            label: desc.label,
            primitive_state,
        }
    }

    pub fn get_vertex_buffer_view(&self) -> BufferView {
        self.buffer.slice(..).get_mapped_range()
    }

    pub fn read_vertices<V: Vertex>(&self) -> Mesh<V> {
        if TypeId::of::<V>() != self.vertex_type_id {
            panic!("incompatible vertex type in GpuMesh::read_vertices_unindexed");
        }

        let vertices_view = self.get_vertex_buffer_view();
        let vertices: &[V] = bytemuck::cast_slice(&vertices_view);

        let indices = match self.indices {
            GpuIndices::Unindexed => None,
            GpuIndices::Indexed(ref buffer, format) => {
                let indices_view = buffer.slice(..).get_mapped_range();
                Some(match format {
                    IndexFormat::Uint16 => Indices::U16(
                        bytemuck::cast_slice(&indices_view).to_vec()
                    ),
                    IndexFormat::Uint32 => Indices::U32(
                        bytemuck::cast_slice(&indices_view).to_vec()
                    ),
                })
            },
        };
        
        Mesh::new(vertices.to_vec(), indices, self.primitive_state.topology)
    }

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

impl Render for GpuMesh {
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
    Indexed(Buffer, IndexFormat),
}
assert_impl_all!(GpuIndices: Send, Sync, Component);

impl GpuIndices {
    pub fn format(&self) -> Option<IndexFormat> {
        match self {
            Self::Unindexed => None,
            Self::Indexed(_, format) => Some(*format),
        }
    }
}



/// Trait that all vertices should satisfy to allow usage on GPU.
pub trait Vertex: Default + Pod + PartialEq {
    const ATTRIBUTES: &'static [VertexAttribute];

    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;

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
}



pub trait Render {
    type Error: std::error::Error;

    fn render<'rp, 's: 'rp>(
        &'s self, pipeline: &'rp RenderPipeline, render_pass: &mut RenderPass<'rp>,
    ) -> Result<(), Self::Error>;
}
assert_obj_safe!(Render<Error = ()>);
