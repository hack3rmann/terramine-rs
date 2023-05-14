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

impl Vertex for FullVertex {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] =
        &vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Uint32];

    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
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

impl Vertex for LowVertex {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] =
        &vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Uint32];

    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
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



#[derive(Debug, Default)]
pub struct ChunkMesh {
    pub full_mesh: Option<ChunkFullMesh>,
    pub low_meshes: [Option<GpuMesh>; Chunk::N_LODS],
    pub active_lod: Option<Lod>,
}
assert_impl_all!(ChunkMesh: Send, Sync);

impl ChunkMesh {
    /// Checks if [chunk][Chunk]'s mesh is partitioned.
    pub fn is_partitioned(&self) -> bool {
        matches!(self.full_mesh, Some(ChunkFullMesh::Partial(_)))
    }

    /// Connects mesh partitions into one mesh. If [chunk][Chunk] is not
    /// partitioned then it will do nothing.
    pub fn connect_partitions(&mut self, device: &Device) {
        let mesh = if let Some(ChunkFullMesh::Partial(ref meshes)) = self.full_mesh {
            let meshes = meshes.iter().map(|gpu_mesh| gpu_mesh.read_vertices::<FullVertex>());
            let mesh = Mesh::from_iter(meshes);
            GpuMesh::new(Self::make_partition_desc(device, &mesh))
        } else { return };

        self.full_mesh.replace(ChunkFullMesh::Standart(mesh));
    }

    pub fn upload_partition(
        &mut self, device: &Device, partition: &Mesh<FullVertex>, partition_idx: usize,
    ) {
        match self.full_mesh {
            Some(ChunkFullMesh::Partial(ref mut meshes)) => {
                meshes[partition_idx] = GpuMesh::new(
                    Self::make_partition_desc(device, partition)
                )
            },
            _ => panic!("cannot upload only one partition"),
        }
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
    pub fn upload_partial_mesh(&mut self, device: &Device, meshes: [&Mesh<FullVertex>; 8]) {
        let partitions = array_init(|i| {
            GpuMesh::new(Self::make_partition_desc(device, meshes[i]))
        });
        self.full_mesh.replace(ChunkFullMesh::Partial(Box::new(partitions)));
    }

    /// Sets mesh to [chunk][Chunk].
     pub fn upload_full_detail_vertices(&mut self, device: &Device, mesh: &Mesh<FullVertex>) {
         let mesh = GpuMesh::new(Self::make_full_desc(device, mesh));
         self.full_mesh.replace(ChunkFullMesh::Standart(mesh));
     }

    /// Sets mesh to [chunk][Chunk].
    pub fn upload_low_detail_vertices(&mut self, device: &Device, mesh: &Mesh<LowVertex>, lod: Lod) {
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