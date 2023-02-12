pub mod chunk_array;
pub mod iterator;

use {
    crate::app::utils::{
        cfg,
        werror::prelude::*,
        graphics::{
            Graphics,
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
        generator,
    },
    math_linear::prelude::*,
    glium::{
        DrawError,
        uniforms::Uniforms,
        Frame,
        index::PrimitiveType,
        DrawParameters,
        implement_vertex,
    },
    std::{
        ptr::NonNull,
        cell::RefCell, marker::PhantomData,
        borrow::Cow, num::NonZeroU32, fmt::Display
    },
    iterator::{CubeBorder, SpaceIter},
};

/// Full-detailed vertex.
#[derive(Copy, Clone)]
pub struct DetailedVertex {
    pub position: (f32, f32, f32),
    pub tex_coords: (f32, f32),
    pub light: f32
}

/// Low-detailed vertex.
#[derive(Copy, Clone)]
pub struct LoweredVertex {
    pub position: (f32, f32, f32),
    pub color: (f32, f32, f32),
    pub light: f32
}

/* Implement Vertex structs as glium intended */
implement_vertex!(DetailedVertex, position, tex_coords, light);
implement_vertex!(LoweredVertex, position, color, light);

/**
 * Index cheatsheet!
 * 
 * i = d(hx + y) + z
 * 
 * x = (i / d) / h
 * y = (i / d) % h
 * z = i % d
 */

/// Type of voxel array. May be something different during progress.
type VoxelArray = Vec<Id>;
type Lod = u32;

pub enum ChunkOptional<Item> {
    OutsideChunk,
    Item(Item, Lod),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChunkFill {
    #[default]
    Standart,

    Empty,
    All(Id),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChunkDetails {
    #[default]
    Full,

    /// Low details with factor that represents how much chunk devided by 2.
    /// It means that side of chunk is devided by 2^factor.
    Low(NonZeroU32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChunkDetailsError {
    NoMoreDivisible { size: usize, level: Lod },
    FirstDivisionFailed { level: Lod },
    NotEnoughInformation,
}

impl Display for ChunkDetailsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::NoMoreDivisible { size, level } =>
                write!(f, "Chunk can not be lowered: chunk size is {size}, but detail level is {level}!"),
            Self::FirstDivisionFailed { level } =>
                write!(f, "First division failed: chunk size is {size}, but detail level is {level}!", size = MeshlessChunk::SIZE),
            Self::NotEnoughInformation =>
                write!(f, "Not enougth information passed!"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Addition {
    #[default]
    NothingToKnow,

    Know {
        fill: Option<ChunkFill>,
        details: ChunkDetails,
    },
}

impl AsRef<Self> for Addition {
    fn as_ref(&self) -> &Self { self }
}

impl AsMut<Self> for Addition {
    fn as_mut(&mut self) -> &mut Self { self }
}

/// Chunk struct.
pub struct MeshlessChunk {
    pub voxel_ids: VoxelArray,
    pub pos: Int3,
    pub additional_data: Addition,
}

impl MeshlessChunk {
    /// Predefined chunk values.
    pub const SIZE:	  usize = cfg::terrain::CHUNK_SIZE;
    pub const VOLUME: usize = Self::SIZE.pow(3);

    /// Constructs new chunk in given position 
    pub fn new(pos: Int3) -> Self {
        /* Voxel array initialization */
        let mut voxels = VoxelArray::with_capacity(Self::VOLUME);

        /* Additional data */
        let mut fill = ChunkFill::Empty;

        /* Iterating in the chunk */
        for curr in Self::pos_iter() {
            let global_pos = Self::local_to_global_pos(pos, curr);

            /* Update addidional chunk data */
            let mut update_data = |id| {
                match fill {
                    ChunkFill::Empty => {
                        /* Check for first iteration */
                        fill = if curr != Int3::zero() {
                            ChunkFill::Standart
                        } else {
                            ChunkFill::All(id)
                        }
                    },
                    ChunkFill::All(all_id) => {
                        if all_id != id {
                            fill = ChunkFill::Standart;
                        }
                    }
                    ChunkFill::Standart => (),
                }
            };

            /* Kind of trees generation */
            if generator::trees(global_pos) {
                let id = LOG_VOXEL_DATA.id;
                update_data(id);
                voxels.push(id);
            }
            
            /* Sine-like floor */
            else if generator::sine(global_pos) {
                let id = STONE_VOXEL_DATA.id;
                update_data(id);
                voxels.push(id);
            }
            
            /* Air */
            else {
                if let ChunkFill::All(_) = fill {
                    fill = ChunkFill::Standart
                }

                voxels.push(AIR_VOXEL_DATA.id)
            }
        }

        /* Chunk is empty so array can be empty */
        if let ChunkFill::Empty   = fill { voxels = vec![  ] }
        if let ChunkFill::All(id) = fill { voxels = vec![id] }
        
        // FIXME: Remove this:
        let mut chunk = MeshlessChunk {
            voxel_ids: voxels, pos,
            additional_data: Addition::Know {
                fill: Some(fill), details: ChunkDetails::Full,
            }
        };
        let abs = Float3::from(pos).len().round() as Lod;
        if abs >= 1 {
            chunk.lower_detail(abs.min((Self::SIZE as f32).log2() as Lod)).wunwrap();
        }
        return chunk
    }

    /// Constructs new empty chunk.
    pub fn new_empty(pos: Int3) -> Self {
        Self {
            pos, voxel_ids: vec![],
            additional_data: Addition::Know {
                fill: Some(ChunkFill::Empty),
                details: ChunkDetails::Full
            }
        }
    }

    /// Constructs new filled chunk.
    pub fn new_filled(pos: Int3, id: Id) -> Self {
        Self {
            pos, voxel_ids: vec![id],
            additional_data: Addition::Know {
                fill: Some(ChunkFill::All(id)),
                details: ChunkDetails::Full
            }
        }
    }

    /// Lowers details of chunk to given value.
    pub fn lower_detail(&mut self, level: Lod) -> Result<(), ChunkDetailsError> {
        /* Do nothing if level is zero */
        //* Also, this required by Safety arguments below.
        if level == 0 { return Ok(()) }

        //* Safety: this operation is safe until level is not zero.
        //* In previous step level had been checked so it is not zero.
        let level = unsafe { NonZeroU32::new_unchecked(level) };

        match self.additional_data.as_ref() {
            /* First division */
            Addition::Know { details: ChunkDetails::Full, fill } => {
                /* Check that chunk can be lowered */
                if Self::SIZE as Lod % (2 as Lod).pow(level.get()) != 0 {
                    return Err(ChunkDetailsError::FirstDivisionFailed { level: level.get() })
                }

                self.additional_data = Addition::Know { details: ChunkDetails::Low(level), fill: *fill };
            },

            /* Further more divisions */
            Addition::Know { details: ChunkDetails::Low(old_level), fill } => {
                /* Calculate old chunk size */
                let old_size = Self::SIZE / 2_usize.pow(old_level.get());

                /* Check that chunk can be lowered */
                if old_size as Lod % (2 as Lod).pow(level.get()) != 0 {
                    return Err(ChunkDetailsError::NoMoreDivisible { size: old_size, level: level.get() })
                }

                //* Safety: this operation is safe until level is not zero.
                //* Given value is not zero because it is the sum of positive values.
                let new_level = unsafe { NonZeroU32::new_unchecked(old_level.get() + level.get()) };

                self.additional_data = Addition::Know { details: ChunkDetails::Low(new_level), fill: *fill };
            }
            
            /* Nothing presented */
            Addition::NothingToKnow => return Err(ChunkDetailsError::NotEnoughInformation),
        }

        return Ok(())
    }
    
    pub fn set_lod_data(&mut self, mut lod: Lod) -> Result<(), ChunkDetailsError> {
        if Self::SIZE as Lod % (2 as Lod).pow(lod) != 0 {
            // TODO: calculate maximum lod, then return error with that value
            lod = (Self::SIZE as f32).log2().floor() as Lod;
            //return Err(ChunkDetailsError::NoMoreDivisible { size: Self::SIZE, level: lod })
        }

        match self.additional_data.as_mut() {
            Addition::NothingToKnow =>
                return Err(ChunkDetailsError::NotEnoughInformation),
            
            Addition::Know { ref mut details, .. } => if lod == 0 {
                *details = ChunkDetails::Full;
            } else {
                // * Safety: creating non-zero u32 is safe here 'cause zero is already checked.
                let lod_non_zero = unsafe { NonZeroU32::new_unchecked(lod) };
                *details = ChunkDetails::Low(lod_non_zero);
            }
        }

        return Ok(())
    }

    /// Checks if chunk is empty by its data.
    pub fn is_empty(&self) -> bool {
        match self.additional_data {
            Addition::Know { fill: Some(ChunkFill::Empty), .. } => true,
            _ => false,
        }
    }

    /// Checks if chunk is empty by its data.
    pub fn is_filled(&self) -> bool {
        match self.additional_data {
            Addition::Know { fill: Some(ChunkFill::All(_)), .. } => true,
            _ => false,
        }
    }

    /// Gives fill id if available.
    pub fn fill_id(&self) -> Option<Id> {
        if let Addition::Know { fill: Some(ChunkFill::All(id)), .. } = self.additional_data {
            Some(id)
        } else { None }
    }

    pub fn make_full_detailed_vertices_filled(&self, env: &ChunkEnvironment, id: Id) -> DetailedVertexVec {
        /* Cycle over all coordinates in chunk */
        let mut vertices = vec![];
        for pos in CubeBorder::new(Self::SIZE as i32) {
            self.to_triangles_inner_detailed(pos, id, env, &mut vertices);
        }

        /* Shrink vector */
        vertices.shrink_to_fit();

        return Detailed::Full(vertices)
    }

    pub fn make_full_detailed_vertices_standart(&self, env: &ChunkEnvironment) -> DetailedVertexVec {
        /* Construct vertex array */
        let mut vertices = vec![];

        self.voxel_ids.iter()
            .enumerate()
            .map(|(i, &id)| (idx_to_pos(i), id))
            .for_each(|(pos, id)|
                self.to_triangles_inner_detailed(pos, id, env, &mut vertices)
            );

        /* Shrink vector */
        vertices.shrink_to_fit();

        return Detailed::Full(vertices)
    }

    pub fn make_lowered_detail_vertices(&self, env: &ChunkEnvironment, lod: NonZeroU32) -> DetailedVertexVec {
        /* Resulting vertex array */
        let mut vertices = vec![];
        
        /* Side length of low-res voxel in world units */
        let voxel_size = 2_usize.pow(lod.get());
        
        /* Side length of low-res chunk in low-res voxels */
        let low_side_len = Self::SIZE / voxel_size;

        /* Array of low-res voxels */
        let lowered = self.get_lowered_voxels(lod).wunwrap();

        lowered.iter()
            .enumerate()
            .map(|(i, lowered_voxel)| {
                use cfg::terrain::VOXEL_SIZE;

                /* Find `position` in 'smaller chunk' */
                let pos_low = general_position(i, low_side_len, low_side_len);
                let mut pos = vec3::from(pos_low);

                /* Then stretch it and center */
                pos += vec3::all(VOXEL_SIZE * 0.5);
                pos *= voxel_size as f32;
                pos -= vec3::all(VOXEL_SIZE * 0.5);

                /* Move to world coordinates */
                let pos = pos_in_chunk_to_world_float3(pos, self.pos.into());

                /* Return lowered chunk position in world, 
                |* position in lowered array and lowered chunk itself */
                return (pos, pos_low, lowered_voxel)
            })
            .for_each(|(pos, pos_low, low)|
                self.to_triangles_inner_lowered(pos, pos_low, lod, low, env, &mut vertices)
            );

        return Detailed::Low(vertices)
    }

    /// Creates trianlges Vec from Chunk and its environment.
    pub fn to_triangles(&self, env: &ChunkEnvironment) -> DetailedVertexVec {
        match self.additional_data.as_ref() {
            /* Empty chunk passed in */
            Addition::Know { fill: Some(ChunkFill::Empty), details } => return match details {
                ChunkDetails::Full => Detailed::Full(vec![]),
                ChunkDetails::Low(_) => Detailed::Low(vec![]),
            },

            /* "Filled" chunk with full details passed in */
            Addition::Know { fill: Some(ChunkFill::All(id)), details: ChunkDetails::Full } =>
                return self.make_full_detailed_vertices_filled(env, *id),

            /* Standart chunk with full details passed in */
            Addition::Know { fill: Some(ChunkFill::Standart), details: ChunkDetails::Full } =>
                self.make_full_detailed_vertices_standart(env),

            /* Lowered details uniplemented */
            Addition::Know { details: ChunkDetails::Low(lod), fill: _ } =>
                self.make_lowered_detail_vertices(env, *lod),

            /* Not enough information */
            not_enough @ Addition::NothingToKnow | not_enough @ Addition::Know { fill: None, .. } =>
                panic!("No required information passed into mesh builder! {addition:?}", addition = not_enough),
        }
    }

    fn to_triangles_inner_detailed(&self, in_chunk_pos: Int3, id: Id, env: &ChunkEnvironment, vertices: &mut Vec<DetailedVertex>) {
        if id == AIR_VOXEL_DATA.id { return }
        
        /* Get position from index */
        let position = pos_in_chunk_to_world_int3(in_chunk_pos, self.pos);
        
        /* Draw checker */
        let is_drawable = |input: Option<Voxel>| -> bool {
            match input {
                None => true,
                Some(voxel) => voxel.data == AIR_VOXEL_DATA,
            }
        };
        
        /* Mesh builder */
        let build = |bias, env: Option<NonNull<MeshlessChunk>>| {
            if !is_drawable(self.get_voxel_or_none(position + bias)) { return false }

            match env {
                None => true,
                Some(chunk) => {
                    //* Safety: Safe because environment chunks lives as long as other chunks or that given chunk.
                    //* And it also needs only at chunk generation stage.
                    is_drawable(unsafe { chunk.as_ref().get_voxel_or_none(position + bias) })
                }
            }
        };

        /* Cube vertices generator */
        let cube = CubeDetailed::new(&VOXEL_DATA[id as usize]);
        
        /* Build all sides separately */
        if build(veci!( 1,  0,  0), env.back)   { cube.back  (position, vertices) };
        if build(veci!(-1,  0,  0), env.front)  { cube.front (position, vertices) };
        if build(veci!( 0,  1,  0), env.top)    { cube.top   (position, vertices) };
        if build(veci!( 0, -1,  0), env.bottom) { cube.bottom(position, vertices) };
        if build(veci!( 0,  0,  1), env.right)  { cube.right (position, vertices) };
        if build(veci!( 0,  0, -1), env.left)   { cube.left  (position, vertices) };
    }

    fn is_blocked_with_lod(&self, low_pos: Int3, side: Int3, lod: NonZeroU32, neighbor: Option<NonNull<MeshlessChunk>>) -> bool {
        let voxel_size = 2_usize.pow(lod.get()) as i32;
        let neighbor_min_pos = (low_pos + side) * voxel_size;
        let neighbor_max_pos = (low_pos + side) * voxel_size + Int3::all(voxel_size);
        
        let neighbor_lod = {
            let neighbor_min_world_pos = pos_in_chunk_to_world_int3(neighbor_min_pos, self.pos);
            match self.get_voxel(neighbor_min_world_pos) {
                ChunkOptional::Item(_, lod) => lod,
                ChunkOptional::OutsideChunk => match neighbor {
                    /* No voxel found means no LOD */
                    None => 0,

                    // TODO: add safety argument.
                    Some(chunk) => match unsafe { chunk.as_ref() }.get_voxel(neighbor_min_world_pos) {
                        ChunkOptional::Item(_, lod) => lod,
                        ChunkOptional::OutsideChunk => 0,
                    },
                },
            }
        };

        let small_blocked = |pos: Int3| -> bool {
            match self.get_voxel(pos) {
                ChunkOptional::Item(voxel, _) =>
                    voxel.data != AIR_VOXEL_DATA,

                ChunkOptional::OutsideChunk => match neighbor {
                    None => false,
                    Some(chunk) => match unsafe { chunk.as_ref() }.get_voxel(pos) {
                        ChunkOptional::Item(voxel, _) => voxel.data != AIR_VOXEL_DATA,
                        ChunkOptional::OutsideChunk => false,
                    },
                }
            }
        };
        
        let mut pos_iter = SpaceIter::new(neighbor_min_pos..neighbor_max_pos)
            .map(|pos| Self::local_to_global_pos(self.pos, pos));

        if lod.get() <= neighbor_lod {
            pos_iter.any(small_blocked)
        } else {
            pos_iter.all(small_blocked)
        }
    }

    fn to_triangles_inner_lowered(
        &self, build_pos: vec3, low_pos: Int3, lod: NonZeroU32,
        low_voxel: &LoweredVoxel, env: &ChunkEnvironment, vertices: &mut Vec<LoweredVertex>
    ) {
        let voxel_color = match low_voxel {
            LoweredVoxel::Transparent => return,
            LoweredVoxel::Colored(color) => *color,
        };

        let voxel_size_f32 = cfg::terrain::VOXEL_SIZE * 2_usize.pow(lod.get()) as f32;
        let cube_mesher = CubeLowered::new(voxel_size_f32);

        if !self.is_blocked_with_lod(low_pos, veci!( 1,  0,  0), lod, env.back)   { cube_mesher  .back(build_pos, voxel_color, vertices); }
        if !self.is_blocked_with_lod(low_pos, veci!(-1,  0,  0), lod, env.front)  { cube_mesher .front(build_pos, voxel_color, vertices); }
        if !self.is_blocked_with_lod(low_pos, veci!( 0,  1,  0), lod, env.top)    { cube_mesher   .top(build_pos, voxel_color, vertices); }
        if !self.is_blocked_with_lod(low_pos, veci!( 0, -1,  0), lod, env.bottom) { cube_mesher.bottom(build_pos, voxel_color, vertices); }
        if !self.is_blocked_with_lod(low_pos, veci!( 0,  0,  1), lod, env.right)  { cube_mesher .right(build_pos, voxel_color, vertices); }
        if !self.is_blocked_with_lod(low_pos, veci!( 0,  0, -1), lod, env.left)   { cube_mesher  .left(build_pos, voxel_color, vertices); }
    }

    fn get_lowered_voxels(&self, lod: NonZeroU32) -> Result<Vec<LoweredVoxel>, String> {
        /* Divide factor of chunk */
        let factor = 2_usize.pow(lod.get());

        /* Side length of low-res chunk in low-res voxels */
        let new_size = Self::SIZE / factor;

        /* Volume of low-res voxels array in low-res voxels */
        let new_volume = Self::VOLUME / factor.pow(3);
        
        match self.additional_data.as_ref() {
            /* Chunk was empty */
            Addition::Know { fill: Some(ChunkFill::Empty), details: _ } =>
                return Ok(vec![]),

            /* Chunk was filled with the save voxel */
            Addition::Know { fill: Some(ChunkFill::All(id)), details: _ } => {
                let voxel = LoweredVoxel::Colored(VOXEL_DATA[*id as usize].avarage_color);
                return Ok(vec![voxel; new_volume])
            },

            /* Chunk was standart-filled */
            Addition::Know { fill: Some(ChunkFill::Standart), details: _ } => {
                /* Resulting array of loweres voxels */
                let mut result = vec![LoweredVoxel::Transparent; new_volume];

                /* Write count array each element of is the denominator of avarage count */
                let mut n_writes = vec![0; new_volume];

                let iter = (0..Self::VOLUME).map(|i| {
                    let pos = idx_to_pos(i);
                    let id = self.voxel_ids[i];
                    return (id, pos / factor as i32)
                });

                /* Low-res array index function */
                let low_index = |pos: Int3|
                    sdex::get_index(&[pos.x() as usize, pos.y() as usize, pos.z() as usize], &[new_size; 3]);

                for (id, new_pos) in iter {
                    /* Lowered voxels index shortcut */
                    let low_i = low_index(new_pos);

                    /* Writes count shortcut */
                    let count = n_writes[low_i] as f32;

                    /* Air blocks are not in count.
                    |* Leaves empty voxels as [`LoweredVoxel::Transparent`] */
                    if id != AIR_VOXEL_DATA.id {
                        match &mut result[low_i] {
                            /* If voxel is already initialyzed with some color */
                            LoweredVoxel::Colored(color) => {
                                /* Color shortcut */
                                let new_color = VOXEL_DATA[id as usize].avarage_color;

                                /* Calculate new avarage color */
                                *color = (*color * count + new_color) / (count + 1.0);

                                /* Increment writes count */
                                n_writes[low_i] += 1;
                            },

                            /* If voxel going to be initialyzed */
                            LoweredVoxel::Transparent => {
                                result[low_i] = LoweredVoxel::Colored(VOXEL_DATA[id as usize].avarage_color);
                                n_writes[low_i] = 1;
                            }
                        }
                    }
                }

                return Ok(result)
            },

            /* Not enough information */
            not_enough @ Addition::Know { fill: None, details: _ } |
            not_enough @ Addition::NothingToKnow => {
                return Err(format!(
                    "Not enough information! Addition was: {addition:?}",
                    addition = not_enough
                ))
            }
        }
    }

    /// Gives voxel by world coordinate.
    pub fn get_voxel(&self, global_pos: Int3) -> ChunkOptional<Voxel> {
        /* Transform to local */
        let pos = world_coords_to_in_some_chunk(global_pos, self.pos);
        
        if pos.x < 0 || pos.x >= Self::SIZE as i32 ||
           pos.y < 0 || pos.y >= Self::SIZE as i32 ||
           pos.z < 0 || pos.z >= Self::SIZE as i32
        { return ChunkOptional::OutsideChunk }
        
        match self.additional_data.as_ref() {
            /* For empty chunks */
            Addition::Know { fill: Some(ChunkFill::Empty), details } => {
                let voxel = Voxel::new(global_pos, &AIR_VOXEL_DATA);
                let lod = match details {
                    ChunkDetails::Low(lod) => lod.get(),
                    ChunkDetails::Full     => 0,
                };

                return ChunkOptional::Item(voxel, lod)
            }

            /* For known-filled chunks */
            Addition::Know { fill: Some(fill), details } => {
                let voxel = match fill {
                    ChunkFill::Standart =>
                        Voxel::new(global_pos, &VOXEL_DATA[self.voxel_ids[pos_to_idx(pos)] as usize]),
                        
                    ChunkFill::All(id) =>
                        Voxel::new(global_pos, &VOXEL_DATA[*id as usize]),

                    ChunkFill::Empty =>
                        Voxel::new(global_pos, &AIR_VOXEL_DATA),
                };

                let lod = match details {
                    ChunkDetails::Full     => 0,
                    ChunkDetails::Low(lod) => lod.get(),
                };

                return ChunkOptional::Item(voxel, lod)
            }

            /* No information */
            Addition::NothingToKnow => panic!("Not enough information!"),

            /* Other types */
            addition => panic!("Unresolvable chunk Addition {:?}!", addition),
        }
    }

    /// Gives voxel by world coordinate.
    pub fn get_voxel_or_none(&self, pos: Int3) -> Option<Voxel> {
        match self.get_voxel(pos) {
            ChunkOptional::Item(voxel, _) => Some(voxel),
            ChunkOptional::OutsideChunk => None,
        }
    }

    /// Checks if chunk is in camera view.
    pub fn is_visible(&self, camera: &Camera) -> bool {
        /* AABB init */
        let mut lo = vec3::from(Self::chunk_to_global_pos(self.pos));
        let mut hi = lo + vec3::all(Self::SIZE as f32);

        /* Bias (voxel centration) */
        const BIAS: f32 = cfg::terrain::VOXEL_SIZE * 0.5;

        /* Biasing */
        lo -= vec3::all(BIAS);
        hi -= vec3::all(BIAS);

        /* Frustum check */
        camera.is_aabb_in_view(AABB::from_float3(lo, hi))
    }

    /// Upgrades chunk to be meshed.
    #[allow(dead_code)]
    pub fn envs_upgrade(self, graphics: &Graphics, env: &ChunkEnvironment) -> MeshedChunk {
        MeshedChunk::from_meshless_envs(graphics, self, env)
    }

    /// Upgrades chunk to be meshed.
    pub fn triangles_upgrade(self, graphics: &Graphics, vertices: DetailedVertexSlice) -> MeshedChunk {
        MeshedChunk::from_meshless_triangles(graphics, self, vertices)
    }

    /// Gives position iterator that gives position for all voxels in chunk.
    /// Internally, calls `SpaceIter::zeroed_cubed(CHUNK_SIZE as i32)`.
    pub fn pos_iter() -> SpaceIter {
        SpaceIter::zeroed_cubed(Self::SIZE as i32)
    }

    /// Computes global position from relative to chunk position.
    pub fn local_to_global_pos(chunk_absolute_pos: Int3, relative_voxel_pos: Int3) -> Int3 {
        pos_in_chunk_to_world_int3(relative_voxel_pos, chunk_absolute_pos)
    }

    /// Calculates chunk position relative to world from chunks position.
    pub fn chunk_to_global_pos(chunk_pos: Int3) -> Int3 {
        chunk_coords_to_min_world_int3(chunk_pos)
    }

    /// Gives LOD.
    pub fn get_lod(&self) -> Lod {
        match self.additional_data.as_ref() {
            Addition::NothingToKnow => 0,
            Addition::Know { details, .. } => match details {
                ChunkDetails::Full => 0,
                ChunkDetails::Low(lod) => lod.get(),
            },
        }
    }
}

/// Describes blocked chunks by environent or not.
#[derive(Clone, Default)]
pub struct ChunkEnvironment<'s> {
    pub top:	Option<NonNull<MeshlessChunk>>,
    pub bottom:	Option<NonNull<MeshlessChunk>>,
    pub front:	Option<NonNull<MeshlessChunk>>,
    pub back:	Option<NonNull<MeshlessChunk>>,
    pub left:	Option<NonNull<MeshlessChunk>>,
    pub right:	Option<NonNull<MeshlessChunk>>,

    pub _phantom: PhantomData<&'s MeshlessChunk>
}

impl<'c> ChunkEnvironment<'c> {
    /// Empty description.
    pub fn none() -> Self {
        ChunkEnvironment {
            top: None, bottom: None, front: None, back: None,
            left: None, right: None, _phantom: PhantomData
        }
    }
}

pub enum Detailed<FullType, LowType> {
    Full(FullType),
    Low(LowType),
}

pub type DetailedVertexSlice<'v> = Detailed<&'v [DetailedVertex], &'v [LoweredVertex]>;
pub type DetailedVertexVec = Detailed<Vec<DetailedVertex>, Vec<LoweredVertex>>;

pub struct ChunkMesh(Detailed<
    RefCell<UnindexedMesh<DetailedVertex>>,
    RefCell<UnindexedMesh<LoweredVertex>>
>);

impl ChunkMesh {
    /// Render mesh.
    pub fn render(
        &self, target: &mut Frame, full_shader: &Shader, low_shader: &Shader,
        draw_params: &DrawParameters<'_>, uniforms: &impl Uniforms) -> Result<(), DrawError>
    {
        match &self.0 {
            Detailed::Full(mesh) => mesh.borrow().render(target, full_shader, draw_params, uniforms),
            Detailed::Low(mesh)  => mesh.borrow().render(target, low_shader,  draw_params, uniforms),
        }
    }

    /// Checks if mesh is empty.
    pub fn is_empty(&self) -> bool {
        match &self.0 {
            Detailed::Full(mesh) => mesh.borrow().is_empty(),
            Detailed::Low(mesh)  => mesh.borrow().is_empty()
        }
    }
}

/// Chunk struct.
pub struct MeshedChunk {
    pub inner: MeshlessChunk,
    pub mesh: ChunkMesh,
}

impl MeshedChunk {
    /// Constructs new chunk in given position.
    #[allow(dead_code)]
    pub fn from_envs(graphics: &Graphics, pos: Int3, env: &ChunkEnvironment) -> Self {
        /* Construct new meshless */
        let meshless = MeshlessChunk::new(pos);
        
        /* Upgrade it */
        Self::from_meshless_envs(graphics, meshless, env)
    }

    /// Constructs mesh for meshless chunk.
    #[allow(dead_code)]
    pub fn from_meshless_envs(graphics: &Graphics, meshless: MeshlessChunk, env: &ChunkEnvironment) -> Self {
        /* Create chunk */
        let triangles = meshless.to_triangles(env);
        let triangles = match &triangles {
            Detailed::Full(vec) => Detailed::Full(&vec[..]),
            Detailed::Low(vec) => Detailed::Low(&vec[..]),
        };

        MeshedChunk {
            inner: meshless,
            mesh: Self::make_mesh(&graphics.display, triangles)
        }
    }

    /// Constructs mesh for meshless chunk.
    pub fn from_meshless_triangles(graphics: &Graphics, meshless: MeshlessChunk, vertices: Detailed<&[DetailedVertex], &[LoweredVertex]>) -> Self {
        MeshedChunk {
            inner: meshless,
            mesh: Self::make_mesh(&graphics.display, vertices)
        }
    }

    /// Renders chunk.
    /// * Mesh should be constructed before this function call.
    pub fn render<U: Uniforms>(
        &self, target: &mut Frame, full_shader: &Shader, low_shader: &Shader,
        uniforms: &U, draw_params: &glium::DrawParameters, camera: &Camera) -> Result<(), DrawError>
    {
        if self.mesh.is_empty() || !self.is_visible(camera) {
            return Ok(());
        }

        return self.mesh.render(target, full_shader, low_shader, draw_params, uniforms)
    }

    pub fn make_mesh(display: &glium::Display, vertices: Detailed<&[DetailedVertex], &[LoweredVertex]>) -> ChunkMesh {
        match vertices {
            Detailed::Full(vertices) => {
                /* Vertex buffer for chunks */
                let vertex_buffer = VertexBuffer::no_indices(display, vertices, PrimitiveType::TrianglesList);

                ChunkMesh(Detailed::Full(RefCell::new(Mesh::new(vertex_buffer))))
            },

            Detailed::Low(vertices) => {
                /* Vertex buffer for chunks */
                let vertex_buffer = VertexBuffer::no_indices(display, vertices, PrimitiveType::TrianglesList);

                ChunkMesh(Detailed::Low(RefCell::new(Mesh::new(vertex_buffer))))
            }
        }
    }

    pub fn make_mesh_owned(display: &glium::Display, vertices: Detailed<Vec<DetailedVertex>, Vec<LoweredVertex>>) -> ChunkMesh {
        match vertices {
            Detailed::Full(vertices) => Self::make_mesh(display, Detailed::Full(&vertices)),
            Detailed::Low(vertices)  => Self::make_mesh(display, Detailed::Low(&vertices)),
        }
    }

    /// Creates trianlges Vec from Chunk and its environment.
    #[allow(dead_code)]
    pub fn to_triangles(&self, env: &ChunkEnvironment) -> Detailed<Vec<DetailedVertex>, Vec<LoweredVertex>> {
        self.inner.to_triangles(env)
    }

    /// Gives voxel by world coordinate.
    #[allow(dead_code)]
    pub fn get_voxel_optional(&self, global_pos: Int3) -> ChunkOptional<Voxel> {
        self.inner.get_voxel(global_pos)
    }

    /// Gives voxel by world coordinate.
    #[allow(dead_code)]
    pub fn get_voxel_or_none(&self, pos: Int3) -> Option<Voxel> {
        self.inner.get_voxel_or_none(pos)
    }

    /// Checks if chunk is in camera view.
    pub fn is_visible(&self, camera: &Camera) -> bool {
        self.inner.is_visible(camera)
    }

    pub fn update_details_data(&mut self, camera_pos: vec3) -> bool {
        // TODO: check nearby chunks to update their meshes

        let new_lod = self.calculate_desired_lod(camera_pos);
        //if self.inner.get_lod() == new_lod { return false }

        self.inner.set_lod_data(new_lod).expect("Can't set LOD data in .update_details_data(..)!");
        return true
    }

    pub fn refresh_mesh(&mut self, display: &glium::Display, env: &ChunkEnvironment) {
        let vertices = self.inner.to_triangles(env);
        self.mesh = Self::make_mesh_owned(display, vertices);
    }

    /// Calculates best-fit LOD value.
    fn calculate_desired_lod(&self, camera_pos: vec3) -> Lod {
        let max_lod: Lod = (MeshlessChunk::SIZE as f32).log2().floor() as Lod;

        let chunk_pos = vec3::from(
            MeshlessChunk::chunk_to_global_pos(self.inner.pos) +
            Int3::all(MeshlessChunk::SIZE as i32) / 2
        );

        let distance = (chunk_pos - camera_pos).len();

        const DIST_MULTIPLIER: f32 = 0.006;
        let lod = (distance * DIST_MULTIPLIER).floor() as Lod;

        return max_lod.min(lod)
    }
}

/// FIXME: turn into free function to prevent from conflicts, because [`VoxelArray`] = [`Vec<u16>`].
unsafe impl StaticSize for VoxelArray {
    fn static_size() -> usize {
        MeshlessChunk::VOLUME * u16::static_size()
    }
}

unsafe impl Reinterpret for MeshlessChunk { }

unsafe impl ReinterpretAsBytes for MeshlessChunk {
    fn reinterpret_as_bytes(&self) -> Vec<u8> {
        let voxels = match self.additional_data.as_ref() {
            Addition::Know { fill: Some(ChunkFill::Empty), .. } => {
                Cow::Owned(vec![AIR_VOXEL_DATA.id; MeshlessChunk::VOLUME])
            },
            Addition::Know { fill: Some(ChunkFill::All(id)), .. } => {
                Cow::Owned(vec![*id; MeshlessChunk::VOLUME])
            },
            addition => {
                assert_eq!(
                    self.voxel_ids.len(), MeshlessChunk::VOLUME,
                    "Unresolvable array! Addition is {:?}", addition
                );
                Cow::Borrowed(&self.voxel_ids)
            }
        };

        let mut bytes = Vec::with_capacity(Self::static_size());

        bytes.append(&mut voxels.reinterpret_as_bytes());
        bytes.append(&mut self.pos.reinterpret_as_bytes());

        return bytes
    }
}

unsafe impl ReinterpretFromBytes for MeshlessChunk {
    fn reinterpret_from_bytes(source: &[u8]) -> Self {
        let voxel_array_size: usize = VoxelArray::static_size();

        let voxels = VoxelArray::reinterpret_from_bytes(&source[.. voxel_array_size]);
        let pos = Int3::reinterpret_from_bytes(&source[voxel_array_size .. voxel_array_size + Int3::static_size()]);

        MeshlessChunk { voxel_ids: voxels, pos, additional_data: Default::default() }
    }
}

unsafe impl ReinterpretSize for MeshlessChunk {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for MeshlessChunk {
    fn static_size() -> usize {
        VoxelArray::static_size() + Int3::static_size() + 1
    }
}

unsafe impl Reinterpret for ChunkFill { }

unsafe impl ReinterpretAsBytes for ChunkFill {
    fn reinterpret_as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Empty => vec![0; Self::static_size()],
            Self::Standart => {
                let mut result = vec![0; Self::static_size()];
                result[0] = 2;

                return result
            },
            Self::All(id) => {
                let mut result = vec![1];
                result.append(&mut id.reinterpret_as_bytes());

                return result
            },
        }
    }
}

unsafe impl ReinterpretFromBytes for ChunkFill {
    fn reinterpret_from_bytes(source: &[u8]) -> Self {
        match source[0] {
            0 => Self::Empty,
            1 => {
                let id = Id::reinterpret_from_bytes(&source[1..]);
                return Self::All(id)
            },
            2 => Self::Standart,
            _ => unreachable!("There's no ChunkFill variant that matches with {}!", source[0])
        }
    }
}

unsafe impl ReinterpretSize for ChunkFill {
    fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for ChunkFill {
    fn static_size() -> usize { u8::static_size() + Id::static_size() }
}

#[cfg(test)]
mod reinterpret_test {
    use super::*;

    #[test]
    fn reinterpret_chunk() {
        let before = MeshlessChunk::new(Int3::new(12, 12, 11));
        let after = MeshlessChunk::reinterpret_from_bytes(&before.reinterpret_as_bytes());

        assert_eq!(before.voxel_ids, after.voxel_ids);
        assert_eq!(before.pos, after.pos);
    }
}



/// Transforms world coordinates to chunk.
#[allow(dead_code)]
pub fn world_coords_to_chunk(pos: Int3) -> Int3 {
    pos / MeshlessChunk::SIZE as i32
}

/// Transforms chunk coords to world.
pub fn chunk_coords_to_min_world_int3(pos: Int3) -> Int3 {
    pos * MeshlessChunk::SIZE as i32
}

/// Transforms chunk coords to world.
pub fn chunk_coords_to_min_world_float3(pos: Float3) -> Float3 {
    pos * MeshlessChunk::SIZE as f32
}

/// Transforms in-chunk coords to world.
pub fn pos_in_chunk_to_world_int3(in_chunk: Int3, chunk: Int3) -> Int3 {
    chunk_coords_to_min_world_int3(chunk) + in_chunk
}

/// Transforms in-chunk coords to world.
pub fn pos_in_chunk_to_world_float3(in_chunk: Float3, chunk: Float3) -> Float3 {
    chunk_coords_to_min_world_float3(chunk) + in_chunk
}

/// Transforms world coordinates to chunk.
#[allow(dead_code)]
pub fn world_coords_to_in_chunk(pos: Int3) -> Int3 {
    let chunk_size = Int3::all(MeshlessChunk::SIZE as i32);
    return (pos % chunk_size + chunk_size) % chunk_size
}

/// Transforms world coordinates to chunk.
pub fn world_coords_to_in_some_chunk(pos: Int3, chunk: Int3) -> Int3 {
    pos - chunk_coords_to_min_world_int3(chunk)
}

/// Index function.
pub fn pos_to_idx(pos: Int3) -> usize {
    sdex::get_index(&USize3::from(pos).as_array(), &[MeshlessChunk::SIZE; 3])
}

/// Position function.
pub fn idx_to_pos(i: usize) -> Int3 {
    general_position(i, MeshlessChunk::SIZE, MeshlessChunk::SIZE)
}

/// Position function.
pub fn general_position(i: usize, height: usize, depth: usize) -> Int3 {
    let xy = i / depth;

    let z =  i % depth;
    let y = xy % height;
    let x = xy / height;

    veci!(x, y, z)
}