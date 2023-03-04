pub mod iterator;
pub mod chunk_array;
pub mod tasks;

use {
    crate::app::utils::{
        cfg,
        graphics::{
            mesh::{Mesh, UnindexedMesh},
            shader::Shader,
            vertex_buffer::VertexBuffer,
            camera::Camera,
        },
        reinterpreter::*,
    },
    super::voxel::{
        Voxel,
        LoweredVoxel,
        shape::{CubeDetailed, CubeLowered},
        voxel_data::*,
        generator as gen,
    },
    math_linear::prelude::*,
    glium::{
        self as gl,
        DrawError,
        uniforms::Uniforms,
        Frame,
        index::PrimitiveType,
        implement_vertex,
    },
    iterator::{CubeBorder, SpaceIter, Sides},
    thiserror::Error,
};

/// Full-detailed vertex.
#[derive(Copy, Clone, Debug)]
pub struct DetailedVertex {
    pub position: (f32, f32, f32),
    pub tex_coords: (f32, f32),
    pub light: f32
}

/// Low-detailed vertex.
#[derive(Copy, Clone, Debug)]
pub struct LoweredVertex {
    pub position: (f32, f32, f32),
    pub color: (f32, f32, f32),
    pub light: f32
}

/* Implement Vertex structs as glium intended */
implement_vertex!(DetailedVertex, position, tex_coords, light);
implement_vertex!(LoweredVertex, position, color, light);

macro_rules! impl_chunk_with_refs {
    ($($impls:item)*) => {
        impl Chunk { $($impls)* }
        impl ChunkRef<'_> { $($impls)* }
        impl ChunkMut<'_> { $($impls)* }
    };
}

#[derive(Debug)]
pub struct Chunk {
    pub pos: Int3,
    pub voxel_ids: Vec<Id>,
    pub meta_info: MetaInfo,

    pub detailed_mesh: Option<UnindexedMesh<DetailedVertex>>,
    pub low_meshes: [Option<UnindexedMesh<LoweredVertex>>; Self::N_LODS],
}

