#![allow(unused, clippy::diverging_sub_expression, clippy::let_unit_value)]

use super::commands;

use {
    crate::{
        prelude::*,
        terrain::{
            chunk::{
                prelude::*, EditError, Sides, VoxelId,
                tasks::{FullTask, LowTask, Task, GenTask, PartitionTask},
                mesh::Resolution,
                commands::Command,
            },
            voxel::{self, Voxel},
        },
        saves::Save,
        graphics::*,
        geometry::frustum::Frustum,
    },
    math_linear::math::ray::space_3d::Line,
    std::io,
    tokio::task::{JoinHandle, JoinError},
};



pub static GENERATOR_SIZES: Mutex<[usize; 3]> = Mutex::new(USize3::ZERO.as_array());



saves::define_key! {
    pub enum ChunkArrSaveKey { Sizes, Array }
}



/// Represents 3d array of [`Chunk`]s. Can control their [mesh][ChunkMesh] generation, etc.
#[derive(Debug)]
pub struct ChunkArray {
    pub array_entity: Entity,
    pub chunk_entities: Vec<Entity>,

    pub sizes: USize3,
    pub lod_threashold: f32,

    pub tasks: ChunkArrayTasks,
}
assert_impl_all!(ChunkArray: Send, Sync, Component);

impl ChunkArray {
    // FIXME: add camera input
    // const MAX_TRACE_STEPS: usize = 1024;

    /// Generates new chunks.
    /// 
    /// # Panic
    /// 
    /// Panics if `sizes` is not valid. See `ChunkArray::validate_sizes()`.
    pub async fn new(world: &mut World, sizes: USize3) -> AnyResult<Self> {
        Self::validate_sizes(sizes)?;
        let (start_pos, end_pos) = Self::pos_bounds(sizes);

        let chunks = Range3d::from(start_pos..end_pos)
            .map(move |pos| Chunk::new(pos, sizes))
            .map(Arc::new)
            .collect();

        Self::from_chunks(world, sizes, chunks).await
    }

    /// Constructs [`ChunkArray`] with passed in chunks.
    /// 
    /// # Panic
    /// 
    /// Panics if `sizes` is not valid. See `ChunkArray::validate_sizes()`.
    pub async fn from_chunks(world: &mut World, sizes: USize3, chunks: Vec<ChunkRef>) -> AnyResult<Self> {
        Self::validate_sizes(sizes)?;
        let volume = Self::volume(sizes);

        ensure!(
            chunks.len() == volume,
            StrError::from(format!(
                "passed in chunk `Vec` should have same size as
                passed in sizes, but sizes: {sizes}, len: {len}",
                len = chunks.len(),
            ))
        );

        let self_entity = world.spawn_empty();

        // Self::make_binds(world, self_entity).await;

        let chunk_entities = world.spawn_batch(
            chunks.into_iter().map(|chunk| {
                (chunk, Mesh::default())
            })
        ).collect();

        Ok(Self {
            sizes,
            chunk_entities,
            array_entity: self_entity,
            lod_threashold: 5.8,
            tasks: default(),
        })
    }

    // pub async fn make_binds(world: &mut World, self_entity: Entity) {
    //     let graphics = world.resource::<&Graphics>().unwrap();
    //     let cam_uniform = world.resource::<&CameraUniformBuffer>().unwrap();
    //     
    //     let binds = ChunkBinds::new(&graphics.context.device, &graphics.context.queue)
    //         .await
    //         .expect("failed to make binds for chunk array");

    //     let layout = ChunkPipelineLayout::new(
    //         &graphics.context.device,
    //         &graphics.common_uniform.binds,
    //         &binds,
    //         &cam_uniform.binds,
    //     );

    //     let pipeline = ChunkRenderPipeline::new(&graphics.context.device, &layout).await;

    //     drop(graphics);
    //     drop(cam_uniform);

    //     world.insert(self_entity, (binds, pipeline, layout)).unwrap();
    // }

    /// Constructs [`ChunkArray`] with empty chunks.
    /// 
    /// # Panic
    /// 
    /// Panics if `sizes` is not valid. See `ChunkArray::validate_sizes()`.
    pub async fn new_empty_chunks(world: &mut World, sizes: USize3) -> AnyResult<Self> {
        Self::validate_sizes(sizes)?;
        let (start_pos, end_pos) = Self::pos_bounds(sizes);

        let chunks = Range3d::from(start_pos..end_pos)
            .map(Chunk::new_empty)
            .map(ChunkRef::new)
            .collect();

        Self::from_chunks(world, sizes, chunks).await
    }

    /// Computes start and end poses from chunk array sizes.
    pub fn pos_bounds(sizes: USize3) -> (Int3, Int3) {
        (
            Self::volume_index_to_pos(sizes, USize3::ZERO),
            Self::volume_index_to_pos(sizes, sizes),
        )
    }

    /// Checks that sizes is valid.
    /// 
    /// # Error
    /// 
    /// Returns `Err` if `sizes.x * sizes.y * sizes.z` > `MAX_CHUNKS`.
    pub fn validate_sizes(sizes: USize3) -> AnyResult<()> {
        let volume = Self::volume(sizes);
        (volume <= cfg::terrain::MAX_CHUNKS)
            .then_some(())
            .ok_or_else(||
                StrError::from(format!("cannot allocate too many chunks: {volume}")).into()
            )
    }

    /// Gives empty [`ChunkArray`].
    pub async fn new_empty(world: &mut World) -> AnyResult<Self> {
        Self::new_empty_chunks(world, USize3::ZERO).await
    }

    /// Clears all [chunk array][ChunkArray] stuff out from the `world`.
    pub fn remove(world: &mut World) {
        let mut this = world.resource::<&mut Self>().unwrap();

        let array_entity = this.array_entity;
        let mut chunk_entities = mem::take(&mut this.chunk_entities);

        drop(this);

        world.despawn(array_entity).unwrap();

        for chunk_entity in chunk_entities.drain(..) {
            world.despawn(chunk_entity).unwrap();
        }
    }

