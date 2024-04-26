pub mod tasks;
pub mod commands;
pub mod mesh;
pub mod array;

use {
    crate::{
        prelude::*,
        geometry::{Frustum, Aabb},
    },
    super::voxel::{
        self,
        Voxel,
        VoxelColor,
        shape::CubeDetailed,
        voxel_data::{data::*, VoxelId},
        // TODO: remove: generator as gen,
    },
    mesh::HiResVertex,
    array::ChunkAdj,
};



pub mod prelude {
    pub use super::{
        Chunk, SetLodError, ChunkRenderError, ChunkInfo, Lod,
        ChunkOption, FillType,
    };
}



#[derive(Debug)]
pub struct Chunk {
    pub pos: Atomic<IVec3>,
    // TODO: try use `Arc<[Atomic<Id>]> instead.
    pub voxel_ids: Vec<Atomic<VoxelId>>,
    pub info: Atomic<ChunkInfoPacked>,
}
assert_impl_all!(Chunk: Send, Sync, Component);

impl ConstDefault for Chunk {
    #[allow(clippy::declare_interior_mutable_const)]
    const DEFAULT: Self = Self {
        voxel_ids: const_default(),
        pos: const_default(),
        info: const_default(),
    };
}

impl Default for Chunk {
    fn default() -> Self { const_default() }
}

impl Chunk {
    /// [Chunk] size in voxels.
    pub const SIZE: usize = cfg::terrain::CHUNK_SIZE;

    /// [Chunk] sizes in voxels.
    pub const SIZES: U16Vec3 = U16Vec3::splat(Self::SIZE as u16);

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
    pub fn low_voxel_iter(&self, lod: Lod) -> impl ExactSizeIterator<Item = (VoxelColor, IVec3)> + '_ {
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
                    .fold((Vec3::ZERO, 0_usize), |(col_acc, n_acc), col|
                        (col_acc + col, n_acc + 1)
                    );

