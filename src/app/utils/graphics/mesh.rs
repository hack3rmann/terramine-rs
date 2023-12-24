use {
    crate::{prelude::*, components::Name},
    std::any::TypeId,
    graphics::{Device, RenderPass, RenderPipeline, Graphics},
    wgpu::Buffer,
};



pub use wgpu::{
    PrimitiveTopology, PrimitiveState, PolygonMode, VertexStepMode, VertexAttribute, vertex_attr_array,
    VertexBufferLayout, BufferView, IndexFormat, FrontFace, Face,
};



impl ConstDefault for PrimitiveState {
    const DEFAULT: Self = Self {
        topology: const_default(),
        strip_index_format: const_default(),
        front_face: const_default(),
        cull_mode: const_default(),
        unclipped_depth: const_default(),
        polygon_mode: const_default(),
        conservative: const_default(),
    };
}

impl ConstDefault for PrimitiveTopology {
    const DEFAULT: Self = Self::TriangleList;
}

impl ConstDefault for IndexFormat {
    const DEFAULT: Self = Self::Uint32;
}

impl ConstDefault for FrontFace {
    const DEFAULT: Self = Self::Ccw;
}

impl ConstDefault for PolygonMode {
    const DEFAULT: Self = Self::Fill;
}

impl ConstDefault for VertexStepMode {
    const DEFAULT: Self = Self::Vertex;
}



bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct MeshFlags: u32 {
        /// Marks this mesh as convertable to renderable mesh
        const RENDERABLE = 0x1;
    }
}

impl ConstDefault for MeshFlags {
    const DEFAULT: Self = Self::empty();
}

impl Default for MeshFlags {
    fn default() -> Self {
        Self::DEFAULT
    }
}



/// CPU-side mesh containing list of vertices and
/// possibly indices with specified [topology][PrimitiveTopology].
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SimpleMesh {
    pub flags: MeshFlags,
    pub vertex_type: TypeId,
    pub vertices: Vec<u8>,
    pub indices: Option<Indices>,
    pub primitive_topology: PrimitiveTopology,
    pub polygon_mode: PolygonMode,
}
assert_impl_all!(SimpleMesh: Send, Sync);

impl SimpleMesh {
    pub fn new<V: Vertex>(
        vertices: Vec<V>, indices: Option<Indices>,
        primitive_topology: PrimitiveTopology,
        polygon_mode: PolygonMode,
    ) -> Self {
        Self {
            flags: MeshFlags::DEFAULT,
            vertex_type: TypeId::of::<V>(),
            vertices: bytemuck::cast_vec(vertices),
            indices,
            primitive_topology,
            polygon_mode,
        }
    }

    pub fn new_empty<V: Vertex>(
        primitive_topology: PrimitiveTopology, polygon_mode: PolygonMode
    ) -> Self {
        Self::new::<V>(vec![], None, primitive_topology, polygon_mode)
    }
    
    pub fn connect(meshes: impl IntoIterator<Item = Self>) -> Self {
        meshes.into_iter().reduce(|mut acc, mut elem| {
            assert_eq!(
                acc.vertex_type,
                elem.vertex_type,
                "vertex types should be the same to connect meshes",
            );

            assert_eq!(
                acc.primitive_topology,
                elem.primitive_topology,
                "primitive topologies should be the same to connect meshes",
            );

            assert_eq!(
                acc.polygon_mode,
                elem.polygon_mode,
                "polygon modes should be the same to connect meshes",
            );

            match (&mut acc.indices, &mut elem.indices) {
                (Some(to), Some(from)) => to.append(from),
                (None, None) => (),
                _ => panic!("all meshes should have / not have indices to connect them"),
            }

            acc.vertices.append(&mut elem.vertices);

            acc.flags |= elem.flags;

            acc
        }).expect("failed to connect 0 meshes")
    }
}

impl FromIterator<SimpleMesh> for SimpleMesh {
    fn from_iter<T: IntoIterator<Item = SimpleMesh>>(iter: T) -> Self {
        Self::connect(iter)
    }
}



/// CPU-side index buffer containing indices for [mesh][Mesh].
#[derive(Debug, Clone, Hash, PartialEq, Eq, From, IsVariant)]
pub enum Indices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}
assert_impl_all!(Indices: Send, Sync, Component);