    pub async fn save_to_file(
        sizes: USize3, chunks: Vec<ChunkRef>, save_name: impl Into<String>, save_path: &str,
    ) -> io::Result<()> {
        let save_name = save_name.into();

        logger::scope!(from = "chunk-array", "saving to {save_name} in {save_path}");

        let is_all_generated = chunks.iter()
            .all(|chunk| chunk.is_generated());

        let volume = sizes.volume();
        
        assert!(is_all_generated, "chunks should be generated to save them to file");
        assert_eq!(volume, chunks.len(), "chunks should have same length as sizes volume");

        let loading = loading::start_new("Saving chunks");

        Save::builder(save_name)
            .create(save_path).await?
            .write(&sizes, ChunkArrSaveKey::Sizes).await
            .pointer_array(volume, ChunkArrSaveKey::Array, |i| {
                let chunks = &chunks;
                let loading = &loading;

                async move {
                    loading.refresh(loading::Discrete(i, volume));
                    Self::chunk_as_bytes(&chunks[i])
                }
            }).await
            .save()
            .await?;

        Ok(())
    }

    pub async fn read_from_file(save_name: &str, save_path: &str) -> io::Result<(USize3, Vec<Chunk>)> {
        logger::scope!(from = "chunk-array", "reading chunks from {save_name} in {save_path}");

        let loading = loading::start_new("Reading chunks");

        let mut save = Save::builder(save_name)
            .open(save_path)
            .await?;
        
        let sizes: USize3 = save.read(ChunkArrSaveKey::Sizes).await;

        let chunks = save.read_pointer_array(ChunkArrSaveKey::Array, |i, bytes| {
            let loading = &loading;

            async move {
                loading.refresh(loading::Discrete(i, sizes.volume()));
                Self::chunk_from_bytes(&bytes)
            }
        }).await;

        Ok((sizes, chunks))
    }

    /// Reinterprets [chunk][Chunk] as bytes. It uses Huffman's compresstion.
    pub fn chunk_as_bytes(chunk: &Chunk) -> Vec<u8> {
        use { bit_vec::BitVec, huffman_compress::CodeBuilder };

        let pos = chunk.pos.load(Relaxed);

        match chunk.info.load(Relaxed).get_fill_type() {
            FillType::AllSame(id) => itertools::chain!(
                FillType::AllSame(id).as_bytes(),
                pos.as_bytes(),
            ).collect(),

            FillType::Unspecified => {
                let n_voxels = chunk.voxel_ids.len();
                assert_eq!(
                    n_voxels, Chunk::VOLUME,
                    "cannot save unknown-sized chunk with size {n_voxels}",
                );

                let freqs = Self::count_voxel_frequencies(
                    chunk.voxel_ids.iter()
                        .map(|id| id.load(Relaxed))
                );

                let (book, _) = CodeBuilder::from_iter(
                    freqs.iter().map(|(&k, &v)| (k, v))
                ).finish();
                let mut bits = BitVec::new();

                for voxel_id in chunk.voxel_ids.iter() {
                    let voxel_id = voxel_id.load(Relaxed);
                    book.encode(&mut bits, &voxel_id)
                        .expect("voxel id should be in the book");
                }

                itertools::chain!(
                    FillType::Unspecified.as_bytes(),
                    freqs.as_bytes(),
                    pos.as_bytes(),
                    bits.as_bytes(),
                ).collect()
            }
        }
    }

    /// Reads bytes as [chunk][Chunk].
    pub fn chunk_from_bytes(bytes: &[u8]) -> Chunk {
        use { bit_vec::BitVec, huffman_compress::CodeBuilder };

        let mut reader = ByteReader::new(bytes);
        let fill_type: FillType = reader.read()
            .expect("failed to reinterpret bytes");

        match fill_type {
            FillType::Unspecified => {
                let freqs: HashMap<VoxelId, usize> = reader.read()
                    .expect("failed to read frequencies map from bytes");

                let pos: Int3 = reader.read()
                    .expect("failed to read chunk pos from bytes");

                let bits: BitVec = reader.read()
                    .expect("failed to read `BitVec` from bytes");

                let (_, tree) = CodeBuilder::from_iter(freqs).finish();
                let voxel_ids: Vec<_> = tree.unbounded_decoder(bits).collect();

                let is_id_valid = voxel_ids.iter()
                    .copied()
                    .all(voxel::is_id_valid);

                assert!(is_id_valid, "Voxel ids in voxel array should be valid");
                assert_eq!(voxel_ids.len(), Chunk::VOLUME, "There's should be Chunk::VOLUME voxels");

                Chunk::from_voxels(voxel_ids, pos)
            },

            FillType::AllSame(id) => {
                let pos: Int3 = reader.read()
                    .expect("failed to read chunk pos from bytes");

                Chunk::new_same_filled(pos, id)
            }
        }
    }

    /// Gets a [chunk][Chunk] from the `world` without panic.
    pub fn get_chunk(&self, world: &World, pos: Int3) -> Option<ChunkRef> {
        let chunk_pos = Chunk::global_to_local(pos);
        let chunk_idx = Self::pos_to_idx(self.sizes, chunk_pos)?;
        self.get_chunk_by_idx(world, chunk_idx)
    }

    /// Get a [chunk][Chunk] from the `world`.
    pub fn chunk(&self, world: &World, pos: Int3) -> ChunkRef {
        self.get_chunk(world, pos)
            .unwrap_or_else(|| panic!("there's no chunk in world with position {pos}"))
    }

    /// Gets a [chunk][Chunk] from the `world` without panic.
    pub fn get_chunk_by_idx(&self, world: &World, idx: usize) -> Option<ChunkRef> {
        world.get::<&ChunkRef>(self.chunk_entities[idx]).ok().as_deref().cloned()
    }

    /// Get a [chunk][Chunk] from the `world`.
    pub fn chunk_by_idx(&self, world: &World, idx: usize) -> ChunkRef {
        self.get_chunk_by_idx(world, idx)
            .unwrap_or_else(|| panic!("there's no chunk in world with idx {idx}"))
    }