impl_chunk_with_refs! {
    /// Gives `Some()` with fill id or returns `None`.
    pub fn voxels(&self) -> impl Iterator<Item = Voxel> + '_ {
        self.voxel_ids.iter()
            .copied()
            .zip(Chunk::global_pos_iter(self.pos.to_owned()))
            .map(|(id, pos)| Voxel::new(pos, &VOXEL_DATA[id as usize]))
    }

    /// Gives iterator over low-detail voxels with their coords.
    pub fn low_voxel_iter(&self, lod: Lod) -> impl Iterator<Item = (LoweredVoxel, Int3)> + '_ {
        let sub_chunk_size = 2_i32.pow(lod as u32);

        Chunk::chunked_pos_iter(sub_chunk_size as usize)
            .map(move |chunk_iter| {
                let (color_sum, n_colors) = chunk_iter
                    .map(|pos| self.get_voxel_local(pos))
                    .filter(|voxel| !voxel.is_air())
                    .map(|voxel| voxel.data.avarage_color)
                    .fold((Color::ZERO, 0_usize), |(col_acc, n_acc), col|
                        (col_acc + col, n_acc + 1)
                    );

                match n_colors {
                    0 => LoweredVoxel::Transparent,
                    n => LoweredVoxel::Colored(color_sum / n as f32),
                }
            })
            .zip(SpaceIter::zeroed_cubed(Chunk::SIZE as i32 / sub_chunk_size))
    }

    /// Checks if chunk is empty.
    pub fn is_empty(&self) -> bool {
        if self.voxel_ids.is_empty() {
            return true
        }

        match self.meta_info.fill_type {
            FillType::AllSame(id) => id == AIR_VOXEL_DATA.id,
            _ => false,
        }
    }

    /// Gives `Some()` with fill id or returns `None`.
    pub fn fill_id(&self) -> Option<Id> {
        match self.meta_info.fill_type {
            FillType::AllSame(id) => Some(id),
            _ => None,
        }
    }

    /// Checks if chunk is filled with same voxel id.
    pub fn is_same_filled(&self) -> bool {
        self.fill_id().is_some()
    }

    pub fn make_vertices_detailed(&self, chunk_adj: ChunkAdj) -> Vec<DetailedVertex> {
        if self.is_empty() { return vec![] }

        let pos_iter: Box<dyn Iterator<Item = Int3>> = match self.meta_info.fill_type {
            FillType::Default =>
                Box::new(Chunk::local_pos_iter()),

            FillType::AllSame(id) => if id == AIR_VOXEL_DATA.id {
                Box::new(std::iter::empty())
            } else {
                Box::new(CubeBorder::new(Chunk::SIZE as i32))
            },
        };

        pos_iter
            .map(|pos| self.get_voxel_local(pos))
            .filter(|voxel| !voxel.is_air())
            .flat_map(|voxel| {
                let side_iter = SpaceIter::adj_iter(Int3::ZERO)
                    .filter(|&offset| {
                        let adj = chunk_adj.sides.by_offset(offset);
                        match self.get_voxel_global(voxel.pos + offset) {
                            ChunkOption::Item(voxel) => voxel.is_air(),

                            ChunkOption::OutsideChunk => match adj {
                                None => true,

                                Some(chunk) => match chunk.get_voxel_global(voxel.pos + offset) {
                                    ChunkOption::Item(voxel) => voxel.is_air(),
                                    ChunkOption::OutsideChunk => true,
                                }
                            },
                        }
                    });

                const N_CUBE_VERTICES: usize = 36;
                let mut vertices = Vec::with_capacity(N_CUBE_VERTICES);

                let mesh_builder = CubeDetailed::new(voxel.data);
                for offset in side_iter {
                    mesh_builder.by_offset(offset, voxel.pos, &mut vertices);
                }

                vertices
            })
            .collect()
    }

    /// Makes vertices for *low detail* mesh from voxel array.
    pub fn make_vertices_low(&self, chunk_adj: ChunkAdj, lod: Lod) -> Vec<LoweredVertex> {
        assert!(lod > 0, "There's a separate function for LOD = 0! Use .make_vertices_detailed() instead!");
        
        if self.is_empty() { return vec![] }

        // TODO: optimize for same-filled chunks
        let sub_chunk_size = 2_i32.pow(lod as u32);
        self.low_voxel_iter(lod)
            .filter_map(|(voxel, p)| match voxel {
                LoweredVoxel::Transparent => None,
                LoweredVoxel::Colored(color) => Some((color, p)),
            })
            .flat_map(|(voxel_color, local_low_pos)| {
                let local_pos = local_low_pos * sub_chunk_size;
                let global_pos = Chunk::local_to_global_pos(self.pos.to_owned(), local_pos);

                let center_pos = vec3::from(global_pos)
                         + 0.5 * vec3::all(sub_chunk_size as f32)
                         - 0.5 * vec3::all(cfg::terrain::VOXEL_SIZE);
                         
                let is_blocking_voxel = |pos: Int3, offset: Int3| match self.get_voxel_global(pos) {
                    ChunkOption::OutsideChunk => {
                        match chunk_adj.sides.by_offset(offset) {
                            /* There is no chunk so voxel isn't blocked */
                            None => false,
                            
                            Some(chunk) => match chunk.get_voxel_global(pos) {
                                ChunkOption::OutsideChunk => unreachable!("Can't fall out of an adjacent chunk"),
                                ChunkOption::Item(voxel) => !voxel.is_air(),
                            }
                        }
                    },
                    
                    ChunkOption::Item(voxel) => !voxel.is_air(),
                };

                let is_blocked_subchunk = |offset: Int3| -> bool {
                    let start_pos = global_pos + offset * sub_chunk_size;
                    let end_pos   = global_pos + (offset + Int3::ONE) * sub_chunk_size;

                    let pred = |pos| is_blocking_voxel(pos, offset);
                    let mut iter = SpaceIter::new(start_pos..end_pos);

                    let is_on_surface = match offset.as_tuple() {
                        (-1, 0, 0) if 0 == local_pos.x => true,
                        (0, -1, 0) if 0 == local_pos.y => true,
                        (0, 0, -1) if 0 == local_pos.z => true,
                        (1, 0, 0) if Chunk::SIZE as i32 == local_pos.x + sub_chunk_size => true,
                        (0, 1, 0) if Chunk::SIZE as i32 == local_pos.y + sub_chunk_size => true,
                        (0, 0, 1) if Chunk::SIZE as i32 == local_pos.z + sub_chunk_size => true,
                        _ => false,
                    };
                    
                    match chunk_adj.sides.by_offset(offset) {
                        Some(_) if is_on_surface =>
                            iter.all(pred),
                        _ =>
                            iter.any(pred),
                    }
                };

                let mesh_builder = CubeLowered::new(sub_chunk_size as f32);
                
                const N_CUBE_VERTICES: usize = 36;
                let mut vertices = Vec::with_capacity(N_CUBE_VERTICES);

                for offset in SpaceIter::adj_iter(Int3::ZERO).filter(|&o| !is_blocked_subchunk(o)) {
                    mesh_builder.by_offset(offset, center_pos, voxel_color, &mut vertices);
                }

                vertices
            })
            .collect()
    }

    /// Givex voxel from global position.
    pub fn get_voxel_global(&self, global_pos: Int3) -> ChunkOption<Voxel> {
        let local_pos = Chunk::global_to_local_pos(self.pos.to_owned(), global_pos);

        if local_pos.x < 0 || local_pos.x >= Chunk::SIZE as i32 ||
           local_pos.y < 0 || local_pos.y >= Chunk::SIZE as i32 ||
           local_pos.z < 0 || local_pos.z >= Chunk::SIZE as i32
        { return ChunkOption::OutsideChunk }

        ChunkOption::Item(self.get_voxel_local(local_pos))
    }

    /// Gives voxel from local position (relative to chunk).
    pub fn get_voxel_local(&self, local_pos: Int3) -> Voxel {
        let global_pos = Chunk::local_to_global_pos(self.pos.to_owned(), local_pos);

        // FIXME: handle this more convinient.
        if !self.is_generated() {
            return Voxel::new(global_pos, AIR_VOXEL_DATA)
        }

        match self.meta_info.fill_type {
            FillType::Default => Voxel::new(
                global_pos,
                &VOXEL_DATA[self.voxel_ids[Chunk::voxel_pos_to_idx(local_pos)] as usize]
            ),

            FillType::AllSame(id) =>
                Voxel::new(global_pos, &VOXEL_DATA[id as usize]),
        }
    }

    /// Tests that chunk is visible by camera.
    pub fn is_visible_by_camera(&self, camera: &Camera) -> bool {
        const HALF_VOXEL_SIZE: f32 = cfg::terrain::VOXEL_SIZE * 0.5;

        let global_chunk_pos = Chunk::global_pos(self.pos.to_owned());

        let lo = vec3::from(global_chunk_pos)
               - vec3::all(HALF_VOXEL_SIZE);

        let hi = lo
               + vec3::from(Chunk::SIZES)
               - vec3::all(HALF_VOXEL_SIZE);

        camera.is_aabb_in_view(AABB::from_float3(lo, hi))
    }

    /// Checks if [`Chunk`] is not already generated.
    pub fn is_generated(&self) -> bool {
        !self.voxel_ids.is_empty()
    }
}