impl Indices {
    pub const fn format(&self) -> IndexFormat {
        match self {
            Self::U16(_) => IndexFormat::Uint16,
            Self::U32(_) => IndexFormat::Uint32,
        }
    }

    pub fn append(&mut self, from: &mut Self) {
        match (self, from) {
            (Self::U16(to), Self::U16(from)) => to.append(from),
            (Self::U32(to), Self::U32(from)) => to.append(from),
            _ => panic!("appending different types of indices is unsupported"),
        }
    }
}

impl ConstDefault for Indices {
    const DEFAULT: Self = Self::U16(const_default());
}

impl Default for Indices {
    fn default() -> Self { const_default() }
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



#[derive(Clone, Debug, Hash, PartialEq, Eq, IsVariant)]
pub enum Mesh {
    Simple(SimpleMesh),
    Tree(Vec<Mesh>),
}
assert_impl_all!(Mesh: Send, Sync, Component);

impl Mesh {
    pub fn to_tree(&mut self) {
        let Self::Simple(simple) = self else { return };
        let simple = mem::replace(
            simple,
            SimpleMesh {
                flags: simple.flags,
                vertex_type: simple.vertex_type,
                primitive_topology: simple.primitive_topology,
                polygon_mode: simple.polygon_mode,
                vertices: vec![],
                indices: None,
            }
        );

        *self = Self::Tree(vec![Self::Simple(simple)]);
    }

    pub fn flags(&self) -> MeshFlags {
        match self {
            Self::Simple(simple) => simple.flags,
            Self::Tree(tree) => tree.iter()
                .map(Mesh::flags)
                .reduce(|acc, elem| acc | elem)
                .unwrap_or_default(),
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        // FIXME: make more sutable default value for mesh
        Self::Tree(vec![])
    }
}

impl From<SimpleMesh> for Mesh {
    fn from(value: SimpleMesh) -> Self {
        Self::Simple(value)
    }
}



/// GPU-side mesh
#[derive(Debug)]
pub struct SimpleGpuMesh {
    pub is_enabled: AtomicBool,

    pub buffer: Buffer,
    pub n_vertices: usize,

    pub indices: GpuIndices,

    pub label: Option<StaticStr>,
    pub primitive_state: PrimitiveState,

    pub vertex_type_id: TypeId,
}
assert_impl_all!(SimpleGpuMesh: Send, Sync, Component);

impl SimpleGpuMesh {
    pub fn new(
        device: &Device, mesh: &SimpleMesh, label: Option<impl Into<StaticStr>>,
    ) -> Self {
        use crate::graphics::{BufferUsages, DeviceExt, BufferInitDescriptor};

        let label = label.map(|value| value.into());

        let vertices = device.create_buffer_init(
            &BufferInitDescriptor {
                label: label.as_deref(),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            },
        );

        let indices = match mesh.indices {
            None => GpuIndices::Unindexed,
            
            Some(ref indices) => {
                let (contents, idx_format) = match indices {
                    Indices::U16(indices) => (bytemuck::cast_slice(indices), IndexFormat::Uint16),
                    Indices::U32(indices) => (bytemuck::cast_slice(indices), IndexFormat::Uint32),
                };

                GpuIndices::Indexed(device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: label.as_deref(),
                        contents,
                        usage: BufferUsages::all(),
                    },
                ), idx_format)
            },
        };

        let primitive_state = PrimitiveState {
            topology: mesh.primitive_topology,
            strip_index_format: indices.format(),
            front_face: FrontFace::Ccw,
            cull_mode: None,// FIXME: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: mesh.polygon_mode,
            conservative: false,

        };

        let vertex_type_id = mesh.vertex_type;

        Self {
            vertex_type_id,
            is_enabled: AtomicBool::new(true),
            buffer: vertices,
            n_vertices: mesh.vertices.len(),
            indices,
            label,
            primitive_state,
        }
    }

    pub fn get_vertex_buffer_view(&self) -> BufferView {
        // TODO: map range to avoid random panic
        self.buffer.slice(..).get_mapped_range()
    }