    /// Sets voxel's id with position `pos` to `new_id` and returns old [`Id`]. If voxel is 
    /// set then this function should drop all its meshes and the neighbor ones.
    /// 
    /// # Error
    /// 
    /// Returns [`Err`] if `new_id` is not valid or `pos` is not in this [chunk array][ChunkArray].
    pub fn set_voxel(&mut self, world: &World, pos: Int3, new_id: VoxelId) -> Result<VoxelId, EditError> {
        // We know that `chunk_idx` is valid so we can get-by-index.
        let mut chunk = self.get_chunk(world, pos)
            .ok_or(EditError::PosIdConversion(pos))?;

        let old_id = unsafe {
            Arc::get_mut_unchecked(&mut chunk)
                .set_voxel(pos, new_id)?
        };

        Ok(old_id)
    }

    /// Gives voxel if it is in the [array][ChunkArray].
    pub fn get_voxel(&self, world: &World, pos: Int3) -> Option<Voxel> {
        match self.get_chunk(world, pos)?.get_voxel_global(pos) {
            ChunkOption::Voxel(voxel) => Some(voxel),
            ChunkOption::OutsideChunk => unreachable!("pos {pos} is indeed in that chunk"),
            ChunkOption::Failed => None,
        }
    }

    /// Fills volume of voxels to same [id][Id] and returnes `is_changed`.
    pub fn fill_voxels(&mut self, world: &World, pos_from: Int3, pos_to: Int3, new_id: VoxelId) -> Result<bool, EditError> {
        let chunk_pos_from = Chunk::global_to_local(pos_from);
        let chunk_pos_to   = Chunk::global_to_local(pos_to + Int3::from(Chunk::SIZES) - Int3::ONE);

        Self::pos_to_idx(self.sizes, chunk_pos_from)
            .ok_or(EditError::PosIdConversion(chunk_pos_from))?;

        Self::pos_to_idx(self.sizes, chunk_pos_to - Int3::ONE)
            .ok_or(EditError::PosIdConversion(chunk_pos_to - Int3::ONE))?;

        let mut is_changed = false;

        for chunk_pos in Range3d::from(chunk_pos_from..chunk_pos_to) {
            let idx = Self::pos_to_idx(self.sizes, chunk_pos)
                .expect("chunk_pos already valid");

            let min_voxel_pos = Chunk::local_to_global(chunk_pos);
            let end_voxel_pos = min_voxel_pos + Int3::from(Chunk::SIZES);

            let pos_from = Int3::new(
                Ord::max(pos_from.x, min_voxel_pos.x),
                Ord::max(pos_from.y, min_voxel_pos.y),
                Ord::max(pos_from.z, min_voxel_pos.z),
            );

            let pos_to = Int3::new(
                Ord::min(pos_to.x, end_voxel_pos.x),
                Ord::min(pos_to.y, end_voxel_pos.y),
                Ord::min(pos_to.z, end_voxel_pos.z),
            );

            let mut chunk = self.chunk_by_idx(world, idx);
            let chunk_changed = unsafe {
                Arc::get_mut_unchecked(&mut chunk)
                    .fill_voxels(pos_from, pos_to, new_id)?
            };

            if chunk_changed {
                is_changed = true;
                
                for idx in Self::get_adj_chunks_idxs(self.sizes, chunk_pos).as_array().into_iter().flatten() {
                    let mut mesh = self.chunk_component::<&mut Mesh>(world, idx);
                    *mesh = default();
                }
            }
        }

        Ok(is_changed)
    }

    pub fn chunk_component<'w, T: ComponentRef<'w>>(&self, world: &'w World, idx: usize) -> T::Ref {
        world.get::<T>(self.chunk_entities[idx])
            .expect("failed to get a component from chunk entity")
    }

    /// Drops all meshes from each [chunk][Chunk].
    pub fn drop_all_meshes(&self, world: &World) {
        for idx in 0..self.chunk_entities.len() {
            let mut mesh = self.chunk_component::<&mut Mesh>(world, idx);
            _ = mem::take(mesh.deref_mut());
        }
    }

    fn count_voxel_frequencies(voxel_ids: impl IntoIterator<Item = VoxelId>) -> HashMap<VoxelId, usize> {
        let mut result = HashMap::new();

        for id in voxel_ids.into_iter() {
            match result.get_mut(&id) {
                None => drop(result.insert(id, 1)),
                Some(freq) => *freq += 1,
            }
        }

        result
    }

    pub async fn apply_new(
        world: &mut World, sizes: USize3, chunks: Vec<Chunk>,
    ) -> AnyResult<()> {
        ensure_eq!(
            Self::volume(sizes),
            chunks.len(),
            StrError::from("chunk-array should have same len as sizes")
        );
        
        let chunks = chunks.into_iter()
            .map(Arc::new)
            .collect();

        let new_array = ChunkArray::from_chunks(world, sizes, chunks).await?;

        ChunkArray::remove(world);

        let mut this = world.resource::<&mut Self>()?;

        this.tasks.stop_all();
        _ = mem::replace(this.deref_mut(), new_array);

        Ok(())
    }

    /// Gives chunk count.
    pub fn volume(arr_sizes: USize3) -> usize {
        arr_sizes.x * arr_sizes.y * arr_sizes.z
    }

    pub fn voxel_pos_to_volume_index(voxel_pos: Int3, chunk_array_sizes: USize3) -> Option<USize3> {
        let chunk_pos = Chunk::global_to_local(voxel_pos);
        let local_voxel_pos = Chunk::global_to_local_pos(chunk_pos, voxel_pos);

        let chunk_coord_idx = Self::local_pos_to_volume_index(chunk_array_sizes, chunk_pos)?;
        let voxel_offset_by_chunk: USize3 = Chunk::local_to_global(chunk_coord_idx.into()).into();

        Some(voxel_offset_by_chunk + USize3::from(local_voxel_pos))
    }

    /// Convertes 3d index into chunk pos.
    pub fn volume_index_to_pos(sizes: USize3, coord_idx: USize3) -> Int3 {
        Int3::from(coord_idx) - Int3::from(sizes) / 2
    }

