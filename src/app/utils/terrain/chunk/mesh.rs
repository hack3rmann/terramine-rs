use {
    crate::{
        prelude::*,
        graphics::*,
        terrain::chunk::prelude::*,
    },
};



/// Full-detailed vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq, Default)]
pub struct FullVertex {
    pub position: vec3,
    pub tex_coords: vec2,
    pub face_idx: u32,
}
assert_impl_all!(FullVertex: Send, Sync);

impl FullVertex {
    pub const fn new(position: vec3, tex_coords: vec2, face_idx: u32) -> Self {
        Self { position, tex_coords, face_idx }
    }
}

impl Vertex for FullVertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Uint32];
}



/// Low-detailed vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq, Default)]
pub struct LowVertex {
    pub position: vec3,
    pub color: Color,
    pub face_idx: u32,
}
assert_impl_all!(LowVertex: Send, Sync);

impl LowVertex {
    pub const fn new(position: vec3, color: Color, face_idx: u32) -> Self {
        Self { position, color, face_idx }
    }
}

impl Vertex for LowVertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Uint32];
}



#[derive(Debug)]
pub enum ChunkFullMesh {
    Standart(GpuMesh),
    Partial(Box<[GpuMesh; 8]>),
}
assert_impl_all!(ChunkFullMesh: Send, Sync);

impl ChunkFullMesh {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Standart(mesh) => mesh.is_empty(),
            Self::Partial(meshes) => meshes.iter()
                .all(GpuMesh::is_empty)
        }
    }
}

impl Render for ChunkFullMesh {
    type Error = !;
    fn render<'rp, 's: 'rp>(
        &'s self, pipeline: &'rp RenderPipeline, render_pass: &mut RenderPass<'rp>,
    ) -> Result<(), Self::Error> {
        match self {
            Self::Standart(mesh) => {
                let Ok(()) = mesh.render(pipeline, render_pass);
            }

            Self::Partial(meshes) => for mesh in meshes.iter() {
                let Ok(()) = mesh.render(pipeline, render_pass);
            }
        }

        Ok(())
    }
}



#[derive(Debug, SmartDefault)]
pub struct ChunkMesh {
    pub full_mesh: Option<ChunkFullMesh>,
    pub low_meshes: [Option<GpuMesh>; Chunk::N_LODS],
    pub active_lod: Option<Lod>,

    #[default(AtomicBool::new(true))]
    pub is_enabled: AtomicBool,
}
assert_impl_all!(ChunkMesh: Send, Sync);

impl ChunkMesh {
    pub fn switch(&self) {
        let _ = self.is_enabled.fetch_update(AcqRel, Relaxed, |old| Some(!old));
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled.load(Acquire)
    }

    pub fn disable(&self) {
        self.is_enabled.store(false, Release);
    }

    pub fn enable(&self) {
        self.is_enabled.store(true, Release);
    }

    /// Checks if [chunk][Chunk]'s mesh is partitioned.
    pub fn is_partial(&self) -> bool {
        matches!(self.full_mesh, Some(ChunkFullMesh::Partial(_)))
    }

    /// Connects [mesh][Mesh] partitions into one [mesh][Mesh]. If [chunk][Chunk] is not
    /// partitioned then it will do nothing.
    pub fn connect_partitions(&mut self, device: &Device) {
        let Some(ChunkFullMesh::Partial(ref meshes)) = self.full_mesh else { return };
        let meshes = meshes.iter().map(|gpu_mesh| gpu_mesh.read_vertices::<FullVertex>());

        let mesh = Mesh::from_iter(meshes);

        let gpu_mesh = GpuMesh::new(Self::make_partition_desc(device, &mesh));
        self.full_mesh.replace(ChunkFullMesh::Standart(gpu_mesh));
    }

    pub fn upload_partition(
        &mut self, device: &Device, partition: &Mesh<FullVertex>, partition_idx: usize,
    ) {
        let Some(ChunkFullMesh::Partial(ref mut meshes)) = self.full_mesh else {
            panic!("cannot upload only one partition");
        };

        meshes[partition_idx] = GpuMesh::new(
            Self::make_partition_desc(device, partition)
        )
    }

