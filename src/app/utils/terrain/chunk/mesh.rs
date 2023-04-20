use {
    crate::{
        prelude::*,
        graphics::{
            glium_mesh::{Mesh, UnindexedMesh},
            glium_shader::Shader,
        },
        terrain::chunk::prelude::*,
    },
    glium::{
        DrawError, uniforms::Uniforms, Surface, VertexBuffer,
        DrawParameters, backend::Facade, index::PrimitiveType,
    },
};

/// Full-detailed vertex.
#[derive(Copy, Clone, Debug)]
pub struct FullVertex {
    pub position: (f32, f32, f32),
    pub tex_coords: (f32, f32),
    pub face_idx: u8,
}

/// Low-detailed vertex.
#[derive(Copy, Clone, Debug)]
pub struct LowVertex {
    pub position: (f32, f32, f32),
    pub color: (f32, f32, f32),
    pub face_idx: u8,
}

/* Implement Vertex structs as glium intended */
glium::implement_vertex!(FullVertex, position, tex_coords, face_idx);
glium::implement_vertex!(LowVertex, position, color, face_idx);

#[derive(Debug)]
pub enum ChunkDetailedMesh {
    Standart(Box<UnindexedMesh<FullVertex>>),
    Partial(Box<[UnindexedMesh<FullVertex>; 8]>),
}

impl ChunkDetailedMesh {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Standart(mesh) => mesh.is_empty(),
            Self::Partial(meshes) => meshes.iter()
                .all(Mesh::is_empty)
        }
    }

    pub fn render(
        &self, target: &mut impl Surface, shader: &Shader,
        draw_params: &DrawParameters<'_>, uniforms: &impl Uniforms,
    ) -> Result<(), DrawError> {
        match self {
            Self::Standart(mesh) =>
                mesh.render(target, shader, draw_params, uniforms),

            Self::Partial(meshes) => {
                for mesh in meshes.iter() {
                    mesh.render(target, shader, draw_params, uniforms)?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct ChunkMesh {
    pub detailed_mesh: Option<ChunkDetailedMesh>,
    pub low_meshes: [Option<UnindexedMesh<LowVertex>>; Chunk::N_LODS],
}

impl Default for ChunkMesh {
    fn default() -> Self {
        Self {
            detailed_mesh: None,
            low_meshes: array_init(|_| None),
        }
    }
}

impl ChunkMesh {
    /// Checks if [chunk][Chunk]'s mesh is partitioned.
    pub fn is_partitioned(&self) -> bool {
        match self.detailed_mesh {
            Some(ref mesh) => match mesh {
                ChunkDetailedMesh::Partial(_) => true,
                ChunkDetailedMesh::Standart(_) => false,
            },

            None => false,
        }
    }

    /// Connects mesh partitions into one mesh. If [chunk][Chunk] is not
    /// partitioned then it will do nothing.
    pub fn connect_partitions(&mut self, facade: &dyn Facade) {
        let mesh = if let Some(ChunkDetailedMesh::Partial(ref meshes)) = self.detailed_mesh {
            let vertices: Vec<_> = meshes.iter()
                .flat_map(|submesh| submesh
                    .vertices
                    .as_slice()
                    .read()
                    .expect("failed to read vertex buffer subbuffer")
                )
                .collect();

            let vbuffer = VertexBuffer::new(facade, &vertices)
                .expect("failed to create vertex buffer");

            Mesh::new_unindexed(vbuffer, PrimitiveType::TrianglesList)
        } else { return };

        self.detailed_mesh.replace(ChunkDetailedMesh::Standart(Box::new(mesh)));
    }

    /// Drops all generated meshes, if they exist.
    pub fn drop_all(&mut self) {
        let _ = self.detailed_mesh.take();
        for _ in self.low_meshes.iter_mut().filter_map(|m| m.take()) { }        
    }

    pub fn upload_partition(
        &mut self, partition: &[FullVertex],
        partition_idx: usize, facade: &dyn Facade,
    ) {
        match self.detailed_mesh {
            None => panic!("cannot upload only one partition"),
            Some(ref mut mesh) => match mesh {
                ChunkDetailedMesh::Standart(_) =>
                    panic!("cannot upload only one partititon"),

                ChunkDetailedMesh::Partial(ref mut meshes) => {
                    let vbuffer = VertexBuffer::new(facade, partition)
                        .expect("failed to create vertex buffer");
                    let mesh = Mesh::new_unindexed(vbuffer, PrimitiveType::TrianglesList);

                    meshes[partition_idx] = mesh;
                },
            }
        }
    }

    /// Sets mesh to chunk.
    pub fn upload_partitioned_vertices(&mut self, vertices: [&[FullVertex]; 8], facade: &dyn Facade) {
        let partitions = array_init(|i| {
            let vbuffer = VertexBuffer::new(facade, vertices[i])
                .expect("failed to create vertex buffer");

            Mesh::new_unindexed(vbuffer, PrimitiveType::TrianglesList)
        });
        self.detailed_mesh.replace(ChunkDetailedMesh::Partial(Box::new(partitions)));
    }

    /// Sets mesh to chunk.
    pub fn upload_full_detail_vertices(&mut self, vertices: &[FullVertex], facade: &dyn Facade) {
        let vbuffer = VertexBuffer::new(facade, vertices)
            .expect("failed to create vertex buffer");
        let mesh = Mesh::new_unindexed(vbuffer, PrimitiveType::TrianglesList);
        
        self.detailed_mesh.replace(ChunkDetailedMesh::Standart(Box::new(mesh)));
    }

    /// Sets mesh to chunk.
    pub fn upload_low_detail_vertices(&mut self, vertices: &[LowVertex], lod: Lod, facade: &dyn Facade) {
        let vbuffer = VertexBuffer::new(facade, vertices)
            .expect("failed to create vertex buffer");
        let mesh = Mesh::new_unindexed(vbuffer, PrimitiveType::TrianglesList);

        self.low_meshes[lod as usize - 1].replace(mesh);
    }

    /// Renders a [mesh][ChunkMesh].
    pub fn render(
        &self, target: &mut impl Surface, draw_info: &ChunkDrawBundle<'_>,
        uniforms: &impl Uniforms, lod: Lod,
    ) -> Result<(), ChunkRenderError> {
        use ChunkRenderError as Err;
        match lod {
            0 => {
                let mesh = self.detailed_mesh
                    .as_ref()
                    .ok_or(Err::NoMesh(lod))?;
                if !mesh.is_empty() {
                    mesh.render(target, &draw_info.full_shader, &draw_info.draw_params, uniforms)?;
                }
            },
            
            lod => {
                let mesh = self.low_meshes
                    .get(lod as usize - 1)
                    .ok_or(Err::TooBigLod(lod))?
                    .as_ref()
                    .ok_or(Err::NoMesh(lod))?;
                if !mesh.is_empty() {
                    mesh.render(target, &draw_info.low_shader, &draw_info.draw_params, uniforms)?;
                }
            }
        }

        Ok(())
    }

    /// Gives list of available LODs.
    pub fn get_available_lods(&self) -> SmallVec<[Lod; Chunk::N_LODS]> {
        let mut result = smallvec![];

        if self.detailed_mesh.is_some() {
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