    /// Convertes chunk pos to 3d index.
    pub fn local_pos_to_volume_index(sizes: USize3, pos: Int3) -> Option<USize3> {
        let sizes = Int3::from(sizes);
        let shifted = pos + sizes / 2;

        (
            0 <= shifted.x && shifted.x < sizes.x &&
            0 <= shifted.y && shifted.y < sizes.y &&
            0 <= shifted.z && shifted.z < sizes.z
        ).then_some(shifted.into())
    }

    /// Convertes 3d index to an array index.
    pub fn volume_index_to_linear(sizes: USize3, coord_idx: USize3) -> usize {
        sdex::get_index(&coord_idx.as_array(), &sizes.as_array())
    }

    /// Convertes [chunk][Chunk] pos to an array index.
    pub fn pos_to_idx(sizes: USize3, pos: Int3) -> Option<usize> {
        let coord_idx = Self::local_pos_to_volume_index(sizes, pos)?;
        Some(Self::volume_index_to_linear(sizes, coord_idx))
    }

    /// Convertes array index to 3d index.
    pub fn linear_index_to_volume(idx: usize, sizes: USize3) -> USize3 {
        iterator::linear_index_to_volume(idx, sizes)
    }

    /// Converts array index to chunk pos.
    pub fn index_to_pos(idx: usize, sizes: USize3) -> Int3 {
        let coord_idx = Self::linear_index_to_volume(idx, sizes);
        Self::volume_index_to_pos(sizes, coord_idx)
    }

    /// Gives adjacent chunks references by center chunk position.
    pub fn get_adj_chunks(&self, world: &World, pos: Int3) -> ChunkAdj {
        Self::get_adj_chunks_idxs(self.sizes, pos)
            .map(|opt| opt.map(|idx| self.chunk_by_idx(world, idx)))
    }

    /// Gives '`iterator`' over adjacent to `pos` array indices.
    pub fn get_adj_chunks_idxs(sizes: USize3, pos: Int3) -> Sides<Option<usize>> {
        Range3d::adj_iter(pos)
            .map(|pos| Self::pos_to_idx(sizes, pos))
            .collect()
    }

    /// Gives iterator over chunk coordinates.
    pub fn chunk_pos_range(sizes: USize3) -> Range3d {
        let (start, end) = Self::pos_bounds(sizes);
        Range3d::from(start..end)
    }

    /// Gives iterator over all chunk's adjacents.
    pub fn adj_iter<'s>(&'s self, world: &'s World) -> impl ExactSizeIterator<Item = ChunkAdj> + 's {
        Self::chunk_pos_range(self.sizes)
            .map(|pos| self.get_adj_chunks(world, pos))
    }

    /// Gives desired [LOD][Lod] value for chunk positioned in `chunk_pos`.
    pub fn desired_lod_at(chunk_pos: Int3, cam_pos: vec3, threashold: f32) -> Lod {
        let chunk_size = Chunk::GLOBAL_SIZE;
        let cam_pos_in_chunks = cam_pos / chunk_size;
        let chunk_pos = vec3::from(chunk_pos);

        let dist = (chunk_pos - cam_pos_in_chunks + vec3::all(0.5)).len();
        cmp::min(
            (dist / threashold).floor() as Lod,
            Chunk::SIZE.ilog2() as Lod,
        )
    }

    /// Gives iterator over desired LOD for each chunk.
    pub fn desired_lod_iter(chunk_array_sizes: USize3, cam_pos: vec3, threashold: f32) -> impl ExactSizeIterator<Item = Lod> {
        Self::chunk_pos_range(chunk_array_sizes)
            .map(move |chunk_pos| Self::desired_lod_at(chunk_pos, cam_pos, threashold))
    }

    /// Gives iterator over all chunks in `world`.
    pub fn chunks<'s>(&'s self, world: &'s World) -> impl ExactSizeIterator<Item = ChunkRef> + 's {
        (0..self.chunk_entities.len())
            .map(|idx| self.chunk_by_idx(world, idx))
    }

    /// Gives iterator over all voxels in [`ChunkArray`].
    pub fn voxels<'s>(&'s self, world: &'s World) -> impl Iterator<Item = Voxel> + 's {
        self.chunks(world)
            .flat_map(|chunk| chunk.voxels().collect_vec())
    }

    /// Gives iterator over mutable chunks and their adjacents.
    pub fn chunks_with_adj<'s>(&'s self, world: &'s World) -> impl ExactSizeIterator<Item = (ChunkRef, ChunkAdj)> + '_ {
        Iterator::zip(self.chunks(world), self.adj_iter(world))
    }

    /// Gives [`Vec`] with [`ChunkRef`]s [`ChunkAdj`]s desired [lod][Lod].
    fn get_indices_sorted(&self, world: &World, cam_pos: vec3) -> Vec<usize> {
        let mut result = Vec::from_iter(0..self.chunk_entities.len());

        result.sort_by_key(|&idx| {
            let chunk = self.chunk_by_idx(world, idx);
            let pos = Chunk::local_to_global(chunk.pos.load(Relaxed));
            let dot = vec3::sqr(cam_pos - pos.into());
            
            NotNan::new(dot)
                .log_error("chunk-array", "square chunk distance is NaN")
        });

        result
    }