                match n_colors {
                    0 => VoxelColor::Transparent,
                    n => VoxelColor::Colored(color_sum / n as f32),
                }
            })
            .zip(Range3d::zeroed_cubed(Chunk::SIZE as i32 / sub_chunk_size))
    }

    /// Checks if chunk is empty.
    pub fn is_empty(&self) -> bool {
        self.voxel_ids.is_empty() || matches!(
            self.info.load(Relaxed).get_fill_type(),
            FillType::AllSame(id) if id == AIR_VOXEL_DATA.id
        )
    }

    /// Gives `Some()` with fill id or returns `None`.
    pub fn fill_id(&self) -> Option<VoxelId> {
        match self.info.load(Relaxed).get_fill_type() {
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
        self.info.load(Relaxed).is_filled()
    }

    #[allow(unused)]
    fn optimize_chunk_adj_for_partitioning(mut chunk_adj: ChunkAdj, partition_coord: U16Vec3) -> ChunkAdj {
        chunk_adj.set(
            IVec3::new(1 - partition_coord.x as i32 * 2, 0, 0),
            None,
        ).expect("failed to set side");

        chunk_adj.set(
            IVec3::new(0, 1 - partition_coord.y as i32 * 2, 0),
            None,
        ).expect("failed to set side");

        chunk_adj.set(
            IVec3::new(0, 0, 1 - partition_coord.z as i32 * 2),
            None,
        ).expect("failed to set side");

        chunk_adj
    }

    pub fn make_partition_vertices(&self, voxel: Voxel, adj: &ChunkAdj) -> SmallVec<[HiResVertex; 36]> {
        let offset_iter = Range3d::adj_iter(IVec3::ZERO)
            .filter(|&offset| {
                let adj = adj.by_offset(offset);
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

        let mut vertices = smallvec![];

        let mesh_builder = CubeDetailed::new(voxel.data);
        for offset in offset_iter {
            mesh_builder.by_offset(offset, voxel.pos.as_vec3(), &mut vertices);
        }

        vertices
    }

    /// Checks that adjacent chunks are filled.
    pub fn is_adj_filled(adj: &ChunkAdj) -> bool {
        adj.inner.iter().all(|chunk|
            matches!(chunk, Some(chunk) if chunk.is_filled())
        )
    }

    // FIXME: move to mesh code
    // Gives [`Vec`] with full detail vertices mesh of [`Chunk`].
    // pub fn make_partitial_meshes(&self, chunk_adj: ChunkAdj) -> [Mesh<HiResVertex>; 8] {
    //     let is_filled_and_blocked = self.is_filled() && Self::is_adj_filled(&chunk_adj);
    //     ensure_or!(!self.is_empty() && !is_filled_and_blocked, return default());

    //     array_init(|idx| self.make_partition(&chunk_adj, idx))
    // }

    /// Gives [voxel id][Id] by it's index in array.
    /// Returns [`Some`] with [id][Id] or [`None`] if `idx` is invalid.
    pub fn get_id(&self, idx: usize) -> Option<VoxelId> {
        ensure_or!(idx < Chunk::VOLUME, return None);

        Some(match self.info.load(Relaxed).get_fill_type() {
            FillType::AllSame(id) => id,
            FillType::Unspecified => self.voxel_ids[idx].load(Relaxed)
        })
    }

    /// Givex voxel from global position.
    pub fn get_voxel_global(&self, global_pos: IVec3) -> ChunkOption<Voxel> {
        let local_pos = Chunk::global_to_local_pos(self.pos.load(Relaxed), global_pos);

        if local_pos.x < 0 || local_pos.x >= Chunk::SIZE as i32 ||
           local_pos.y < 0 || local_pos.y >= Chunk::SIZE as i32 ||
           local_pos.z < 0 || local_pos.z >= Chunk::SIZE as i32
        { return ChunkOption::OutsideChunk }

        let Some(voxel) = self.get_voxel_local(local_pos) else {
            return ChunkOption::Failed
        };
        
        ChunkOption::Voxel(voxel)
    }

    /// Gives voxel from local position (relative to chunk).
    /// 
    /// # Panic
    /// 
    /// Panics if [chunk][Chunk] is not already had been generated or `local_pos` is not local.
    pub fn get_voxel_local(&self, local_pos: IVec3) -> Option<Voxel> {
        ensure_or!(self.is_generated(), return None);
        
        let idx = Chunk::voxel_pos_to_idx(local_pos)?;

        let id = self.get_id(idx)
            .expect("local_pos is local");

        let global_pos = Chunk::local_to_global_pos(self.pos.load(Relaxed), local_pos);
        Some(Voxel::new(global_pos, &VOXEL_DATA[id as usize]))
    }

    /// Tests that chunk is visible by camera.
    pub fn is_in_frustum(&self, frustum: &Frustum) -> bool {
        let global_chunk_pos = Chunk::local_to_global(self.pos.load(Relaxed));
        let global_chunk_pos = global_chunk_pos.as_vec3() * Voxel::SIZE;

        let lo = global_chunk_pos - 0.5 * Vec3::splat(Voxel::SIZE);
        let hi = lo + Vec3::splat(Chunk::GLOBAL_SIZE);

        frustum.intersects(&Aabb::new(lo, hi))
    }

    /// Checks if [`Chunk`] is not already generated.
    pub fn is_generated(&self) -> bool {
        !self.voxel_ids.is_empty()
    }

    /// Transmutes [`Vec<Id>`] into [`Vec<Atomic<Id>>`].
    pub const fn transmute_ids_to_atomic(src: Vec<VoxelId>) -> Vec<Atomic<VoxelId>> {
        // * Safety
        // * 
        // * Safe, because memory layout of `T` is same as `Atomic<T>`,
        // * because `Atomic<T>` is `repr(transparent)` of `UnsafeCell<T>`
        // * and `UnsafeCell<T>` is `repr(transparent)` of `T`.
        unsafe { mem::transmute(src) }
    }

    /// Transmutes [`Vec<Atomic<Id>>`] into [`Vec<Id>`].
    pub const fn transmute_ids_from_atomic(src: Vec<Atomic<VoxelId>>) -> Vec<VoxelId> {
        // * Safety
        // * 
        // * Safe, because memory layout of `T` is same as `Atomic<T>`,
        // * because `Atomic<T>` is `repr(transparent)` of `UnsafeCell<T>`
        // * and `UnsafeCell<T>` is `repr(transparent)` of `T`.
        unsafe { mem::transmute(src) }
    }

    /// Generates voxel id array.
    pub fn generate_voxels(chunk_pos: IVec3, _chunk_array_sizes: U16Vec3) -> Vec<VoxelId> {
        let mut result = Vec::with_capacity(Self::VOLUME);

        for pos in Self::global_pos_iter(chunk_pos) {
            let height = 10 - rand::random::<i32>() % 2; // gen::perlin(pos, chunk_array_sizes);

            let id = if pos.y <= height - 5 {
                STONE_VOXEL_DATA.id
            } else if pos.y < height {
                DIRT_VOXEL_DATA.id
            } else if pos.y <= height {
                GRASS_VOXEL_DATA.id
            } else {
                AIR_VOXEL_DATA.id
            };

            result.push(id);
        }

        result
    }

    /// Generates a chunk.
    pub fn new(chunk_pos: IVec3, chunk_array_sizes: U16Vec3) -> Self {
        Self::from_voxels(Self::generate_voxels(chunk_pos, chunk_array_sizes), chunk_pos)
    }

    /// Constructs empty chunk.
    pub fn new_empty(chunk_pos: IVec3) -> Self {
        Self::from_voxels(vec![], chunk_pos)
    }

    /// Constructs new [chunk][Chunk] filled with the same voxel
    pub fn new_same_filled(chunk_pos: IVec3, fill_id: VoxelId) -> Self {
        Self {
            voxel_ids: vec![Atomic::new(fill_id)],
            info: const_default(),
            ..Self::new_empty(chunk_pos)
        }
    }

    /// Makes a [chunk][Chunk] out of voxel_ids.
    /// 
    /// # Panic
    /// 
    /// Panics if `voxel_ids.len()` is not equal to `Chunk::VOLUME` or `0`.
    pub fn from_voxels(voxel_ids: Vec<VoxelId>, chunk_pos: IVec3) -> Self {
        Self::from_voxels_and_fill(voxel_ids, FillType::Unspecified, chunk_pos)
    }

    /// Makes a [chunk][Chunk] out of voxel_ids.
    /// 
    /// # Panic
    /// 
    /// Panics if `voxel_ids.len()` is not matches the [`fill_type`][FillType].
    pub fn from_voxels_and_fill(voxel_ids: Vec<VoxelId>, fill_type: FillType, chunk_pos: IVec3) -> Self {
        let len = voxel_ids.len();

        match fill_type {
            FillType::Unspecified => {
                assert!(
                    matches!(len, Self::VOLUME | 0),
                    "`voxel_ids.len()` should be equal to `Chunk::VOLUME` or `0`, but it's {len}",
                );

                Self {
                    pos: Atomic::new(chunk_pos),
                    voxel_ids: Self::transmute_ids_to_atomic(voxel_ids),
                    info: default(),
                }.as_optimized()
            },
            FillType::AllSame(id) => {
                assert_eq!(
                    len, 1,
                    "`voxel_ids.len()` should be equal to 1 due to fill_type, but it's {len}",
                );

                assert_eq!(
                    voxel_ids[0], id,
                    "`voxel_ids[0]` should be equal to `fill_type`'s id",
                );

                Self::new_same_filled(chunk_pos, id)
            },
        }
    }

    /// Sets [voxel id][Id] to `new_id` by it's index in array.
    /// Note that it does not drop all meshes that can possibly hold old id.
    /// And note that it may unoptimize chunk even if it can be.
    /// Returns old [id][Id].
    /// 
    /// # Error
    /// 
    /// Returns [`Err`] if `idx` is out of bounds.
    pub fn set_id(&mut self, idx: usize, new_id: VoxelId) -> Result<VoxelId, EditError> {
        ensure!(idx < Self::VOLUME, EditError::IdxOutOfBounds { idx, len: Self::VOLUME });

        let old_id = match self.info.load(Relaxed).get_fill_type() {
            FillType::Unspecified => {
                let old_id = self.voxel_ids[idx].swap(new_id, AcqRel);
                if old_id != new_id { self.optimize() }
                old_id
            },

            FillType::AllSame(old_id) => if old_id != new_id {
                self.unoptimize();
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
    /// - `idx` < `Chunk::VOLUME`
    /// - `self.info.fill_type` should be [`FillType::Unspecified`].
    pub unsafe fn set_id_unchecked(&self, idx: usize, new_id: VoxelId) -> VoxelId {
        self.voxel_ids.get_unchecked(idx).swap(new_id, AcqRel)
    }

    /// Sets voxel's id with position `pos` to `new_id` and returns old [id][Id]. If voxel is 
    /// set then this function should drop all its meshes.
    /// 
    /// # Error
    /// 
    /// Returns `Err` if `new_id` is not valid or `pos` is not in this [`Chunk`].
    pub fn set_voxel(&mut self, pos: IVec3, new_id: VoxelId) -> Result<VoxelId, EditError> {
        ensure!(voxel::is_id_valid(new_id), EditError::InvalidId(new_id));

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
    pub fn fill_voxels(&mut self, pos_from: IVec3, pos_to: IVec3, new_id: VoxelId) -> Result<bool, EditError> {
        ensure!(voxel::is_id_valid(new_id), EditError::InvalidId(new_id));

        let pos = self.pos.load(Relaxed);
        let local_pos_from = Self::global_to_local_pos_checked(pos, pos_from)?;

        Self::global_to_local_pos_checked(pos, pos_to - IVec3::ONE)?;
        let local_pos_to = Self::global_to_local_pos(pos, pos_to);

        self.unoptimize();

        let mut is_changed = false;

        for local_pos in Range3d::from(local_pos_from..local_pos_to) {
            // We can safely not to check idx due to previous check.
            let idx = Self::voxel_pos_to_idx_unchecked(local_pos);
            
            let old_id = self.get_id(idx).expect("idx should be valid");
            ensure_or!(old_id != new_id, continue);

            is_changed = true;

            // * # Safety
            // * 
            // * Safe, because `idx` is valid and `self` is unoptimized.
            unsafe { self.set_id_unchecked(idx, new_id) };
        }

        self.optimize();

        Ok(is_changed)
    }

    /// Gives iterator over all id-vectors in chunk (or relative to chunk voxel positions).
    pub fn local_pos_iter() -> Range3d {
        Range3d::from(IVec3::ZERO..Self::SIZES.into())
    }

    /// Gives iterator over all id-vectors in chunk (or relative to chunk voxel positions).
    pub fn global_pos_iter(chunk_pos: IVec3) -> impl ExactSizeIterator<Item = IVec3> {
        Self::local_pos_iter()
            .map(move |pos| Self::local_to_global_pos(chunk_pos, pos))
    }

    /// Gives iterator that yields iterator over some chunk of voxels.
    pub fn chunked_pos_iter(sub_chunk_size: usize) -> impl ExactSizeIterator<Item = Range3d> {
        Range3d::split_chunks(
            Self::SIZES.into(),
            IVec3::splat(sub_chunk_size as i32),
        )
    }

    /// Applies storage optimizations to voxel array.
    pub fn as_optimized(mut self) -> Self {
        self.optimize();
        self
    }

    /// Applies storage optimizations to [voxel array][Chunk].
    pub fn optimize(&mut self) {
        self.unoptimize();

        ensure_or!(self.is_generated(), return);
        
        let mut info = ChunkInfo {
            active_lod: self.info.load(Acquire).get_active_lod(),
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

        let packed_info = ChunkInfoPacked::from(info);

        self.info.store(packed_info, Release);
    }

    /// Disapplies storage optimizations.
    pub fn unoptimize(&mut self) {
        let mut info = self.info.load(Acquire).unpack();

        if let FillType::AllSame(id) = info.fill_type {
            self.voxel_ids = std::iter::from_fn(|| Some(Atomic::new(id)))
                .take(Self::VOLUME)
                .collect()
        }

        info.fill_type = FillType::Unspecified;

        self.info.store(info.into(), Release);
    }

    /// Converts chunk position to world position.
    pub fn local_to_global(chunk_pos: IVec3) -> IVec3 {
        chunk_pos * Self::SIZE as i32
    }

    /// Converts all in-chunk world positions to that chunk position.
    pub fn global_to_local(world_pos: IVec3) -> IVec3 {
        world_pos.div_euclid(IVec3::from(Self::SIZES))
    }

    /// Computes global position from relative to chunk position.
    pub fn local_to_global_pos(chunk_pos: IVec3, relative_voxel_pos: IVec3) -> IVec3 {
        Self::local_to_global(chunk_pos) + relative_voxel_pos
    }

    /// Computes local (relative to chunk) position from global position.
    pub fn global_to_local_pos(chunk_pos: IVec3, global_voxel_pos: IVec3) -> IVec3 {
        global_voxel_pos - Self::local_to_global(chunk_pos)
    }

    /// Computes local (relative to chunk) position from global position.
    /// 
    /// # Error
    /// 
    /// Returns [`Err`] if local position is out of [chunk][Chunk] bounds.
    pub fn global_to_local_pos_checked(chunk_pos: IVec3, global_voxel_pos: IVec3) -> Result<IVec3, EditError> {
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
    /// Returns [`None`] if `pos` < [`IVec3::ZERO`][IVec3] or `pos` >= [`Chunk::SIZES`][Chunk].
    pub fn voxel_pos_to_idx(pos: IVec3) -> Option<usize> {
        ensure_or!(pos.x >= 0 && pos.y >= 0 && pos.z >= 0, return None);

        let idx = iterator::get_index(&pos.to_array().map(|x| x as usize), &[Self::SIZE; 3]);

        (idx < Self::VOLUME).then_some(idx)
    }

    /// Gives index in voxel array by it's 3D-index (or relative to chunk position)
    /// without idx-safe ckeck.
    pub fn voxel_pos_to_idx_unchecked(pos: IVec3) -> usize {
        iterator::get_index(&pos.to_array().map(|x| x as usize), &[Self::SIZE; 3])
    }

    // FIXME: write mesh analog
    // Partitions [mesh][crate::graphics::mesh::Mesh] of this [chunk][Chunk].
    // pub fn partition_mesh(&self, mesh: &mut ChunkMesh, chunk_adj: ChunkAdj, device: &Device) {
    //     let meshes = self.make_partitial_meshes(chunk_adj);
    //     mesh.upload_partial_meshes(device, &meshes);
    // }

    // FIXME: render a chunk
    // Renders a [`Chunk`].
    // pub fn render<'rp>(
    //     &self, mesh: &'rp mut ChunkMesh, lod: Lod,
    //     pipeline: &'rp ChunkRenderPipeline, pass: &mut RenderPass<'rp>,
    // ) -> Result<(), ChunkRenderError> {
    //     ensure_or!(!self.is_empty(), return Ok(()));

    //     mesh.active_lod = Some(lod);
    //     mesh.render(pipeline, pass)
    // }

    // FIXME: remove all
    // Sets active LOD to given value.
    // 
    // # Panic
    // 
    // Panics if `lod` is not available.
    // pub fn set_active_lod(&self, mesh: &ChunkMesh, lod: Lod) {
    //     self.try_set_active_lod(mesh, lod)
    //         .expect("new LOD value should be available")
    // }

    // /// Tries to set active LOD to given value.
    // pub fn try_set_active_lod(&self, mesh: &ChunkMesh, lod: Lod) -> Result<(), SetLodError> {
    //     let mut info: ChunkInfo = self.info.load(Acquire).into();

    //     mesh.get_available_lods()
    //         .contains(&lod)
    //         .then(|| info.active_lod = Some(lod))
    //         .ok_or(SetLodError::SetActiveLod { tried: lod, active: info.active_lod })?;

    //     self.info.store(info.into(), Release);
    //     Ok(())
    // }

    // /// Tries to set LOD value that have least difference with given value.
    // /// If there is at least one LOD it will return `Some(..)` with that value, otherwise, `None`.
    // pub fn try_set_best_fit_lod(&self, mesh: &ChunkMesh, lod: Lod) -> Option<Lod> {
    //     let best_fit = mesh.get_available_lods()
    //         .into_iter()
    //         .min_by_key(|elem| elem.abs_diff(lod))?;

    //     self.set_active_lod(mesh, best_fit);

    //     Some(best_fit)
    // }

    // /// Gives list of all possible LODs.
    // pub fn get_possible_lods() -> [Lod; Self::N_LODS] {
    //     array_init(|i| i as Lod)
    // }

    // pub fn can_render_active_lod(&self, mesh: &ChunkMesh) -> bool {
    //     matches!(
    //         self.info.load(Relaxed).get_active_lod(),
    //         Some(lod) if mesh.get_available_lods().contains(&lod)
    //     )
    // }
}

impl Clone for Chunk {
    fn clone(&self) -> Self {
        Self {
            pos: Atomic::new(self.pos.load(Relaxed)),
            voxel_ids: unsafe {
                Self::transmute_ids_to_atomic(
                    mem::transmute::<_, &Vec<VoxelId>>(&self.voxel_ids).clone()
                )
            },
            info: Atomic::new(self.info.load(Relaxed)),
        }
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



#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ChunkInfo {
    pub fill_type: FillType,
    pub is_filled: bool,
    pub active_lod: Option<Lod>,
}

impl From<ChunkInfoPacked> for ChunkInfo {
    fn from(value: ChunkInfoPacked) -> Self {
        value.unpack()
    }
}



#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, NoUninit, Display)]
pub struct ChunkInfoPacked {
    /// `bits`: `[fill_id u16][FillType u1: 0 = Unspecified, 1 = AllSame]`\
    /// `[is_filled u1: 0 = false, 1 = true][padding u14][active_lod u32, u32::MAX for None]`
    pub bits: u64,
}
assert_impl_all!(ChunkInfoPacked: Send, Sync);

impl ChunkInfoPacked {
    pub const NO_ACTIVE_LOD: u32 = u32::MAX;

    pub const fn new(value: ChunkInfo) -> Self {
        let mut bits = 0;

        match value.fill_type {
            FillType::Unspecified => (),
            FillType::AllSame(id) => {
                bits |= (id as u64) << 48;
                bits |= 1_u64 << 47;
            },
        }

        bits |= (value.is_filled as u64) << 46;

        match value.active_lod {
            None => bits |= u32::MAX as u64,
            Some(lod) => bits |= lod as u64,
        }

        Self { bits }
    }

    pub const fn unpack(self) -> ChunkInfo {
        ChunkInfo {
            fill_type: match (self.bits >> 47) & 1 {
                0 => FillType::Unspecified,
                1 => FillType::AllSame((self.bits >> 48) as VoxelId),
                _ => unreachable!(),
            },
            is_filled: 0 != (self.bits >> 46) & 1,
            active_lod: if self.bits & u32::MAX as u64 != ChunkInfoPacked::NO_ACTIVE_LOD as u64 {
                Some(self.bits as u32)
            } else { None }
        }
    }

    pub const fn is_fill_type_unspecified(self) -> bool {
        0 == self.bits & (1_u64 << 47)
    }

    pub const fn is_fill_type_all_same(self) -> bool {
        !self.is_fill_type_unspecified()
    }

    pub const fn get_fill_type(self) -> FillType {
        match self.get_fill_id() {
            None => FillType::Unspecified,
            Some(id) => FillType::AllSame(id),
        }
    }

    pub const fn is_filled(self) -> bool {
        0 != self.bits & (1_u64 << 46)
    }

    pub const fn get_fill_id(self) -> Option<VoxelId> {
        if self.is_fill_type_all_same() {
            Some((self.bits >> 48) as u16)
        } else { None }
    }

    pub const fn get_active_lod(self) -> Option<Lod> {
        if self.bits as u32 != u32::MAX {
            Some(self.bits as u32)
        } else { None }
    }
}

impl ConstDefault for ChunkInfoPacked {
    const DEFAULT: Self = Self::new(ChunkInfo {
        fill_type: FillType::Unspecified,
        is_filled: false,
        active_lod: None,
    });
}

impl Default for ChunkInfoPacked {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl From<ChunkInfo> for ChunkInfoPacked {
    fn from(value: ChunkInfo) -> Self {
        Self::new(value)
    }
}



#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum FillType {
    #[default]
    Unspecified,
    AllSame(VoxelId),
}

impl From<Option<VoxelId>> for FillType {
    fn from(value: Option<VoxelId>) -> Self {
        match value {
            None => Self::Unspecified,
            Some(id) => Self::AllSame(id),
        }
    }
}

impl AsBytes for FillType {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Unspecified => vec![0],
            Self::AllSame(id) => itertools::chain!([1], id.as_bytes()).collect()
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
        u8::static_size() + match self {
            Self::Unspecified => 0,
            Self::AllSame(_) => VoxelId::static_size(),
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
    PosIdConversion(IVec3),

    #[error("position is out of chunk bounds")]
    PosOutOfBounds {
        pos: IVec3,
        chunk_pos: IVec3,
    },

    #[error("index out of bounds: index is {idx} but len is {len}")]
    IdxOutOfBounds {
        idx: usize,
        len: usize,
    },

    #[error("invalid id {0}")]
    InvalidId(VoxelId),
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_info_packed() {
        let test = |info|
            assert_eq!(ChunkInfoPacked::new(info).unpack(), info);

        test(ChunkInfo {
            fill_type: FillType::Unspecified,
            is_filled: true,
            active_lod: Some(0),
        });

        test(ChunkInfo {
            fill_type: FillType::AllSame(23),
            is_filled: true,
            active_lod: None,
        });
    }
}