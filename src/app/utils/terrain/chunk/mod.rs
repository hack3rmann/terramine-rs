pub mod iterator;
pub mod chunk_array;
pub mod tasks;
pub mod commands;
pub mod mesh;

use {
    crate::{
        prelude::*,
        graphics::{
            camera_resource::Camera,
            Mesh, Device, RenderPipeline, RenderPass, Render,
        },
    },
    super::voxel::{
        self,
        Voxel,
        LoweredVoxel,
        shape::{CubeDetailed, CubeLowered},
        voxel_data::{data::*, Id},
        generator as gen,
    },
    mesh::{FullVertex, LowVertex, ChunkMesh},
    chunk_array::ChunkAdj,
    iterator::{CubeBoundary, Sides},
};



pub mod prelude {
    pub use super::{
        Chunk,
        SetLodError,
        ChunkRenderError,
        Info as ChunkInfo,
        Lod,
        ChunkOption,
        FillType,
        chunk_array::ChunkArray,
        iterator::{SpaceIter, self},
    };
}



#[derive(Debug)]
pub struct Chunk {
    pub pos: Atomic<Int3>,
    pub voxel_ids: Vec<Atomic<Id>>,
    pub info: Atomic<Info>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxel_ids: default(),
            pos: default(),
            info: Atomic::new(Info {
                fill_type: FillType::AllSame(AIR_VOXEL_DATA.id),
                is_filled: true,
                active_lod: None,
            }),
        }
    }
}

impl Chunk {
    /// [Chunk] size in voxels.
    pub const SIZE: usize = cfg::terrain::CHUNK_SIZE;

    /// [Chunk] sizes in voxels.
    pub const SIZES: USize3 = USize3::all(Self::SIZE);

    /// [Chunk] volume in voxels.
    pub const VOLUME: usize = Self::SIZE.pow(3);

    /// Number of available [LOD][Lod]s.
    pub const N_LODS: usize = Self::SIZE.ilog2() as usize;

    /// [Chunk] size in global units.
    pub const GLOBAL_SIZE: f32 = Self::SIZE as f32 * Voxel::SIZE;
    