    /// TODO: missing docs
    #[profile]
    #[allow(clippy::await_holding_refcell_ref)]
    pub async fn update_meshes(
        world: &World, cam_pos: vec3, _frustum: &Frustum,
    ) -> Result<(), ChunkRenderError> {
        let mut this = world.resource::<&mut Self>().unwrap();

        let sizes = this.sizes;
        ensure_or!(sizes != USize3::ZERO, return Ok(()));

        let graphics = world.resource::<&Graphics>().unwrap();
        let device = &graphics.context.device;

        this.try_finish_all_tasks(world, device).await;

        for idx in this.get_indices_sorted(world, cam_pos) {
            let mut chunk = this.chunk_by_idx(world, idx);
            let chunk_pos = chunk.pos.load(Relaxed);
            
            let chunk_adj = this.get_adj_chunks(world, chunk_pos);
            let mut mesh = this.chunk_component::<&mut Mesh>(world, idx);

            let lod = Self::desired_lod_at(chunk_pos, cam_pos, this.lod_threashold);


            if !chunk.is_generated() {
                if this.tasks.is_gen_running(chunk_pos) {
                    if let Some(new_chunk) = this.tasks.try_finish_gen(chunk_pos).await {
                        this.tasks.stop_all_meshes(chunk_pos);

                        // * Safety:
                        // * Safe, because there's no chunk readers due to tasks drop above
                        unsafe {
                            let _ = mem::replace(Arc::get_mut_unchecked(&mut chunk), new_chunk);
                        }
                    }
                } else if this.tasks.can_start_mesh() {
                    this.tasks.start_gen(chunk_pos, sizes);
                    continue;
                } else {
                    continue;
                }
            }

            // FIXME: add partitioning (fix vbuffer unmap).
            // const CHUNK_MESH_PARTITION_DIST: f32 = 128.0;

            // let chunk_is_close_to_be_partitioned = vec3::len(
            //     vec3::from(Chunk::global_pos(chunk_pos))
            //     - cam_pos + vec3::from(Chunk::SIZES / 2)
            // ) <= CHUNK_MESH_PARTITION_DIST;

            // if chunk_is_close_to_be_partitioned &&
            //    !this.tasks.partition.contains_key(&chunk_pos) &&
            //    !mesh.is_partial()
            // {
            //     this.tasks.start_partitioning(&chunk, chunk_adj.clone());
            // }

            // let chunnk_can_be_connected =
            //     lod == 0 &&
            //     mesh.is_partial() &&
            //     !chunk_is_close_to_be_partitioned;

            // if chunnk_can_be_connected {
            //     mesh.connect_partitions(device);
            // }

            let can_set_new_lod =
                // mesh.get_available_lods().contains(&lod) ||
                this.tasks.is_mesh_running(chunk_pos, lod) &&
                this.tasks.try_finish_mesh(chunk_pos, lod, &mut mesh, device).await;

            if can_set_new_lod {
                // chunk.set_active_lod(&mesh, lod);
            } else if this.tasks.can_start_mesh() {
                this.tasks.start_mesh(&chunk, chunk_adj.clone(), lod).await;
            }

            this.tasks.stop_all_useless(lod, chunk_pos);

            // if !chunk.can_render_active_lod(&mesh) {
            //     chunk.try_set_best_fit_lod(&mesh, lod);
            // }

            // FIXME: make cam vis-check for light.
            // if chunk.can_render_active_lod(&mesh) /* && chunk.is_in_frustum(frustum) */ {
            //     mesh.active_lod = chunk.info.load(Relaxed).get_active_lod();
            //     mesh.enable();
            // } else {
            //     mesh.disable();
            // }
        }

        Ok(())
    }

    // #[profile]
    // pub fn render(
    //     world: &World, common_binds: &Binds,
    //     encoder: &mut CommandEncoder, view: TextureView,
    //     depth: Option<TextureView>,
    // ) -> AnyResult<()> {
    //     let this = world.resource::<&Self>()?;

    //     let cam_unform = world.resource::<&CameraUniformBuffer>()?;
    //     let binds = world.get::<&ChunkBinds>(this.array_entity)?;
    //     let pipeline = world.get::<&ChunkRenderPipeline>(this.array_entity)?;

    //     let mut query = world.query::<&Mesh>();

    //     let mut pass = RenderPass::new("chunk_array_render_pass", encoder, [&view], depth.as_ref());

    //     // for (_entity, mesh) in query.into_iter() {
    //     //     let Some(details) = todo!();// mesh.details() else { continue };

    //     //     Binds::bind_all(&mut pass, [
    //     //         common_binds,
    //     //         binds.by_details(details),
    //     //         &cam_unform.binds,
    //     //     ]);

    //     //     // mesh.render(&pipeline, &mut pass)?;
    //     // }

    //     Ok(())
    // }

    pub async fn try_finish_all_full_tasks(&mut self, world: &World, device: &Device) {
        let iter = self.tasks.full.iter_mut()
            .map(|(&pos, task)| (pos, task));

        for (pos, mesh) in Task::try_take_results(iter).await {
            self.tasks.full.remove(&pos);

            let idx = ChunkArray::pos_to_idx(self.sizes, pos)
                .expect("pos should be valid");

            let mut chunk_mesh = self.chunk_component::<&mut Mesh>(world, idx);
            // chunk_mesh.upload_full_mesh(device, &mesh);
        }
    }

    pub async fn try_finish_all_low_tasks(&mut self, world: &World, device: &Device) {
        let iter = self.tasks.low.iter_mut()
            .map(|(&idx, task)| (idx, task));

        for ((pos, lod), mesh) in Task::try_take_results(iter).await {
            self.tasks.low.remove(&(pos, lod));

            let idx = ChunkArray::pos_to_idx(self.sizes, pos)
                .expect("pos should be valid");

            let mut chunk_mesh = self.chunk_component::<&mut Mesh>(world, idx);
            // chunk_mesh.upload_low_mesh(device, &mesh, lod);
        }
    }

    pub async fn try_finish_all_gen_tasks(&mut self, world: &World) {
        let iter = self.tasks.voxels_gen.iter_mut()
            .map(|(&pos, task)| (pos, task));

        for (pos, voxels) in Task::try_take_results(iter).await {
            self.tasks.voxels_gen.remove(&pos);

            let mut chunk = self.chunk(world, pos);

            self.tasks.stop_all_meshes(pos);
            
            // * Safety:
            // * 
            // * Safe, because there's no chunk readers due to tasks drop above.
            unsafe {
                let _ = mem::replace(
                    Arc::get_mut_unchecked(&mut chunk),
                    Chunk::from_voxels(voxels, pos),
                );
            }
        }
    }