impl Chunk {
    pub const SIZE:   usize = cfg::terrain::CHUNK_SIZE;
    pub const SIZES: USize3 = USize3::all(Self::SIZE);
    pub const VOLUME: usize = Self::SIZE.pow(3);
    pub const N_LODS: usize = Self::SIZE.ilog2() as usize;

    /// Gives shared reference wrapper for chunk.
    pub fn make_ref(&self) -> ChunkRef<'_> {
        ChunkRef {
            pos: &self.pos,
            voxel_ids: &self.voxel_ids,
            meta_info: &self.meta_info,
        }
    }

    /// Gives mutable reference wrapper for chunk.
    pub fn make_mut(&mut self) -> ChunkMut<'_> {
        ChunkMut {
            pos: &mut self.pos,
            voxel_ids: &mut self.voxel_ids,
            meta_info: &mut self.meta_info,
        }
    }

    /// Generates voxel id array.
    pub fn generate_voxels(chunk_pos: Int3) -> Vec<Id> {
        Self::global_pos_iter(chunk_pos)
            .map(|pos| 
                if gen::trees(pos) {
                    LOG_VOXEL_DATA.id
                } else if gen::sine(pos) {
                    STONE_VOXEL_DATA.id
                } else {
                    AIR_VOXEL_DATA.id
                }
            )
            .collect()
    }

    /// Generates a chunk.
    pub fn new(chunk_pos: Int3) -> Self {
        Self::from_voxels(Self::generate_voxels(chunk_pos), chunk_pos)
    }

    /// Constructs empty chunk.
    pub fn new_empty(chunk_pos: Int3) -> Self {
        Self::from_voxels(vec![], chunk_pos)
    }

    pub fn new_same_filled(chunk_pos: Int3, fill_id: Id) -> Self {
        Self {
            voxel_ids: vec![fill_id],
            meta_info: MetaInfo {
                fill_type: FillType::AllSame(fill_id),
                active_lod: 0,
            },
            ..Self::new_empty(chunk_pos)
        }
    }

    pub fn from_voxels(voxel_ids: Vec<Id>, chunk_pos: Int3) -> Self {
        Self {
            pos: chunk_pos,
            voxel_ids,
            meta_info: Default::default(),
            detailed_mesh: None,

            // FIXME:
            low_meshes: [None, None, None, None, None, None],
        }.as_optimized()
    }

    /// Gives iterator over all id-vectors in chunk (or relative to chunk voxel positions).
    pub fn local_pos_iter() -> SpaceIter {
        SpaceIter::new(Int3::ZERO..Self::SIZES.into())
    }

    /// Gives iterator over all id-vectors in chunk (or relative to chunk voxel positions).
    pub fn global_pos_iter(chunk_pos: Int3) -> impl Iterator<Item = Int3> {
        Self::local_pos_iter()
            .map(move |pos| Self::local_to_global_pos(chunk_pos, pos))
    }

    /// Gives iterator that yields iterator over some chunk of voxels.
    pub fn chunked_pos_iter(sub_chunk_size: usize) -> impl Iterator<Item = SpaceIter> {
        SpaceIter::split_chunks(
            Self::SIZES.into(),
            Int3::all(sub_chunk_size as i32),
        )
    }

    /// Applies storage optimizations to voxel array.
    pub fn as_optimized(mut self) -> Self {
        if !self.is_generated() { return self }

        /* All-same pass */
        let sample_id = self.voxel_ids[0];
        if self.voxel_ids.iter().all(|&voxel_id| voxel_id == sample_id) {
            self.voxel_ids = vec![sample_id];
            self.meta_info.fill_type = FillType::AllSame(sample_id);
        }

        return self
    }

    /// Converts chunk position to world position.
    pub fn global_pos(chunk_pos: Int3) -> Int3 {
        chunk_pos * Self::SIZE as i32
    }

    /// Computes global position from relative to chunk position.
    pub fn local_to_global_pos(chunk_pos: Int3, relative_voxel_pos: Int3) -> Int3 {
        Self::global_pos(chunk_pos) + relative_voxel_pos
    }

    /// Computes local (relative to chunk) position from global position.
    pub fn global_to_local_pos(chunk_pos: Int3, global_voxel_pos: Int3) -> Int3 {
        global_voxel_pos - Self::global_pos(chunk_pos)
    }

    /// Gives index in voxel array by it's 3D-index (or relative to chunk position)
    pub fn voxel_pos_to_idx(pos: Int3) -> usize {
        sdex::get_index(&USize3::from(pos).as_array(), &[Self::SIZE; 3])
    }

    /// Sets mesh to chunk.
    pub fn upload_full_detail_vertices(&mut self, vertices: &[DetailedVertex], display: &gl::Display) {
        let vbuffer = VertexBuffer::no_indices(display, vertices, PrimitiveType::TrianglesList);
        let mesh = Mesh::new(vbuffer);
        self.detailed_mesh.replace(mesh);
    }

    /// Sets mesh to chunk.
    pub fn upload_low_detail_vertices(&mut self, vertices: &[LoweredVertex], lod: Lod, display: &gl::Display) {
        let vbuffer = VertexBuffer::no_indices(display, vertices, PrimitiveType::TrianglesList);
        let mesh = Mesh::new(vbuffer);
        self.low_meshes[lod as usize - 1].replace(mesh);
    }

    /// Generates and sets mesh to chunk.
    pub fn generate_mesh(&mut self, lod: Lod, chunk_adj: ChunkAdj, display: &gl::Display) {
        match lod {
            0 => {
                let vertices = self.make_vertices_detailed(chunk_adj);
                self.upload_full_detail_vertices(&vertices, display);
            },
            
            lod => {
                let vertices = self.make_vertices_low(chunk_adj, lod);
                self.upload_low_detail_vertices(&vertices, lod, display);
            }
        }
    }

    /// Renders a [`Chunk`].
    pub fn render(
        &self, target: &mut Frame, draw_info: &ChunkDrawBundle<'_>,
        uniforms: &impl Uniforms, lod: Lod,
    ) -> Result<(), ChunkRenderError> {
        if self.is_empty() { return Ok(()) }

        // TODO: If there no mesh just render a blank chunk

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

    /// Sets active LOD to given value.
    pub fn set_active_lod(&mut self, lod: Lod) {
        self.try_set_active_lod(lod)
            .expect("new LOD value should be available")
    }

    /// Tries to set active LOD to given value.
    pub fn try_set_active_lod(&mut self, lod: Lod) -> Result<(), SetLodError> {
        match self.get_available_lods().contains(&lod) {
            true => Ok(self.meta_info.active_lod = lod),
            false => Err(SetLodError::SetActiveLod { tried: lod, active: self.meta_info.active_lod }),
        }
    }

    /// Tries to set LOD value that have least difference with given value.
    /// If success it will return `Some(..)` with that value, otherwise, `None`.
    pub fn try_set_best_fit_lod(&mut self, lod: Lod) -> Option<Lod> {
        let best_fit = self.get_available_lods()
            .into_iter()
            .max_by(|&lhs, &rhs| {
                let lhs_diff = (lhs as isize - lod as isize).abs();
                let rhs_diff = (rhs as isize - lod as isize).abs();
                lhs_diff.cmp(&rhs_diff)
            })?;

        self.set_active_lod(best_fit);

        Some(best_fit)
    }

    /// Gives list of available LODs.
    pub fn get_available_lods(&self) -> Vec<Lod> {
        let mut result = Vec::with_capacity(Chunk::N_LODS);

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

    /// Gives list of all possible LODs.
    pub fn get_possible_lods() -> Vec<Lod> {
        (0 ..= Self::N_LODS as Lod).collect()
    }

    pub fn can_render_active_lod(&self) -> bool {
        self.get_available_lods()
            .contains(&self.meta_info.active_lod)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ChunkRef<'s> {
    pub pos: &'s Int3,
    pub voxel_ids: &'s Vec<Id>,
    pub meta_info: &'s MetaInfo,
}

#[derive(Debug)]
pub struct ChunkMut<'s> {
    pub pos: &'s mut Int3,
    pub voxel_ids: &'s mut Vec<Id>,
    pub meta_info: &'s mut MetaInfo,
}

impl From<ChunkRef<'_>> for Chunk {
    fn from(other: ChunkRef<'_>) -> Self {
        Self {
            pos: other.pos.clone(),
            voxel_ids: other.voxel_ids.clone(),
            meta_info: other.meta_info.clone(),
            detailed_mesh: None,
            low_meshes: [None, None, None, None, None, None],
        }
    }
}