    /// Gives iterator over all voxels in chunk.
    pub fn voxels(&self) -> impl ExactSizeIterator<Item = Voxel> + '_ {
        self.voxel_ids.iter()
            .map(|id| id.load(Relaxed))
            .zip(Chunk::global_pos_iter(self.pos.load(Relaxed)))
            .map(|(id, pos)| Voxel::new(pos, &VOXEL_DATA[id as usize]))
    }

    /// Gives iterator over low-detail voxels with their coords.
    pub fn low_voxel_iter(&self, lod: Lod) -> impl ExactSizeIterator<Item = (LoweredVoxel, Int3)> + '_ {
        let sub_chunk_size = 2_i32.pow(lod);

        Chunk::chunked_pos_iter(sub_chunk_size as usize)
            .map(move |chunk_iter| {
                let (color_sum, n_colors) = chunk_iter
                    .filter_map(|pos| match self.get_voxel_local(pos) {
                        None => {
                            logger::log!(Error, from = "chunk", "failed to get voxel by pos {pos}");
                            None
                        },
                        some => some,
                    })
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

        matches!(
            self.info.load(Relaxed).fill_type,
            FillType::AllSame(id) if id == AIR_VOXEL_DATA.id
        )
    }

    /// Gives `Some()` with fill id or returns `None`.
    pub fn fill_id(&self) -> Option<Id> {
        match self.info.load(Relaxed).fill_type {
            FillType::AllSame(id) => Some(id),
            _ => None,
        }
    }

    /// Checks if chunk is filled with same voxel id.
    pub fn is_same_filled(&self) -> bool {
        self.fill_id().is_some()
    }

    /// Checks if chunk is filled with non-air voxels.
    pub fn is_filled(&self) -> bool {
        self.info.load(Relaxed).is_filled
    }

    /// Gives [`Vec`] with full detail vertices mesh of [`Chunk`].
    pub fn make_full_mesh(&self, chunk_adj: ChunkAdj) -> Mesh<FullVertex> {
        let is_filled_and_blocked = self.is_filled() && Self::is_adj_filled(&chunk_adj);
        if self.is_empty() || is_filled_and_blocked { return default() }

        let info = self.info.load(Relaxed);
        let pos_iter: Box<dyn Iterator<Item = Int3>> = match info.fill_type {
            FillType::Unspecified =>
                Box::new(Chunk::local_pos_iter()),

            FillType::AllSame(id) => if id == AIR_VOXEL_DATA.id {
                Box::new(std::iter::empty())
            } else {
                Box::new(CubeBoundary::new(Chunk::SIZE as i32))
            },
        };

        let vertices = pos_iter
            .filter_map(|pos| match self.get_voxel_local(pos) {
                None => {
                    logger::log!(Error, from = "chunk", "failed to get voxel from pos {pos}");
                    None
                },
                some => some,
            })
            .filter(|voxel| !voxel.is_air())
            .flat_map(|voxel| {
                let side_iter = SpaceIter::adj_iter(Int3::ZERO)
                    .filter(|&offset| {
                        let adj = chunk_adj.by_offset(offset);
                        match self.get_voxel_global(voxel.pos + offset) {
                            ChunkOption::Voxel(voxel) => voxel.is_air(),

                            ChunkOption::OutsideChunk => match adj {
                                None => true,

                                Some(chunk) => match chunk.get_voxel_global(voxel.pos + offset) {
                                    ChunkOption::Voxel(voxel) => voxel.is_air(),
                                    ChunkOption::OutsideChunk => true,
                                    ChunkOption::Failed => {
                                        logger::log!(
                                            Error, from = "chunk",
                                            "caught on failed chunk voxel in {pos}",
                                            pos = voxel.pos + offset,
                                        );
                                        true
                                    },
                                }
                            },

                            ChunkOption::Failed => {
                                logger::log!(
                                    Error, from = "chunk",
                                    "caught on failed chunk voxel in {pos}",
                                    pos = voxel.pos + offset,
                                );
                                true
                            },
                        }
                    });

                const N_CUBE_VERTICES: usize = 36;
                let mut vertices = SmallVec::<[_; N_CUBE_VERTICES]>::new();

                let mesh_builder = CubeDetailed::new(voxel.data);
                for offset in side_iter {
                    mesh_builder.by_offset(offset, voxel.pos.into(), &mut vertices);
                }

                vertices
            })
            .collect();

        Mesh::new(vertices, None, default())
    }

    fn optimize_chunk_adj_for_partitioning(mut chunk_adj: ChunkAdj, partition_coord: USize3) -> ChunkAdj {
        chunk_adj.set(
            veci!(1 - partition_coord.x as i32 * 2, 0, 0),
            None,
        ).expect("failed to set side");

        chunk_adj.set(
            veci!(0, 1 - partition_coord.y as i32 * 2, 0),
            None,
        ).expect("failed to set side");

        chunk_adj.set(
            veci!(0, 0, 1 - partition_coord.z as i32 * 2),
            None,
        ).expect("failed to set side");

        chunk_adj
    }

    pub fn make_partition(&self, chunk_adj: &ChunkAdj, partition_idx: usize) -> Mesh<FullVertex> {
        let coord_idx = iterator::idx_to_coord_idx(partition_idx, USize3::all(2));
        let chunk_adj = Self::optimize_chunk_adj_for_partitioning(chunk_adj.clone(), coord_idx);

        let start_pos = Int3::from(coord_idx * Chunk::SIZES / 2);
        let end_pos   = start_pos + Int3::from(Chunk::SIZES / 2);

        let vertices = SpaceIter::new(start_pos..end_pos)
            .filter_map(|pos| match self.get_voxel_local(pos) {
                some @ Some(_) => some,
                None => {
                    logger::log!(Error, from = "chunk", "failed to get voxel from pos {pos}");
                    None
                },
            })
            .filter(|voxel| !voxel.is_air())
            .flat_map(|voxel| {
                let offset_iter = SpaceIter::adj_iter(Int3::ZERO)
                    .filter(|&offset| {
                        let adj = chunk_adj.by_offset(offset);
                        match self.get_voxel_global(voxel.pos + offset) {
                            ChunkOption::Voxel(voxel) => voxel.is_air(),

                            ChunkOption::OutsideChunk => match adj {
                                None => true,

                                Some(chunk) => match chunk.get_voxel_global(voxel.pos + offset) {
                                    ChunkOption::Voxel(voxel) => voxel.is_air(),
                                    ChunkOption::OutsideChunk => true,
                                    ChunkOption::Failed => {
                                        logger::log!(
                                            Error, from = "chunk",
                                            "caught on failed chunk voxel in {pos}",
                                            pos = voxel.pos + offset,
                                        );
                                        true
                                    },
                                }
                            },

                            ChunkOption::Failed => {
                                logger::log!(
                                    Error, from = "chunk",
                                    "caught on failed chunk voxel in {pos}",
                                    pos = voxel.pos + offset,
                                );
                                true
                            },
                        }
                    });

                const N_CUBE_VERTICES: usize = 36;
                let mut vertices = SmallVec::<[_; N_CUBE_VERTICES]>::new();

                let mesh_builder = CubeDetailed::new(voxel.data);
                for offset in offset_iter {
                    mesh_builder.by_offset(offset, voxel.pos.into(), &mut vertices);
                }

                vertices
            })
            .collect();

        Mesh::new(vertices, None, default())
    }

    pub fn is_adj_filled(adj: &ChunkAdj) -> bool {
        adj.inner.iter().all(|chunk| matches!(chunk, Some(chunk) if chunk.is_filled()))
    }

    /// Gives [`Vec`] with full detail vertices mesh of [`Chunk`].
    pub fn make_partitial_meshes(&self, chunk_adj: ChunkAdj) -> [Mesh<FullVertex>; 8] {
        let is_filled_and_blocked = self.is_filled() && Self::is_adj_filled(&chunk_adj);
        if self.is_empty() || is_filled_and_blocked {
            return default();
        }

        array_init(|idx| self.make_partition(&chunk_adj, idx))
    }

    /// Makes vertices for *low detail* mesh from voxel array.
    pub fn make_low_mesh(&self, chunk_adj: ChunkAdj, lod: Lod) -> Mesh<LowVertex> {
        assert!(lod > 0, "There's a separate function for LOD = 0! Use .make_vertices_detailed() instead!");
        
        let is_filled_and_blocked = self.is_filled() && Self::is_adj_filled(&chunk_adj);
        if self.is_empty() || is_filled_and_blocked { return default() }

        // TODO: optimize for same-filled chunks
        let sub_chunk_size = 2_i32.pow(lod);
        let vertices = self.low_voxel_iter(lod)
            .filter_map(|(voxel, p)| match voxel {
                LoweredVoxel::Transparent => None,
                LoweredVoxel::Colored(color) => Some((color, p)),
            })
            .flat_map(|(voxel_color, local_low_pos)| {
                let local_pos = local_low_pos * sub_chunk_size;
                let global_pos = Chunk::local_to_global_pos(self.pos.load(Relaxed), local_pos);

                let center_pos = (vec3::from(global_pos)
                          + 0.5 * vec3::all(sub_chunk_size as f32)) * Voxel::SIZE
                          - 0.5 * vec3::all(Voxel::SIZE);
                         
                let is_blocking_voxel = |pos: Int3, offset: Int3| match self.get_voxel_global(pos) {
                    ChunkOption::OutsideChunk => {
                        match chunk_adj.by_offset(offset) {
                            /* There is no chunk so voxel isn't blocked */
                            None => false,
                            
                            Some(chunk) => match chunk.get_voxel_global(pos) {
                                ChunkOption::OutsideChunk => unreachable!("Can't fall out of an adjacent chunk"),
                                ChunkOption::Voxel(voxel) => !voxel.is_air(),
                                ChunkOption::Failed => {
                                    logger::log!(Error, from = "chunk", "caught failed chunk voxel in {pos}");
                                    false
                                },
                            },
                        }
                    },

                    ChunkOption::Voxel(voxel) => !voxel.is_air(),

                    ChunkOption::Failed => {
                        logger::log!(Error, from = "chunk", "caught failed chunk voxel in {pos}");
                        false
                    },
                };

                let is_blocked_subchunk = |offset: Int3| -> bool {
                    let start_pos = global_pos + offset * sub_chunk_size;
                    let end_pos   = global_pos + (offset + Int3::ONE) * sub_chunk_size;

                    let is_on_surface = match offset.as_tuple() {
                        (-1, 0, 0) if 0 == local_pos.x => true,
                        (0, -1, 0) if 0 == local_pos.y => true,
                        (0, 0, -1) if 0 == local_pos.z => true,
                        (1, 0, 0) if Chunk::SIZE as i32 == local_pos.x + sub_chunk_size => true,
                        (0, 1, 0) if Chunk::SIZE as i32 == local_pos.y + sub_chunk_size => true,
                        (0, 0, 1) if Chunk::SIZE as i32 == local_pos.z + sub_chunk_size => true,
                        _ => false,
                    };
                    
                    let mut iter = SpaceIter::new(start_pos..end_pos);
                    let pred = |pos| is_blocking_voxel(pos, offset);

                    match chunk_adj.by_offset(offset) {
                        Some(_) if is_on_surface =>
                            iter.all(pred),
                        _ =>
                            iter.any(pred),
                    }
                };

                let mesh_builder = CubeLowered::new(
                    sub_chunk_size as f32 * Voxel::SIZE
                );
                
                const N_CUBE_VERTICES: usize = 36;
                let mut vertices = Vec::with_capacity(N_CUBE_VERTICES);

                for offset in SpaceIter::adj_iter(Int3::ZERO).filter(|&o| !is_blocked_subchunk(o)) {
                    mesh_builder.by_offset(offset, center_pos, voxel_color, &mut vertices);
                }

                vertices
            })
            .collect();

        Mesh::new(vertices, None, default())
    }

    /// Gives [voxel id][Id] by it's index in array.
    /// Returns [`Some`] with [id][Id] or [`None`] if `idx` is invalid.
    pub fn get_id(&self, idx: usize) -> Option<Id> {
        if Chunk::VOLUME <= idx { return None }

        Some(match self.info.load(Relaxed).fill_type {
            FillType::AllSame(id) => id,
            FillType::Unspecified => self.voxel_ids[idx].load(Relaxed)
        })
    }

    /// Givex voxel from global position.
    pub fn get_voxel_global(&self, global_pos: Int3) -> ChunkOption<Voxel> {
        let local_pos = Chunk::global_to_local_pos(self.pos.load(Relaxed), global_pos);

        if local_pos.x < 0 || local_pos.x >= Chunk::SIZE as i32 ||
           local_pos.y < 0 || local_pos.y >= Chunk::SIZE as i32 ||
           local_pos.z < 0 || local_pos.z >= Chunk::SIZE as i32
        { return ChunkOption::OutsideChunk }

        let voxel = match self.get_voxel_local(local_pos) {
            Some(voxel) => voxel,
            None => return ChunkOption::Failed,
        };
        
        ChunkOption::Voxel(voxel)
    }

    /// Gives voxel from local position (relative to chunk).
    /// 
    /// # Panic
    /// 
    /// Panics if [chunk][Chunk] is not already had been generated or `local_pos` is not local.
    pub fn get_voxel_local(&self, local_pos: Int3) -> Option<Voxel> {
        if !self.is_generated() { return None }
        
        let idx = Chunk::voxel_pos_to_idx(local_pos)?;

        let id = self.get_id(idx)
            .expect("local_pos is local");

        let global_pos = Chunk::local_to_global_pos(self.pos.load(Relaxed), local_pos);
        Some(Voxel::new(global_pos, &VOXEL_DATA[id as usize]))
    }

    /// Tests that chunk is visible by camera.
    pub fn is_visible_by_camera(&self, camera: &mut Camera) -> bool {
        let global_chunk_pos = Chunk::global_pos(self.pos.load(Relaxed));
        let global_chunk_pos = vec3::from(global_chunk_pos) * Voxel::SIZE;

        let lo = global_chunk_pos - 0.5 * vec3::all(Voxel::SIZE);
        let hi = lo + vec3::all(Chunk::GLOBAL_SIZE) - 0.5 * vec3::all(Voxel::SIZE);

        camera.is_aabb_in_view(Aabb::from_float3(lo, hi))
    }

    /// Checks if [`Chunk`] is not already generated.
    pub fn is_generated(&self) -> bool {
        !self.voxel_ids.is_empty()
    }

    /// Generates voxel id array.
    pub fn generate_voxels(chunk_pos: Int3, chunk_array_sizes: USize3) -> Vec<Atomic<Id>> {
        let mut result = Vec::with_capacity(Self::VOLUME);

        for pos in Self::global_pos_iter(chunk_pos) {
            let height = gen::perlin(pos, chunk_array_sizes);
            let id = if pos.y <= height - 5 {
                STONE_VOXEL_DATA.id
            } else if pos.y < height {
                DIRT_VOXEL_DATA.id
            } else if pos.y <= height {
                GRASS_VOXEL_DATA.id
            } else {
                AIR_VOXEL_DATA.id
            };

            result.push(Atomic::new(id));
        }

        result
    }

    /// Generates a chunk.
    pub fn new(chunk_pos: Int3, chunk_array_sizes: USize3) -> Self {
        Self::from_voxels(Self::generate_voxels(chunk_pos, chunk_array_sizes), chunk_pos)
    }

    /// Constructs empty chunk.
    pub fn new_empty(chunk_pos: Int3) -> Self {
        Self::from_voxels(vec![], chunk_pos)
    }

    pub fn new_same_filled(chunk_pos: Int3, fill_id: Id) -> Self {
        Self {
            voxel_ids: vec![Atomic::new(fill_id)],
            info: Atomic::new(Info {
                fill_type: FillType::AllSame(fill_id),
                is_filled: true,
                active_lod: None,
            }),
            ..Self::new_empty(chunk_pos)
        }
    }

    /// Makes a [chunk][Chunk] out of voxel_ids.
    /// 
    /// # Panic
    /// 
    /// Panics if `voxel_ids.len()` is not equal to `Chunk::VOLUME` or `0`.
    pub fn from_voxels(voxel_ids: Vec<Atomic<Id>>, chunk_pos: Int3) -> Self {
        let len = voxel_ids.len();
        assert!(
            matches!(len, Self::VOLUME | 0),
            "`voxel_ids.len()` should be equal to `Chunk::VOLUME` or `0`, but it's {len}",
        );

        Self {
            pos: Atomic::new(chunk_pos),
            voxel_ids,
            info: default(),
        }.as_optimized()
    }

    /// Sets [voxel id][Id] to `new_id` by it's index in array.
    /// Note that it does not drop all meshes that can possibly hold old id.
    /// And note that it may unoptimize chunk even if it can be.
    /// Returns old [id][Id].
    /// 
    /// # Error
    /// 
    /// Returns [`Err`] if `idx` is out of bounds.
    pub fn set_id(&mut self, idx: usize, new_id: Id) -> Result<Id, EditError> {
        if Self::VOLUME <= idx {
            return Err(
                EditError::IdxOutOfBounds { idx, len: Self::VOLUME }
            )
        }

        let old_id = match self.info.load(Relaxed).fill_type {
            FillType::Unspecified => {
                let old_id = self.voxel_ids[idx].swap(new_id, AcqRel);
                if old_id != new_id { self.optimize() }
                old_id
            },

            FillType::AllSame(old_id) => if old_id != new_id {
                self.unoptimyze();
                self.voxel_ids[idx].swap(new_id, AcqRel)
            } else {
                old_id
            },
        };

        Ok(old_id)
    }

    /// Sets new voxel [`id`][Id] to voxel by index.
    /// 
    /// # Safety
    /// 
    /// 1. `idx` < `Chunk::VOLUME`
    /// 2. `self.info.fill_type` should be [`FillType::Default`].
    pub unsafe fn set_id_fast(&self, idx: usize, new_id: Id) -> Id {
        self.voxel_ids.get_unchecked(idx).swap(new_id, AcqRel)
    }

    /// Sets voxel's id with position `pos` to `new_id` and returns old [id][Id]. If voxel is 
    /// set then this function should drop all its meshes.
    /// 
    /// # Error
    /// 
    /// Returns `Err` if `new_id` is not valid or `pos` is not in this [`Chunk`].
    pub fn set_voxel(&mut self, pos: Int3, new_id: Id) -> Result<Id, EditError> {
        if !voxel::is_id_valid(new_id) {
            return Err(EditError::InvalidId(new_id));
        }

        let local_pos = Self::global_to_local_pos_checked(self.pos.load(Relaxed), pos)?;
        let idx = Self::voxel_pos_to_idx_unchecked(local_pos);

        // We know that idx is valid so we can get-by-index.
        let old_id = self.get_id(idx).expect("idx should be valid");
        if old_id != new_id {
            self.set_id(idx, new_id)?;
            self.optimize();
        }

        Ok(old_id)
    }

    /// Sets voxel's ids in range `pos_from..pos_to` to index [`new_id`][Id].
    pub fn fill_voxels(&mut self, pos_from: Int3, pos_to: Int3, new_id: Id) -> Result<bool, EditError> {
        if !voxel::is_id_valid(new_id) {
            return Err(EditError::InvalidId(new_id));
        }

        let pos = self.pos.load(Relaxed);
        let local_pos_from = Self::global_to_local_pos_checked(pos, pos_from)?;

        Self::global_to_local_pos_checked(pos, pos_to - Int3::ONE)?;
        let local_pos_to = Self::global_to_local_pos(pos, pos_to);

        self.unoptimyze();

        let mut is_changed = false;

        for local_pos in SpaceIter::new(local_pos_from..local_pos_to) {
            // We can safely not to check idx due to previous check.
            let idx = Self::voxel_pos_to_idx_unchecked(local_pos);
            
            let old_id = self.get_id(idx).expect("idx should be valid");
            if old_id != new_id {
                is_changed = true;

                // * Safety:
                // * Safe, because `idx` is valid and `self` is unoptimized.
                unsafe {
                    self.set_id_fast(idx, new_id);
                }
            }
        }

        self.optimize();

        Ok(is_changed)
    }

    /// Gives iterator over all id-vectors in chunk (or relative to chunk voxel positions).
    pub fn local_pos_iter() -> SpaceIter {
        SpaceIter::new(Int3::ZERO..Self::SIZES.into())
    }

    /// Gives iterator over all id-vectors in chunk (or relative to chunk voxel positions).
    pub fn global_pos_iter(chunk_pos: Int3) -> impl ExactSizeIterator<Item = Int3> {
        Self::local_pos_iter()
            .map(move |pos| Self::local_to_global_pos(chunk_pos, pos))
    }

    /// Gives iterator that yields iterator over some chunk of voxels.
    pub fn chunked_pos_iter(sub_chunk_size: usize) -> impl ExactSizeIterator<Item = SpaceIter> {
        SpaceIter::split_chunks(
            Self::SIZES.into(),
            Int3::all(sub_chunk_size as i32),
        )
    }

    /// Applies storage optimizations to voxel array.
    pub fn as_optimized(mut self) -> Self {
        self.optimize();
        self
    }

    /// Applies storage optimizations to [voxel array][Chunk].
    pub fn optimize(&mut self) {
        self.unoptimyze();

        if !self.is_generated() { return }
        
        let mut info = Info {
            active_lod: self.info.load(Acquire).active_lod,
            ..default()
        };

        /* All-same pass */
        let is_all_same = self.voxel_ids.iter()
            .map(|id| id.load(Relaxed))
            .all_equal();
        if is_all_same {
            let all = self.voxel_ids[0].load(Relaxed);
            self.voxel_ids = vec![Atomic::new(all)];
            info.fill_type = FillType::AllSame(all);
        }

        let is_all_not_air = self.voxel_ids.iter()
            .all(|voxel_id| voxel_id.load(Relaxed) != AIR_VOXEL_DATA.id);
        info.is_filled = is_all_not_air;

        self.info.store(info, Release);
    }

    /// Disapplies storage optimizations.
    pub fn unoptimyze(&mut self) {
        let mut info = self.info.load(Acquire);

        match info.fill_type {
            FillType::Unspecified => (),
            FillType::AllSame(id) =>
                self.voxel_ids = std::iter::from_fn(|| Some(Atomic::new(id)))
                    .take(Self::VOLUME)
                    .collect(),
        }

        info.fill_type = FillType::Unspecified;

        self.info.store(info, Release);
    }

    /// Converts chunk position to world position.
    pub fn global_pos(chunk_pos: Int3) -> Int3 {
        chunk_pos * Self::SIZE as i32
    }

    /// Converts all in-chunk world positions to that chunk position.
    pub fn local_pos(world_pos: Int3) -> Int3 {
        world_pos.div_euclid(Int3::from(Self::SIZES))
    }

    /// Computes global position from relative to chunk position.
    pub fn local_to_global_pos(chunk_pos: Int3, relative_voxel_pos: Int3) -> Int3 {
        Self::global_pos(chunk_pos) + relative_voxel_pos
    }

    /// Computes local (relative to chunk) position from global position.
    pub fn global_to_local_pos(chunk_pos: Int3, global_voxel_pos: Int3) -> Int3 {
        global_voxel_pos - Self::global_pos(chunk_pos)
    }

    /// Computes local (relative to chunk) position from global position.
    /// 
    /// # Error
    /// 
    /// Returns [`Err`] if local position is out of [chunk][Chunk] bounds.
    pub fn global_to_local_pos_checked(chunk_pos: Int3, global_voxel_pos: Int3) -> Result<Int3, EditError> {
        let local_pos = Self::global_to_local_pos(chunk_pos, global_voxel_pos);

        let is_out_of_bounds =
            local_pos.x < 0 || Self::SIZES.x as i32 <= local_pos.x ||
            local_pos.y < 0 || Self::SIZES.y as i32 <= local_pos.y ||
            local_pos.z < 0 || Self::SIZES.z as i32 <= local_pos.z;

        is_out_of_bounds
            .then_some(local_pos)
            .ok_or(EditError::PosOutOfBounds { pos: global_voxel_pos, chunk_pos })
    }

    /// Gives index in voxel array by it's 3D-index (or relative to chunk position)
    /// 
    /// # Error
    /// 
    /// Returns [`None`] if `pos` < [`Int3::ZERO`][Int3] or `pos` >= [`Chunk::SIZES`][Chunk].
    pub fn voxel_pos_to_idx(pos: Int3) -> Option<usize> {
        if pos.x < 0 || pos.y < 0 || pos.z < 0 {
            return None;
        }

        let idx = sdex::get_index(&USize3::from(pos).as_array(), &[Self::SIZE; 3]);

        (idx < Self::VOLUME).then_some(idx)
    }

    /// Gives index in voxel array by it's 3D-index (or relative to chunk position)
    /// without idx-safe ckeck.
    pub fn voxel_pos_to_idx_unchecked(pos: Int3) -> usize {
        sdex::get_index(&USize3::from(pos).as_array(), &[Self::SIZE; 3])
    }

    /// Generates and sets [mesh][Mesh] to [chunk][Chunk].
    pub fn generate_mesh(&self, chunk_mesh: &mut ChunkMesh, lod: Lod, chunk_adj: ChunkAdj, device: &Device) {
        match lod {
            0 => {
                let mesh = self.make_full_mesh(chunk_adj);
                chunk_mesh.upload_full_mesh(device, &mesh);
            },
            
            _ => {
                let mesh = self.make_low_mesh(chunk_adj, lod);
                chunk_mesh.upload_low_mesh(device, &mesh, lod);
            }
        }
    }

    /// Partitions [mesh][crate::graphics::mesh::Mesh] of this [chunk][Chunk].
    pub fn partition_mesh(&self, mesh: &mut ChunkMesh, chunk_adj: ChunkAdj, device: &Device) {
        let meshes = self.make_partitial_meshes(chunk_adj);
        mesh.upload_partial_meshes(device, &meshes);
    }

    /// Renders a [`Chunk`].
    pub fn render<'rp>(
        &self, mesh: &'rp mut ChunkMesh, lod: Lod,
        pipeline: &'rp RenderPipeline, pass: &mut RenderPass<'rp>,
    ) -> Result<(), ChunkRenderError> {
        if self.is_empty() { return Ok(()) }

        mesh.active_lod = Some(lod);
        mesh.render(pipeline, pass)
    }

    /// Sets active LOD to given value.
    /// 
    /// # Panic
    /// 
    /// Panics if `lod` is not available.
    pub fn set_active_lod(&self, mesh: &ChunkMesh, lod: Lod) {
        self.try_set_active_lod(mesh, lod)
            .expect("new LOD value should be available")
    }

    /// Tries to set active LOD to given value.
    pub fn try_set_active_lod(&self, mesh: &ChunkMesh, lod: Lod) -> Result<(), SetLodError> {
        let mut info = self.info.load(Acquire);

        mesh.get_available_lods()
            .contains(&lod)
            .then(|| info.active_lod = Some(lod))
            .ok_or(SetLodError::SetActiveLod { tried: lod, active: info.active_lod })?;

        self.info.store(info, Release);
        Ok(())
    }

    /// Tries to set LOD value that have least difference with given value.
    /// If there is at least one LOD it will return `Some(..)` with that value, otherwise, `None`.
    pub fn try_set_best_fit_lod(&self, mesh: &ChunkMesh, lod: Lod) -> Option<Lod> {
        let best_fit = mesh.get_available_lods()
            .into_iter()
            .min_by_key(|elem| elem.abs_diff(lod))?;

        self.set_active_lod(mesh, best_fit);

        Some(best_fit)
    }

    /// Gives list of all possible LODs.
    pub fn get_possible_lods() -> [Lod; Self::N_LODS] {
        array_init(|i| i as Lod)
    }

    pub fn can_render_active_lod(&self, mesh: &ChunkMesh) -> bool {
        matches!(
            self.info.load(Relaxed).active_lod,
            Some(lod) if mesh.get_available_lods().contains(&lod)
        )
    }
}