    pub async fn try_finish_all_partition_tasks(&mut self, world: &World, device: &Device) {
        let iter = self.tasks.partition.iter_mut()
            .map(|(&pos, task)| (pos, task));

        for (pos, partitions) in Task::try_take_results(iter).await {
            self.tasks.partition.remove(&pos);

            let idx = ChunkArray::pos_to_idx(self.sizes, pos)
                .expect("pos should be valid");

            let mut chunk_mesh = self.chunk_component::<&mut Mesh>(world, idx);
            // chunk_mesh.upload_partial_meshes(device, &partitions);
        }
    }

    pub async fn try_finish_all_tasks(&mut self, world: &World, device: &Device) {
        self.try_finish_all_full_tasks(world, device).await;
        self.try_finish_all_low_tasks(world, device).await;
        self.try_finish_all_partition_tasks(world, device).await;
        self.try_finish_all_gen_tasks(world).await;
    }

    /// Checks that [chunk][Chunk] [adjacent][ChunkAdj] are generated.
    pub async fn is_adj_generated(adj: &ChunkAdj) -> bool {
        adj.inner.iter()
            .filter_map(Option::as_deref)
            .all(Chunk::is_generated)
    }

    pub async fn generate_new(world: &mut World, sizes: USize3) {
        // world.resource::<&mut Self>().unwrap()
        //     .tasks
        //     .stop_all();
        
        let chunks = Self::new_empty_chunks(world, sizes).await;

        match chunks {
            Ok(new_chunks) => {
                let mut this = world.resource::<&mut Self>().unwrap();
                _ = mem::replace(this.deref_mut(), new_chunks);
            }
            Err(err) => logger::error!(from = "chunk-array", "{err}"),
        }
    }

    pub async fn proccess_command(
        world: &mut World, command: Command, change_tracker: &mut ChangeTracker,
    ) {
        let mut this = world.resource::<&mut Self>().unwrap();

        match command {
            Command::SetVoxel { pos, new_id } => {
                let old_id = this.set_voxel(world, pos, new_id)
                    .log_error("chunk-array", "failed to set voxel");

                if old_id != new_id {
                    change_tracker.track_voxel(pos);
                }
            },

            Command::FillVoxels { pos_from, pos_to, new_id } => {
                let _is_changed = this.fill_voxels(world, pos_from, pos_to, new_id)
                    .log_error("chunk-array", "failed to fill voxels");
            }

            Command::DropAllMeshes => this.drop_all_meshes(world),

            Command::GenerateNew { sizes } => {
                drop(this);
                Self::generate_new(world, sizes).await;
            },
        }
    }

    pub async fn process_commands(world: &mut World) {
        #![allow(clippy::await_holding_lock)]

        use crate::terrain::chunk::commands::*;

        let mut change_tracker = {
            let this = world.resource::<&Self>().unwrap();
            ChangeTracker::new(this.sizes)
        };
        
        while let Ok(command) = COMMAND_CHANNEL.lock().receiver.try_recv() {
            Self::proccess_command(world, command, &mut change_tracker).await;
        }

        let idxs_to_reload = change_tracker.idxs_to_reload_partitioning();
        let n_changed = idxs_to_reload.len();

        {
            let graphics = world.resource::<&Graphics>().unwrap();
            let device = &graphics.context.device;

            for (idx, partition_idx) in idxs_to_reload {
                Self::reload_chunk_partitioning(world, device, idx, partition_idx).await;
            }
        }

        if n_changed != 0 {
            logger::info!(from = "chunk-array", "{n_changed} chunks were updated!");
        }
    }

    pub async fn reload_chunk(&self, world: &World, device: &Device, idx: usize) {
        let chunk_pos = Self::index_to_pos(idx, self.sizes);
        let adj = self.get_adj_chunks(world, chunk_pos);

        if let Some(chunk) = self.get_chunk_by_idx(world, idx) {
            let mut mesh = self.chunk_component::<&mut Mesh>(world, idx);
            // chunk.generate_mesh(&mut mesh, 0, todo!("adj"), device);
        }
    }

    pub async fn reload_chunk_partitioning(
        world: &World, device: &Device, chunk_idx: usize, partition_idx: usize,
    ) {
        let this = world.resource::<&Self>().unwrap();

        let chunk_pos = Self::index_to_pos(chunk_idx, this.sizes);
        let adj = todo!("this.get_adj_chunks(world, chunk_pos)");

        // if let Some(chunk) = this.get_chunk_by_idx(world, chunk_idx) {
        //     let mut chunk_mesh = this.chunk_component::<&mut Mesh>(world, chunk_idx);

        //     if chunk_mesh.is_partial() {
        //         let mesh = todo!();//chunk.make_partition(&adj, partition_idx);
        //         chunk_mesh.upload_partition(device, &mesh, partition_idx);
        //     } else {
        //         // chunk.partition_mesh(&mut chunk_mesh, adj, device);
        //     }
        // }
    }

    pub fn trace_ray<'s>(&'s self, world: &'s World, ray: Line, max_steps: usize) -> impl Iterator<Item = Voxel> + 's {
        (0..max_steps)
            .filter_map(move |i| {
                let pos = ray.point_along(i as f32 * 0.125);
                let pos = Int3::new(
                    pos.x.round() as i32,
                    pos.y.round() as i32,
                    pos.z.round() as i32,
                );

                self.get_voxel(world, pos)
            })
    }

    // FIXME: add camera input
    // pub async fn proccess_camera_input(&mut self, world: &World, cam: &Camera) {
    //     use super::commands::command;

    //     let first_voxel = self.trace_ray(world, Line::new(cam.pos, cam.front), Self::MAX_TRACE_STEPS)
    //         .find(|voxel| !voxel.is_air());

    //     if let Some(voxel) = first_voxel && mouse::just_left_pressed() && cam.captures_mouse
    //     {
    //         command(Command::SetVoxel { pos: voxel.pos, new_id: AIR_VOXEL_DATA.id })
    //     }
    // }