impl From<ChunkMut<'_>> for Chunk {
    fn from(other: ChunkMut<'_>) -> Self {
        Self {
            pos: other.pos.clone(),
            voxel_ids: other.voxel_ids.clone(),
            meta_info: other.meta_info.clone(),
            detailed_mesh: None,
            low_meshes: [None, None, None, None, None, None],
        }
    }
}

impl<'r> From<&'r Chunk> for ChunkRef<'r> {
    fn from(value: &'r Chunk) -> Self {
        value.make_ref()
    }
}

impl<'r> From<&'r mut Chunk> for ChunkMut<'r> {
    fn from(value: &'r mut Chunk) -> Self {
        value.make_mut()
    }
}

#[derive(Error, Debug)]
pub enum SetLodError {
    #[error("failed to set LOD value to {tried} because there's no mesh for it. Active LOD value is {active}")]
    SetActiveLod {
        tried: Lod,
        active: Lod,
    },
}

#[derive(Error, Debug, Clone)]
pub enum ChunkRenderError {
    #[error(transparent)]
    GliumRender(#[from] DrawError),

    #[error("Expected a mesh with LOD value {0}")]
    NoMesh(Lod),

    #[error("Unexpectedly large LOD value: {0}")]
    TooBigLod(Lod),
}

#[derive(Debug)]
pub struct ChunkDrawBundle<'s> {
    full_shader: Shader,
    low_shader:  Shader,
    draw_params: gl::DrawParameters<'s>,
}