#[derive(Error, Debug)]
pub enum SetLodError {
    #[error("failed to set LOD value to {tried} because there's no mesh for it. Active LOD value is {active:?}")]
    SetActiveLod {
        tried: Lod,
        active: Option<Lod>,
    },
}



#[derive(Error, Debug, Clone)]
pub enum ChunkRenderError {
    #[error("Expected a mesh with LOD value {0}")]
    NoMesh(Lod),

    #[error("Unexpectedly large LOD value: {0}")]
    TooBigLod(Lod),
}



#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Info {
    pub fill_type: FillType,
    pub is_filled: bool,
    pub active_lod: Option<Lod>,
}



#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum FillType {
    #[default]
    Unspecified,
    AllSame(Id),
}

impl AsBytes for FillType {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Unspecified => vec![0],
            Self::AllSame(id) => compose! {
                std::iter::once(1),
                id.as_bytes(),
            }.collect()
        }
    }
}

impl FromBytes for FillType {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        let mut reader = ByteReader::new(source);
        let variant: u8 = reader.read()?;

        match variant {
            0 => Ok(Self::Unspecified),
            1 => Ok(Self::AllSame(reader.read()?)),
            _ => Err(ReinterpretError::Conversion(
                format!("conversion of too large byte ({variant}) to FillType")
            ))
        }
    }
}

impl DynamicSize for FillType {
    fn dynamic_size(&self) -> usize {
        u8::static_size() +
        match self {
            Self::Unspecified => 0,
            Self::AllSame(_) => Id::static_size(),
        }
    }
}



#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ChunkOption<T> {
    OutsideChunk,
    Voxel(T),
    Failed,
}

pub type Lod = u32;

#[derive(Debug, Error)]
pub enum EditError {
    #[error("failed to convert voxel position to array index {0}")]
    PosIdConversion(Int3),

    #[error("position is out of chunk bounds")]
    PosOutOfBounds {
        pos: Int3,
        chunk_pos: Int3,
    },

    #[error("index out of bounds: index is {idx} but len is {len}")]
    IdxOutOfBounds {
        idx: usize,
        len: usize,
    },

    #[error("invalid id {0}")]
    InvalidId(Id),
}