    pub async fn update(world: &mut World) -> Result<(), UpdateError> {
        Self::process_commands(world).await;
        
        let mut this = world.resource::<&mut Self>().unwrap();

        // FIXME: add camera input
        // this.proccess_camera_input(world, cam).await;
        
        if keyboard::just_pressed_combo(&[Key::ControlLeft, Key::KeyS]) {
            let chunks = this.chunks(world).collect_vec();
            let handle = tokio::spawn(
                Self::save_to_file(this.sizes, chunks, "world", "world")
            );
            this.tasks.saving = Nullable::new(handle);
        }

        if !this.tasks.saving.is_null() && this.tasks.saving.is_finished() {
            let handle = this.tasks.saving.take();
            handle.await??;
        }

        if keyboard::just_pressed_combo(&[Key::ControlLeft, Key::KeyO]) {
            this.tasks.reading = Nullable::new(
                tokio::spawn(Self::read_from_file("world", "world"))
            );
        }

        if !this.tasks.reading.is_null() && this.tasks.reading.is_finished() {
            let handle = this.tasks.reading.take();
            let (sizes, arr) = handle.await??;

            drop(this);
            Self::apply_new(world, sizes, arr).await?;
        }

        Ok(())
    }

    pub fn spawn_control_window(world: &World, ui: &mut egui::Ui) {
        let mut chunk_array = world.resource::<&mut Self>().unwrap();
        
        ui.collapsing("Chunk array", |ui| {
            ui.label(format!(
                "{} chunk generation tasks",
                chunk_array.tasks.voxels_gen.len(),
            ));

            ui.label(format!(
                "{} mesh generation tasks",
                chunk_array.tasks.low.len() + chunk_array.tasks.full.len(),
            ));

            ui.label(format!(
                "{} partition generation tasks",
                chunk_array.tasks.partition.len(),
            ));

            ui.add(
                egui::Slider::new(&mut chunk_array.lod_threashold, 0.01..=20.0)
                    .prefix("LOD threashold: ")
            );

            ui.separator();

            ui.label("Generate new");

            let mut sizes = GENERATOR_SIZES.lock();

            ui.horizontal(|ui| {
                ui.label("Sizes: ");

                ui.add(egui::DragValue::new(&mut sizes[0]));
                ui.add(egui::DragValue::new(&mut sizes[1]));
                ui.add(egui::DragValue::new(&mut sizes[2]));
            });

            if ui.button("Generate").clicked() {
                commands::command(Command::GenerateNew { sizes: USize3::from(*sizes) });
            }
        });
    }
}



macros::sum_errors! {
    pub enum UpdateError { Join => JoinError, Save => io::Error, Other => AnyError }
}



#[derive(Clone, Debug, PartialEq)]
pub struct ChangeTracker {
    pub sizes: USize3,
    pub voxel_poses: HashSet<Int3>,
}

impl ChangeTracker {
    pub fn new(sizes: USize3) -> Self {
        Self { sizes, voxel_poses: HashSet::new() }
    }

    pub fn track_voxel(&mut self, voxel_pos: Int3) {
        self.voxel_poses.insert(voxel_pos);
    }

    pub fn idxs_to_reload(&self) -> HashSet<usize> {
        let mut result = HashSet::new();

        for &voxel_pos in self.voxel_poses.iter() {
            let chunk_pos = Chunk::global_to_local(voxel_pos);
            let local_pos = Chunk::global_to_local_pos(chunk_pos, voxel_pos);

            let Some(chunk_idx) = ChunkArray::pos_to_idx(self.sizes, chunk_pos) else { continue };

            for offset in iterator::offsets_from_border(local_pos, Int3::ZERO..Int3::from(Chunk::SIZES)) {
                let Some(idx) = ChunkArray::pos_to_idx(self.sizes, chunk_pos + offset) else { continue };
                result.insert(idx);
            }

            result.insert(chunk_idx);
        }

        result
    }

    pub fn idxs_to_reload_partitioning(&self) -> HashSet<(usize, usize)> {
        let mut result = HashSet::new();

        for &voxel_pos in self.voxel_poses.iter() {
            let chunk_sizes = Int3::from(Chunk::SIZES);

            let chunk_pos = Chunk::global_to_local(voxel_pos);
            let local_pos = Chunk::global_to_local_pos(chunk_pos, voxel_pos);
            let voxel_coord_idx = USize3::from(local_pos);

            let partition_idx = ChunkArray::volume_index_to_linear(
                USize3::all(2),
                voxel_coord_idx / (Chunk::SIZES / 2),
            );

            let Some(chunk_idx) = ChunkArray::pos_to_idx(self.sizes, chunk_pos) else { continue };

            result.insert((chunk_idx, partition_idx));

            let local_rem = local_pos.rem_euclid(chunk_sizes / 2);
            for offset in iterator::offsets_from_border(local_rem, Int3::ZERO .. chunk_sizes / 2) {
                let adj_voxel_global_pos = voxel_pos + offset;
                let adj_chunk_pos = Chunk::global_to_local(adj_voxel_global_pos);
                let local_adj_voxel_pos = Chunk::global_to_local_pos(
                    adj_chunk_pos,
                    adj_voxel_global_pos
                );
                let voxel_coord_idx = USize3::from(local_adj_voxel_pos);
                let partition_idx = ChunkArray::volume_index_to_linear(
                    USize3::all(2),
                    voxel_coord_idx / (Chunk::SIZE / 2),
                );

                let Some(idx) = ChunkArray::pos_to_idx(self.sizes, adj_chunk_pos) else { continue };
                result.insert((idx, partition_idx));
            }
        }

        result
    }
}



pub type ChunkRef = Arc<Chunk>;
pub type ChunkAdj = Sides<Option<Arc<Chunk>>>;



pub type ReadingHandle = JoinHandle<io::Result<(USize3, Vec<Chunk>)>>;
pub type SavingHandle = JoinHandle<io::Result<()>>;



/// Unifies [`ChunkArray`] task interface.
#[derive(Debug, Default)]
pub struct ChunkArrayTasks {
    pub full: HashMap<Int3, FullTask>,
    pub low: HashMap<(Int3, Lod), LowTask>,
    pub voxels_gen: HashMap<Int3, GenTask>,
    pub partition: HashMap<Int3, PartitionTask>,

    pub reading: Nullable<ReadingHandle>,
    pub saving: Nullable<SavingHandle>,
}
assert_impl_all!(ChunkArrayTasks: Send, Sync);