    pub fn read_vertices(&self) -> SimpleMesh {
        let vertices_view = self.get_vertex_buffer_view();
        let vertices: &[u8] = bytemuck::cast_slice(&vertices_view);

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
        
        SimpleMesh {
            flags: MeshFlags::DEFAULT,
            vertex_type: self.vertex_type_id,
            vertices: vertices.to_vec(),
            indices,
            primitive_topology: self.primitive_state.topology,
            polygon_mode: self.primitive_state.polygon_mode,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.n_vertices == 0
    }

    pub fn switch_visibility(&self) {
        _ = self.is_enabled.fetch_xor(true, AcqRel);
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

    pub fn draw<'rp, 's: 'rp>(
        &'s self, render_pass: &mut RenderPass<'rp>,
    ) {
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
        render_pass.draw(0..self.n_vertices as u32, 0..1);
    }

    pub fn render<'rp, 's: 'rp>(
        &'s self, pipeline: &'rp RenderPipeline, render_pass: &mut RenderPass<'rp>
    ) {
        if !self.is_empty() || !self.is_enabled() {
            return;
        }

        render_pass.set_pipeline(pipeline);
        self.draw(render_pass);
    }
}



#[derive(Debug)]
pub enum GpuIndices {
    Unindexed,
    Indexed(Buffer, IndexFormat),
}
assert_impl_all!(GpuIndices: Send, Sync, Component);

impl GpuIndices {
    pub const fn format(&self) -> Option<IndexFormat> {
        match self {
            Self::Unindexed => None,
            Self::Indexed(_, format) => Some(*format),
        }
    }
}



#[derive(Debug, Clone, IsVariant)]
pub enum GpuMesh {
    Simple(Arc<SimpleGpuMesh>),
    Tree(Vec<GpuMesh>),
}
assert_impl_all!(GpuMesh: Send, Sync, Component);

impl GpuMesh {
    pub fn new(
        device: &Device, mesh: &Mesh, label: Option<impl Into<StaticStr>>
    ) -> Self {
        let label = label.map(|value| value.into());

        match mesh {
            Mesh::Simple(mesh)
                => Self::Simple(Arc::new(SimpleGpuMesh::new(device, mesh, label))),

            Mesh::Tree(mesh) => Self::Tree(mesh.iter().map(|mesh| {
                Self::new(device, mesh, label.clone())
            }).collect())
        }
    }

    fn draw<'rp, 's: 'rp>(
        &'s self, render_pass: &mut RenderPass<'rp>,
    ) {
        match self {
            Self::Simple(mesh) => mesh.draw(render_pass),
            Self::Tree(meshes) => for mesh in meshes {
                mesh.draw(render_pass);
            }
        }
    }

    pub fn render<'rp, 's: 'rp>(
        &'s self, pipeline: &'rp RenderPipeline, render_pass: &mut RenderPass<'rp>,
    ) {
        render_pass.set_pipeline(pipeline);
        self.draw(render_pass);
    }

    pub fn make_renderable_system(world: &mut World) {
        let device = world.resource::<&Graphics>().unwrap().device();

        let insert_map: Vec<(Entity, GpuMesh)>
            = world.query::<(&Mesh, Option<&Name>, Option<&GpuMesh>)>()
            .into_iter()
            .filter_map(|(entity, (mesh, maybe_name, gpu_mesh))| {
                if gpu_mesh.is_some() || mesh.flags().contains(MeshFlags::RENDERABLE) {
                    return None;
                }

                let label = maybe_name.map(|name| name.value.clone());
                let gpu_mesh = GpuMesh::new(&device, mesh, label);

                Some((entity, gpu_mesh))
            })
            .collect();

        for (entity, mesh) in insert_map {
            _ = world.insert_one(entity, mesh);
        }
    }

    pub fn to_tree(&mut self) {
        let Self::Simple(simple) = self else { return };
        let simple = simple.clone();
    
        *self = Self::Tree(vec![GpuMesh::Simple(simple)]);
    }
}

impl From<SimpleGpuMesh> for GpuMesh {
    fn from(value: SimpleGpuMesh) -> Self {
        Self::Simple(Arc::new(value))
    }
}

impl From<Vec<GpuMesh>> for GpuMesh {
    fn from(value: Vec<GpuMesh>) -> Self {
        Self::Tree(value)
    }
}