impl<'s> ChunkDrawBundle<'s> {
    pub fn new(display: &gl::Display) -> ChunkDrawBundle<'s> {
        /* Chunk draw parameters */
        let draw_params = gl::DrawParameters {
            depth: gl::Depth {
                test: gl::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            backface_culling: gl::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };
        
        /* Create shaders */
        let full_shader = Shader::new("full_detail", "full_detail", &display);
        let low_shader  = Shader::new("low_detail", "low_detail", &display);

        ChunkDrawBundle { full_shader, low_shader, draw_params }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MetaInfo {
    pub fill_type: FillType,
    pub active_lod: Lod,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FillType {
    #[default]
    Default,
    AllSame(Id),
}

unsafe impl Reinterpret for FillType { }

unsafe impl ReinterpretAsBytes for FillType {
    fn reinterpret_as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Default =>
                vec![0; Self::static_size()],

            Self::AllSame(id) => {
                let mut result = Vec::with_capacity(Self::static_size());
                result.push(1);
                result.append(&mut id.reinterpret_as_bytes());

                assert_eq!(result.capacity(), Self::static_size());

                return result
            },
        }
    }
}

unsafe impl ReinterpretFromBytes for FillType {
    fn reinterpret_from_bytes(source: &[u8]) -> Self {
        match source[0] {
            0 => Self::Default,
            1 => {
                let id = Id::reinterpret_from_bytes(&source[1..]);
                Self::AllSame(id)
            },
            _ => unreachable!("There's no FillType variant that matches with {} byte!", source[0])
        }
    }
}

unsafe impl ReinterpretSize for FillType {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for FillType {
    fn static_size() -> usize { u8::static_size() + Id::static_size() }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ChunkOption<T> {
    OutsideChunk,
    Item(T),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChunkAdj<'s> {
    pub sides: Sides<Option<ChunkRef<'s>>>,
}

impl ChunkAdj<'_> {
    pub fn none() -> Self {
        Self { sides: Sides::all(None) }
    }
}

pub type Lod = u32;

/// FIXME: turn into free function to prevent from conflicts, because [`Vec<u16>`].
unsafe impl StaticSize for Vec<Id> {
    fn static_size() -> usize {
        Chunk::VOLUME * u16::static_size()
    }
}