impl ChunkArrayTasks {
    /// Stops all mesh generation tasks that differst with `useful_lod` by more than `2`.
    pub fn stop_all_useless(&mut self, useful_lod: Lod, cur_pos: Int3) {
        // for lod in Chunk::get_possible_lods() {
        //     if 2 < lod.abs_diff(useful_lod) {
        //         self.stop_mesh(cur_pos, lod);
        //     }
        // }
    }

    /// Stops all mesh generation tasks with level of details of `lod`.
    pub fn stop_mesh(&mut self, pos: Int3, lod: Lod) {
        match lod {
            0 => drop(self.full.remove(&pos)),
            _ => drop(self.low.remove(&(pos, lod))),
        }
    }

    /// Stops all mesh generation tasks.
    pub fn stop_all_meshes(&mut self, pos: Int3) {
        // let vals_to_be_dropped = Chunk::get_possible_lods()
        //     .into_iter()
        //     .cartesian_product(Range3d::adj_iter(pos).chain([pos]));
        
        // for (lod, pos) in vals_to_be_dropped {
        //     self.stop_mesh(pos, lod);
        // }
    }

    /// Tests for voxel generation task running.
    pub fn is_gen_running(&self, pos: Int3) -> bool {
        self.voxels_gen.contains_key(&pos)
    }

    /// Checks if generate mesh task id running.
    pub fn is_mesh_running(&self, pos: Int3, lod: Lod) -> bool {
        match lod {
            0 => self.full.contains_key(&pos),
            _ => self.low.contains_key(&(pos, lod)),
        }
    }

    /// Starts new voxel generation task.
    pub fn start_gen(&mut self, pos: Int3, sizes: USize3) {
        let prev_value = self.voxels_gen.insert(pos, Task::spawn(async move {
            Chunk::generate_voxels(pos, sizes)
        }));

        assert!(prev_value.is_none(), "threre should be only one task");
    }

    /// Starts new mesh generation task.
    pub async fn start_mesh(&mut self, chunk: &ChunkRef, adj: ChunkAdj, lod: Lod) {
        return;

        let chunk = ChunkRef::clone(chunk);
        
        let chunk_pos = chunk.pos.load(Relaxed);
        if lod == 0 && self.full.contains_key(&chunk_pos)
            || lod != 0 && self.low.contains_key(&(chunk_pos, lod))
            || !chunk.is_generated()
            || !ChunkArray::is_adj_generated(&adj).await
        { return }
    
        let adj = todo!();

        let prev_is_none = match lod {
            0 => self.full.insert(chunk_pos, Task::spawn(async move {
                // chunk.make_full_mesh(adj);
                todo!()
            })).is_none(),

            _ => self.low.insert((chunk_pos, lod), Task::spawn(async move {
                // chunk.make_low_mesh(adj, lod);
                todo!()
            })).is_none(),
        };

        assert!(prev_is_none, "there should be only one task");
    }

    /// Starts new mesh partitioning task.
    pub fn start_partitioning(&mut self, chunk: &ChunkRef, adj: ChunkAdj) {
        return;

        let chunk = ChunkRef::clone(chunk);

        let prev_value = self.partition.insert(chunk.pos.load(Relaxed), Task::spawn(async move {
            // chunk.make_partitial_meshes(todo!("adj"));
            todo!()
        }));
        assert!(prev_value.is_none(), "there should be only one task");
    }

    /// Tries to finish chunk generation task.
    /// Returns [`Some`] with [chunk][Chunk] if it's ready.
    pub async fn try_finish_gen(&mut self, pos: Int3) -> Option<Chunk> {
        let voxel_ids = self.voxels_gen
            .get_mut(&pos)?
            .try_take_result().await?;

        self.voxels_gen.remove(&pos);

        Some(Chunk::from_voxels(voxel_ids, pos))
    }

    /// Tries to finish mesh generation task and then applies it to `mesh`.
    /// Returns [`true`] if success.
    pub async fn try_finish_mesh(
        &mut self, pos: Int3, lod: Lod, mesh: &mut Mesh, device: &Device,
    ) -> bool {
        match lod {
            0 => self.try_finish_full(pos, mesh, device).await,
            _ => self.try_finish_low(pos, lod, mesh, device).await,
        }
    }

    /// Tries to finish full resolution mesh generation task and then applies it to `mesh`.
    /// Returns [`true`] if success.
    pub async fn try_finish_full(
        &mut self, pos: Int3, mesh: &mut Mesh, device: &Device,
    ) -> bool {
        let Some(task) = self.full.get_mut(&pos) else { return false };
        let Some(vertices) = task.try_take_result().await else { return false };

        // mesh.upload_full_mesh(device, &vertices);
        _ = self.full.remove(&pos)
            .expect("there should be a task due to check before");

        true
    }

    /// Tries to finish low resolution mesh generation task and then applies it to `mesh`.
    /// Returns [`true`] if success.
    pub async fn try_finish_low(
        &mut self, pos: Int3, lod: Lod, mesh: &mut Mesh, device: &Device,
    ) -> bool {
        let Some(task) = self.low.get_mut(&(pos, lod)) else { return false };
        let Some(vertices) = task.try_take_result().await else { return false };

        // mesh.upload_low_mesh(device, &vertices, lod);
        let _ = self.low.remove(&(pos, lod))
            .expect("there should be a task due to check before");

        true
    }

    /// Tests for available slot for new mesh generation tasks.
    pub fn can_start_mesh(&self) -> bool {
        self.saving.is_null()
            && self.reading.is_null()
            && self.low.len() + self.full.len() <= cfg::terrain::MAX_TASKS
    }

    /// Stops all tasks.
    pub fn stop_all(&mut self) {
        self.full.clear();
        self.low.clear();
        self.voxels_gen.clear();
        self.partition.clear();
    }

    /// Tests for any task running.
    pub fn any_running(&self) -> bool {
        !self.low.is_empty() ||
        !self.full.is_empty() ||
        !self.voxels_gen.is_empty() ||
        !self.partition.is_empty()
    }
}