    pub fn make_partition_desc<'s>(
        device: &'s Device, partition: &'s Mesh<FullVertex>,
    ) -> GpuMeshDescriptor<'s, FullVertex> {
        GpuMeshDescriptor {
            device,
            label: Some("chunk_mesh_partition".into()),
            polygon_mode: PolygonMode::Fill,
            mesh: partition,
        }
    }

    pub fn make_full_desc<'s>(
        device: &'s Device, mesh: &'s Mesh<FullVertex>,
    ) -> GpuMeshDescriptor<'s, FullVertex> {
        GpuMeshDescriptor {
            device,
            label: Some("chunk_full_detail_mesh".into()),
            polygon_mode: PolygonMode::Fill,
            mesh,
        }
    }

    pub fn make_low_desc<'s>(
        device: &'s Device, mesh: &'s Mesh<LowVertex>,
    ) -> GpuMeshDescriptor<'s, LowVertex> {
        GpuMeshDescriptor {
            device,
            label: Some("chunk_low_detail_mesh".into()),
            polygon_mode: PolygonMode::Fill,
            mesh,
        }
    }

    /// Sets mesh to [chunk][Chunk].
    pub fn upload_partial_meshes(&mut self, device: &Device, meshes: &[Mesh<FullVertex>; 8]) {
        let partitions = array_init(|i| {
            GpuMesh::new(Self::make_partition_desc(device, &meshes[i]))
        });
        self.full_mesh.replace(ChunkFullMesh::Partial(Box::new(partitions)));
    }

    /// Sets mesh to [chunk][Chunk].
    pub fn upload_full_mesh(&mut self, device: &Device, mesh: &Mesh<FullVertex>) {
        let mesh = GpuMesh::new(Self::make_full_desc(device, mesh));
        self.full_mesh.replace(ChunkFullMesh::Standart(mesh));
    }

    /// Sets mesh to [chunk][Chunk].
    pub fn upload_low_mesh(&mut self, device: &Device, mesh: &Mesh<LowVertex>, lod: Lod) {
        assert_ne!(lod, 0, "`lod` in upload_low_detail_vertices() should be not 0");
    
        let mesh = GpuMesh::new(Self::make_low_desc(device, mesh));
        self.low_meshes[lod as usize - 1].replace(mesh);
    }

    /// Gives list of available [LODs][Lod].
    pub fn get_available_lods(&self) -> SmallVec<[Lod; Chunk::N_LODS]> {
        let mut result = smallvec![];
    
        if self.full_mesh.is_some() {
            result.push(0)
        }
    
        for (low_mesh, lod) in self.low_meshes.iter().zip(1 as Lod..) {
            if low_mesh.is_some() {
                result.push(lod)
            }
        }
    
        result
    }
}

impl Render for ChunkMesh {
    type Error = ChunkRenderError;

    /// Renders a [mesh][ChunkMesh].
    fn render<'rp, 's: 'rp>(
        &'s self, pipeline: &'rp RenderPipeline, render_pass: &mut RenderPass<'rp>,
    ) -> Result<(), Self::Error> {
        use ChunkRenderError as Err;

        if !self.is_enabled() { return Ok(()) }

        match self.active_lod {
            None => return Ok(()),

            Some(0) => {
                let mesh = self.full_mesh
                    .as_ref()
                    .ok_or(Err::NoMesh(0))?;

                let Ok(()) = mesh.render(pipeline, render_pass);
            },

            Some(lod) => {
                let mesh = self.low_meshes
                    .get(lod as usize - 1)
                    .ok_or(Err::TooBigLod(lod))?
                    .as_ref()
                    .ok_or(Err::NoMesh(lod))?;

                let Ok(()) = mesh.render(pipeline, render_pass);
            }
        }

        Ok(())
    }
}