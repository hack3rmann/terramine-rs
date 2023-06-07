use {
    crate::{
        prelude::*,
        terrain::{
            chunk::{
                prelude::*, EditError, Sides, Id,
                tasks::{FullTask, LowTask, Task, GenTask, PartitionTask},
                mesh::ChunkMesh,
                commands::Command,
            },
            voxel::{self, Voxel, voxel_data::data::*},
        },
        saves::Save,
        graphics::{*, camera_resource::Camera},
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
    const MAX_TRACE_STEPS: usize = 1024;

    /// Generates new chunks.
    /// 
    /// # Panic
    /// 
    /// Panics if `sizes` is not valid. See `ChunkArray::validate_sizes()`.
    pub async fn new(world: &mut World, sizes: USize3) -> Result<Self, UserFacingError> {
        Self::validate_sizes(sizes)?;
        let (start_pos, end_pos) = Self::pos_bounds(sizes);

        let chunks = Range3d::new(start_pos..end_pos)
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
    pub async fn from_chunks(world: &mut World, sizes: USize3, chunks: Vec<ChunkRef>) -> Result<Self, UserFacingError> {
        Self::validate_sizes(sizes)?;
        let volume = Self::volume(sizes);

        ensure!(
            chunks.len() == volume,
            UserFacingError::new("sizes are not match with data").help(format!(
                "passed in chunk `Vec` should have same size as
                passed in sizes, but sizes: {sizes}, len: {len}",
                len = chunks.len(),
            ))
        );

        let self_entity = world.spawn_empty();

        {
            let graphics = world.resource::<&Graphics>().unwrap();
            
            let binds = ChunkBinds::new(&graphics.context.device, &graphics.context.queue)
                .await
                .expect("failed to make binds for chunk array");

            let layout = ChunkPipelineLayout::new(
                &graphics.context.device,
                &binds.full.layouts().collect_vec(),
                &binds.low.layouts().collect_vec(),
            );

            let pipeline = ChunkRenderPipeline::new(&graphics.context.device, &layout).await;

            drop(graphics);

            world.insert(self_entity, (binds, pipeline, layout)).unwrap();
        };

        let chunk_entities = world.spawn_batch(
            chunks.into_iter().map(|chunk| {
                (chunk, ChunkMesh::default())
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

    /// Constructs [`ChunkArray`] with empty chunks.
    /// 
    /// # Panic
    /// 
    /// Panics if `sizes` is not valid. See `ChunkArray::validate_sizes()`.
    pub async fn new_empty_chunks(world: &mut World, sizes: USize3) -> Result<Self, UserFacingError> {
        Self::validate_sizes(sizes)?;
        let (start_pos, end_pos) = Self::pos_bounds(sizes);

        let chunks = Range3d::new(start_pos..end_pos)
            .map(Chunk::new_empty)
            .map(ChunkRef::new)
            .collect();

        Self::from_chunks(world, sizes, chunks).await
    }

    /// Computes start and end poses from chunk array sizes.
    pub fn pos_bounds(sizes: USize3) -> (Int3, Int3) {
        (
            Self::coord_idx_to_pos(sizes, USize3::ZERO),
            Self::coord_idx_to_pos(sizes, sizes),
        )
    }

    /// Checks that sizes is valid.
    /// 
    /// # Error
    /// 
    /// Returns `Err` if `sizes.x * sizes.y * sizes.z` > `MAX_CHUNKS`.
    pub fn validate_sizes(sizes: USize3) -> Result<(), UserFacingError> {
        let volume = Self::volume(sizes);
        (volume <= cfg::terrain::MAX_CHUNKS)
            .then_some(())
            .ok_or_else(|| UserFacingError::new("too many chunks")
                .reason(format!("cannot allocate too many chunks: {volume}"))
            )
    }

    /// Gives empty [`ChunkArray`].
    pub async fn new_empty(world: &mut World) -> Result<Self, UserFacingError> {
        Self::new_empty_chunks(world, USize3::ZERO).await
    }

    /// Clears all [chunk array][ChunkArray] stuff out from the `world`.
    pub fn clean_world(&mut self, world: &mut World) {
        world.despawn(self.array_entity).unwrap();
        self.array_entity = Entity::DANGLING;

        for chunk_entity in mem::take(&mut self.chunk_entities) {
            world.despawn(chunk_entity).unwrap();
        }
    }

    pub async fn save_to_file(
        sizes: USize3, chunks: Vec<ChunkRef>, save_name: impl Into<String>, save_path: &str,
    ) -> io::Result<()> {
        let save_name = save_name.into();

        let _work_guard = logger::scope("chunk-array", format!("saving to {save_name} in {save_path}"));

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
        let _work_guard = logger::scope("chunk-array", format!("reading chunks from {save_name} in {save_path}"));

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
        use { bit_vec::BitVec, huffman_compress as hc };

        let pos = chunk.pos.load(Relaxed);

        match chunk.info.load(Relaxed).fill_type {
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

                let (book, _) = hc::CodeBuilder::from_iter(
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
        use { bit_vec::BitVec, huffman_compress as hc };

        let mut reader = ByteReader::new(bytes);
        let fill_type: FillType = reader.read()
            .expect("failed to reinterpret bytes");

        match fill_type {
            FillType::Unspecified => {
                let freqs: HashMap<Id, usize> = reader.read()
                    .expect("failed to read frequencies map from bytes");

                let pos: Int3 = reader.read()
                    .expect("failed to read chunk pos from bytes");

                let bits: BitVec = reader.read()
                    .expect("failed to read `BitVec` from bytes");

                let (_, tree) = hc::CodeBuilder::from_iter(freqs).finish();
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
        let chunk_pos = Chunk::local_pos(pos);
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
    pub fn set_voxel(&mut self, world: &World, pos: Int3, new_id: Id) -> Result<Id, EditError> {
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
    pub fn fill_voxels(&mut self, world: &World, pos_from: Int3, pos_to: Int3, new_id: Id) -> Result<bool, EditError> {
        let chunk_pos_from = Chunk::local_pos(pos_from);
        let chunk_pos_to   = Chunk::local_pos(pos_to + Int3::from(Chunk::SIZES) - Int3::ONE);

        Self::pos_to_idx(self.sizes, chunk_pos_from)
            .ok_or(EditError::PosIdConversion(chunk_pos_from))?;

        Self::pos_to_idx(self.sizes, chunk_pos_to - Int3::ONE)
            .ok_or(EditError::PosIdConversion(chunk_pos_to - Int3::ONE))?;

        let mut is_changed = false;

        for chunk_pos in Range3d::new(chunk_pos_from..chunk_pos_to) {
            let idx = Self::pos_to_idx(self.sizes, chunk_pos)
                .expect("chunk_pos already valid");

            let min_voxel_pos = Chunk::global_pos(chunk_pos);
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
                    let mut mesh = self.chunk_component::<&mut ChunkMesh>(world, idx);
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
            let mut mesh = self.chunk_component::<&mut ChunkMesh>(world, idx);
            *mesh = default();
        }
    }

    fn count_voxel_frequencies(voxel_ids: impl IntoIterator<Item = Id>) -> HashMap<Id, usize> {
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
        &mut self, world: &mut World, sizes: USize3, chunks: Vec<Chunk>,
    ) -> Result<(), UserFacingError> {
        ensure_eq!(
            Self::volume(sizes),
            chunks.len(),
            UserFacingError::new("chunk-array should have same len as sizes")
        );

        let chunks = chunks.into_iter()
            .map(Arc::new)
            .collect();

        let new_chunks = ChunkArray::from_chunks(world, sizes, chunks).await?;
        self.tasks.stop_all();
        let _ = mem::replace(self, new_chunks);

        Ok(())
    }

    /// Gives chunk count.
    pub fn volume(arr_sizes: USize3) -> usize {
        arr_sizes.x * arr_sizes.y * arr_sizes.z
    }

    pub fn voxel_pos_to_coord_idx(voxel_pos: Int3, chunk_array_sizes: USize3) -> Option<USize3> {
        let chunk_pos = Chunk::local_pos(voxel_pos);
        let local_voxel_pos = Chunk::global_to_local_pos(chunk_pos, voxel_pos);

        let chunk_coord_idx = Self::pos_to_coord_idx(chunk_array_sizes, chunk_pos)?;
        let voxel_offset_by_chunk: USize3 = Chunk::global_pos(chunk_coord_idx.into()).into();

        Some(voxel_offset_by_chunk + USize3::from(local_voxel_pos))
    }

    /// Convertes 3d index into chunk pos.
    pub fn coord_idx_to_pos(sizes: USize3, coord_idx: USize3) -> Int3 {
        Int3::from(coord_idx) - Int3::from(sizes) / 2
    }

    /// Convertes chunk pos to 3d index.
    pub fn pos_to_coord_idx(sizes: USize3, pos: Int3) -> Option<USize3> {
        let sizes = Int3::from(sizes);
        let shifted = pos + sizes / 2;

        (
            0 <= shifted.x && shifted.x < sizes.x &&
            0 <= shifted.y && shifted.y < sizes.y &&
            0 <= shifted.z && shifted.z < sizes.z
        ).then_some(shifted.into())
    }

    /// Convertes 3d index to an array index.
    pub fn coord_idx_to_idx(sizes: USize3, coord_idx: USize3) -> usize {
        sdex::get_index(&coord_idx.as_array(), &sizes.as_array())
    }

    /// Convertes [chunk][Chunk] pos to an array index.
    pub fn pos_to_idx(sizes: USize3, pos: Int3) -> Option<usize> {
        let coord_idx = Self::pos_to_coord_idx(sizes, pos)?;
        Some(Self::coord_idx_to_idx(sizes, coord_idx))
    }

    /// Convertes array index to 3d index.
    pub fn idx_to_coord_idx(idx: usize, sizes: USize3) -> USize3 {
        iterator::idx_to_coord_idx(idx, sizes)
    }

    /// Converts array index to chunk pos.
    pub fn idx_to_pos(idx: usize, sizes: USize3) -> Int3 {
        let coord_idx = Self::idx_to_coord_idx(idx, sizes);
        Self::coord_idx_to_pos(sizes, coord_idx)
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
        Range3d::new(start..end)
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
            let pos = Chunk::global_pos(chunk.pos.load(Relaxed));
            let dot = vec3::sqr(cam_pos - pos.into());
            
            NotNan::new(dot)
                .log_error("chunk-array", "square chunk distance is NaN")
        });

        result
    }

    /// TODO: missing docs
    pub async fn control_meshes(
        &mut self, world: &World, device: &Device, cam: &mut Camera,
    ) -> Result<(), ChunkRenderError> {
        #![allow(clippy::await_holding_refcell_ref)]

        let sizes = self.sizes;
        ensure_or!(sizes != USize3::ZERO, return Ok(()));

        self.try_finish_all_tasks(world, device).await;

        for idx in self.get_indices_sorted(world, cam.pos) {
            let mut chunk = self.chunk_by_idx(world, idx);
            let chunk_pos = chunk.pos.load(Relaxed);
            
            let chunk_adj = self.get_adj_chunks(world, chunk_pos);
            let mut mesh = self.chunk_component::<&mut ChunkMesh>(world, idx);

            let lod = Self::desired_lod_at(chunk_pos, cam.pos, self.lod_threashold);


            if !chunk.is_generated() {
                if self.tasks.is_gen_running(chunk_pos) {
                    if let Some(new_chunk) = self.tasks.try_finish_gen(chunk_pos).await {
                        self.tasks.stop_all_meshes(chunk_pos);

                        // * Safety:
                        // * Safe, because there's no chunk readers due to tasks drop above
                        unsafe {
                            let _ = mem::replace(Arc::get_mut_unchecked(&mut chunk), new_chunk);
                        }
                    }
                } else if self.tasks.can_start_mesh() {
                    self.tasks.start_gen(chunk_pos, sizes);
                    continue;
                } else {
                    continue;
                }
            }

            const CHUNK_MESH_PARTITION_DIST: f32 = 128.0;

            let chunk_is_close_to_be_partitioned = vec3::len(
                vec3::from(Chunk::global_pos(chunk_pos))
                - cam.pos + vec3::from(Chunk::SIZES / 2)
            ) <= CHUNK_MESH_PARTITION_DIST;

            if chunk_is_close_to_be_partitioned &&
               !self.tasks.partition.contains_key(&chunk_pos) &&
               !mesh.is_partial()
            {
                self.tasks.start_partitioning(&chunk, chunk_adj.clone());
            }

            let chunnk_can_be_connected =
                lod == 0 &&
                mesh.is_partial() &&
                !chunk_is_close_to_be_partitioned;

            if chunnk_can_be_connected {
                mesh.connect_partitions(device);
            }

            let can_set_new_lod =
                mesh.get_available_lods().contains(&lod) ||
                self.tasks.is_mesh_running(chunk_pos, lod) &&
                self.tasks.try_finish_mesh(chunk_pos, lod, &mut mesh, device).await;

            if can_set_new_lod {
                chunk.set_active_lod(&mesh, lod);
            } else if self.tasks.can_start_mesh() {
                self.tasks.start_mesh(&chunk, chunk_adj.clone(), lod).await;
            }

            self.tasks.stop_all_useless(lod, chunk_pos);

            if !chunk.can_render_active_lod(&mesh) {
                chunk.try_set_best_fit_lod(&mesh, lod);
            }

            // FIXME: make cam vis-check for light.
            if chunk.can_render_active_lod(&mesh) && chunk.is_visible_by_camera(cam) {
                mesh.active_lod = chunk.info.load(Relaxed).active_lod;
                mesh.enable();
            }
        }

        Ok(())
    }

    pub async fn try_finish_all_full_tasks(&mut self, world: &World, device: &Device) {
        let iter = self.tasks.full.iter_mut()
            .map(|(&pos, task)| (pos, task));

        for (pos, mesh) in Task::try_take_results(iter).await {
            self.tasks.full.remove(&pos);

            let idx = ChunkArray::pos_to_idx(self.sizes, pos)
                .expect("pos should be valid");

            let mut chunk_mesh = self.chunk_component::<&mut ChunkMesh>(world, idx);
            chunk_mesh.upload_full_mesh(device, &mesh);
        }
    }

    pub async fn try_finish_all_low_tasks(&mut self, world: &World, device: &Device) {
        let iter = self.tasks.low.iter_mut()
            .map(|(&idx, task)| (idx, task));

        for ((pos, lod), mesh) in Task::try_take_results(iter).await {
            self.tasks.low.remove(&(pos, lod));

            let idx = ChunkArray::pos_to_idx(self.sizes, pos)
                .expect("pos should be valid");

            let mut chunk_mesh = self.chunk_component::<&mut ChunkMesh>(world, idx);
            chunk_mesh.upload_low_mesh(device, &mesh, lod);
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

            let mut chunk_mesh = self.chunk_component::<&mut ChunkMesh>(world, idx);
            chunk_mesh.upload_partial_meshes(device, &partitions);
        }
    }

    pub async fn try_finish_all_tasks(&mut self, world: &World, device: &Device) {
        self.try_finish_all_full_tasks(world, device).await;
        self.try_finish_all_low_tasks(world, device).await;
        self.try_finish_all_partition_tasks(world, device).await;
        self.try_finish_all_gen_tasks(world).await;
    }

    pub fn render_pass(&self, world: &World, encoder: &mut CommandEncoder, target_view: TextureView) {
        let pipeline = world.get::<&RenderPipeline>(self.array_entity).unwrap();
        let binds = world.get::<&Binds>(self.array_entity).unwrap();

        let mut query = world.query::<&ChunkMesh>();

        let mut pass = RenderPass::new(encoder, "chunk_array_pass", [&target_view]);

        binds.bind(&mut pass, 0);

        for (_entity, mesh) in query.iter() {
            mesh.render(&pipeline, &mut pass)
                .expect("failed to render a chunk mesh");
        }
    }

    /// Checks that [chunk][Chunk] [adjacent][ChunkAdj] are generated.
    pub async fn is_adj_generated(adj: &ChunkAdj) -> bool {
        adj.inner.iter()
            .filter_map(Option::as_deref)
            .all(Chunk::is_generated)
    }

    pub async fn spawn_control_window(&mut self, world: &mut World, ui: &imgui::Ui) {
        use crate::app::utils::graphics::ui::imgui_ext::make_window;

        make_window(ui, "Chunk array")
            .always_auto_resize(true)
            .build(|| {
                ui.text(format!(
                    "{n} chunk generation tasks.",
                    n = self.tasks.voxels_gen.len(),
                ));

                ui.text(format!(
                    "{n} mesh generation tasks.",
                    n = self.tasks.low.len() + self.tasks.full.len(),
                ));

                ui.text(format!(
                    "{n} partition generation tasks.",
                    n = self.tasks.partition.len(),
                ));

                ui.slider(
                    "Chunks lod threashold",
                    0.01, 20.0,
                    &mut self.lod_threashold,
                );

                ui.separator();

                ui.text("Generate new");

                let mut sizes = GENERATOR_SIZES.lock();

                ui.input_scalar_n("Sizes", &mut *sizes).build();

                if ui.button("Generate") {
                    self.tasks.stop_all();

                    let chunks = tokio::task::block_in_place(|| RUNTIME.block_on(
                        Self::new_empty_chunks(world, USize3::from(*sizes))
                    ));

                    match chunks {
                        Ok(new_chunks) => {
                            let _ = mem::replace(self, new_chunks);
                        },
                        Err(err) => logger::log!(Error, from = "chunk-array", "{err}")
                    }
                }
            });
    }

    pub async fn proccess_command(
        &mut self, world: &World,
        command: Command, change_tracker: &mut ChangeTracker,
    ) {
        match command {
            Command::SetVoxel { pos, new_id } => {
                let old_id = self.set_voxel(world, pos, new_id)
                    .log_error("chunk-array", "failed to set voxel");

                if old_id != new_id {
                    change_tracker.track_voxel(pos);
                }
            },

            Command::FillVoxels { pos_from, pos_to, new_id } => {
                let _is_changed = self.fill_voxels(world, pos_from, pos_to, new_id)
                    .log_error("chunk-arrat", "failed to fill voxels");
            }

            Command::DropAllMeshes => self.drop_all_meshes(world),
        }
    }

    pub async fn process_commands(&mut self, world: &World, device: &Device) {
        #![allow(clippy::await_holding_lock)]

        use crate::terrain::chunk::commands::*;

        let mut change_tracker = ChangeTracker::new(self.sizes);
        
        while let Ok(command) = COMMAND_CHANNEL.lock().receiver.try_recv() {
            self.proccess_command(world, command, &mut change_tracker).await;
        }

        let idxs_to_reload = change_tracker.idxs_to_reload_partitioning();
        let n_changed = idxs_to_reload.len();
        for (idx, partition_idx) in idxs_to_reload {
            self.reload_chunk_partitioning(device, world, idx, partition_idx).await;
        }

        if n_changed != 0 {
            logger::log!(Info, from = "chunk-array", "{n_changed} chunks were updated!");
        }
    }

    pub async fn reload_chunk(&self, world: &World, device: &Device, idx: usize) {
        let chunk_pos = Self::idx_to_pos(idx, self.sizes);
        let adj = self.get_adj_chunks(world, chunk_pos);

        if let Some(chunk) = self.get_chunk_by_idx(world, idx) {
            let mut mesh = self.chunk_component::<&mut ChunkMesh>(world, idx);
            chunk.generate_mesh(&mut mesh, 0, adj, device);
        }
    }

    pub async fn reload_chunk_partitioning(
        &self, device: &Device, world: &World, chunk_idx: usize, partition_idx: usize,
    ) {
        let chunk_pos = Self::idx_to_pos(chunk_idx, self.sizes);
        let adj = self.get_adj_chunks(world, chunk_pos);

        if let Some(chunk) = self.get_chunk_by_idx(world, chunk_idx) {
            let mut chunk_mesh = self.chunk_component::<&mut ChunkMesh>(world, chunk_idx);

            if chunk_mesh.is_partial() {
                let mesh = chunk.make_partition(&adj, partition_idx);
                chunk_mesh.upload_partition(device, &mesh, partition_idx);
            } else {
                chunk.partition_mesh(&mut chunk_mesh, adj, device);
            }
        }
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

    pub async fn proccess_camera_input(&mut self, world: &World, cam: &Camera) {
        use super::commands::command;

        let first_voxel = self.trace_ray(world, Line::new(cam.pos, cam.front), Self::MAX_TRACE_STEPS)
            .find(|voxel| !voxel.is_air());

        if let Some(voxel) = first_voxel && mouse::just_left_pressed() && cam.captures_mouse
        {
            command(Command::SetVoxel { pos: voxel.pos, new_id: AIR_VOXEL_DATA.id })
        }
    }

    pub async fn update(&mut self, world: &mut World, device: &Device, cam: &Camera) -> Result<(), UpdateError> {
        self.proccess_camera_input(world, cam).await;
        self.process_commands(world, device).await;

        if keyboard::just_pressed_combo([Key::LControl, Key::S]) {
            let chunks = self.chunks(world).collect_vec();
            let handle = tokio::spawn(
                ChunkArray::save_to_file(self.sizes, chunks, "world", "world")
            );
            self.tasks.saving = Nullable::new(handle);
        }

        if !self.tasks.saving.is_null() && self.tasks.saving.is_finished() {
            let handle = self.tasks.saving.take();
            handle.await??;
        }

        if keyboard::just_pressed_combo([Key::LControl, Key::O]) {
            self.tasks.reading = Nullable::new(
                tokio::spawn(ChunkArray::read_from_file("world", "world"))
            );
        }

        if !self.tasks.reading.is_null() && self.tasks.reading.is_finished() {
            let handle = self.tasks.reading.take();
            let (sizes, arr) = handle.await??;
            self.apply_new(world, sizes, arr).await?;
        }

        Ok(())
    }
}



macros::sum_errors! {
    pub enum UpdateError { Join => JoinError, Save => io::Error, Other => UserFacingError }
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
            let chunk_pos = Chunk::local_pos(voxel_pos);
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

            let chunk_pos = Chunk::local_pos(voxel_pos);
            let local_pos = Chunk::global_to_local_pos(chunk_pos, voxel_pos);
            let voxel_coord_idx = USize3::from(local_pos);

            let partition_idx = ChunkArray::coord_idx_to_idx(
                USize3::all(2),
                voxel_coord_idx / (Chunk::SIZES / 2),
            );

            let Some(chunk_idx) = ChunkArray::pos_to_idx(self.sizes, chunk_pos) else { continue };

            result.insert((chunk_idx, partition_idx));

            let local_rem = local_pos.rem_euclid(chunk_sizes / 2);
            for offset in iterator::offsets_from_border(local_rem, Int3::ZERO .. chunk_sizes / 2) {
                let adj_voxel_global_pos = voxel_pos + offset;
                let adj_chunk_pos = Chunk::local_pos(adj_voxel_global_pos);
                let local_adj_voxel_pos = Chunk::global_to_local_pos(
                    adj_chunk_pos,
                    adj_voxel_global_pos
                );
                let voxel_coord_idx = USize3::from(local_adj_voxel_pos);
                let partition_idx = ChunkArray::coord_idx_to_idx(
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
        for lod in Chunk::get_possible_lods() {
            if 2 < lod.abs_diff(useful_lod) {
                self.stop_mesh(cur_pos, lod);
            }
        }
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
        let vals_to_be_dropped = Chunk::get_possible_lods()
            .into_iter()
            .cartesian_product(Range3d::adj_iter(pos).chain([pos]));
        
        for (lod, pos) in vals_to_be_dropped {
            self.stop_mesh(pos, lod);
        }
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
        let chunk = ChunkRef::clone(chunk);

        let chunk_pos = chunk.pos.load(Relaxed);
        if lod == 0 && self.full.contains_key(&chunk_pos)
        || lod != 0 && self.low.contains_key(&(chunk_pos, lod))
        || !chunk.is_generated()
        || !ChunkArray::is_adj_generated(&adj).await
        { return }

        let prev_is_none = match lod {
            0 => self.full.insert(chunk_pos, Task::spawn(async move {
                chunk.make_full_mesh(adj)
            })).is_none(),

            _ => self.low.insert((chunk_pos, lod), Task::spawn(async move {
                chunk.make_low_mesh(adj, lod)
            })).is_none(),
        };

        assert!(prev_is_none, "there should be only one task");
    }

    /// Starts new mesh partitioning task.
    pub fn start_partitioning(&mut self, chunk: &ChunkRef, adj: ChunkAdj) {
        let chunk = ChunkRef::clone(chunk);

        let prev_value = self.partition.insert(chunk.pos.load(Relaxed), Task::spawn(async move {
            chunk.make_partitial_meshes(adj)
        }));
        assert!(prev_value.is_none(), "there should be only one task");
    }

    /// Tries to finish chunk generation task.
    /// Returns [`Some`] with [chunk][Chunk] if it's ready.
    pub async fn try_finish_gen(&mut self, pos: Int3) -> Option<Chunk> {
        if let Some(task) = self.voxels_gen.get_mut(&pos)
            && let Some(voxel_ids) = task.try_take_result().await
        {
            self.voxels_gen.remove(&pos);
            return Some(Chunk::from_voxels(voxel_ids, pos))
        }

        None
    }

    /// Tries to finish mesh generation task and then applies it to `mesh`.
    /// Returns [`true`] if success.
    pub async fn try_finish_mesh(
        &mut self, pos: Int3, lod: Lod, mesh: &mut ChunkMesh, device: &Device,
    ) -> bool {
        match lod {
            0 => self.try_finish_full(pos, mesh, device).await,
            _ => self.try_finish_low(pos, lod, mesh, device).await,
        }
    }

    /// Tries to finish full resolution mesh generation task and then applies it to `mesh`.
    /// Returns [`true`] if success.
    pub async fn try_finish_full(
        &mut self, pos: Int3, mesh: &mut ChunkMesh, device: &Device,
    ) -> bool {
        let Some(task) = self.full.get_mut(&pos) else { return false };
        let Some(vertices) = task.try_take_result().await else { return false };

        mesh.upload_full_mesh(device, &vertices);
        let _ = self.full.remove(&pos)
            .expect("there should be a task due to check before");

        true
    }

    /// Tries to finish low resolution mesh generation task and then applies it to `mesh`.
    /// Returns [`true`] if success.
    pub async fn try_finish_low(
        &mut self, pos: Int3, lod: Lod, mesh: &mut ChunkMesh, device: &Device,
    ) -> bool {
        let Some(task) = self.low.get_mut(&(pos, lod)) else { return false };
        let Some(vertices) = task.try_take_result().await else { return false };

        mesh.upload_low_mesh(device, &vertices, lod);
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
        drop(mem::take(&mut self.full));
        drop(mem::take(&mut self.low));
        drop(mem::take(&mut self.voxels_gen));
        drop(mem::take(&mut self.partition));
    }

    /// Tests for any task running.
    pub fn any_running(&self) -> bool {
        !self.low.is_empty() ||
        !self.full.is_empty() ||
        !self.voxels_gen.is_empty() ||
        !self.partition.is_empty()
    }
}



#[derive(Debug, Clone)]
pub struct ChunkBinds {
    pub full: Binds,
    pub low: Binds,
}

impl ChunkBinds {
    pub async fn make_full(device: &Device, queue: &Queue) -> Result<Binds, LoadImageError> {
        use crate::graphics::*;

        let dir = Path::new(cfg::texture::DIRECTORY);

        let (albedo_image, normal_image) = tokio::try_join!(
            Image::from_file(dir.join("texture_atlas.png")),
            Image::from_file(dir.join("normal_atlas.png")),
        )?;

        let albedo = GpuImage::new(
            GpuImageDescriptor {
                device,
                queue,
                image: &albedo_image,
                label: Some("chunk_array_albedo_image_atlas".into()),
            },
        );

        let normal = GpuImage::new(
            GpuImageDescriptor {
                device,
                queue,
                image: &normal_image,
                label: Some("chunk_array_normal_image_atlas".into()),
            },
        );

        let image_layout = GpuImage::bind_group_layout(device);

        let albedo_bind = albedo.as_bind_group(device, &image_layout);
        let normal_bind = normal.as_bind_group(device, &image_layout);

        Ok(Binds::from_iter([
            (albedo_bind, Some(image_layout.clone())),
            (normal_bind, Some(image_layout)),
        ]))
    }

    pub async fn new(device: &Device, queue: &Queue) -> Result<Self, LoadImageError> {
        Ok(Self {
            full: Self::make_full(device, queue).await?,
            low: default(),
        })
    }
}



#[derive(Debug)]
pub struct ChunkPipelineLayout {
    pub full: PipelineLayout,
    pub low: PipelineLayout,
}

impl ChunkPipelineLayout {
    pub fn make_pipeline_layout(device: &Device, bind_group_layouts: &[&wgpu::BindGroupLayout]) -> PipelineLayout {
        use crate::graphics::PipelineLayoutDescriptor as Desc;

        PipelineLayout::new(
            device,
            &Desc {
                label: Some("chunk_array_pipeline"),
                bind_group_layouts,
                push_constant_ranges: &[],
            },
        )
    }

    pub fn new(
        device: &Device,
        full_bind_layouts: &[&wgpu::BindGroupLayout],
        low_bind_layouts: &[&wgpu::BindGroupLayout],
    ) -> Self {
        Self {
            full: Self::make_pipeline_layout(device, full_bind_layouts),
            low: Self::make_pipeline_layout(device, low_bind_layouts),
        }
    }
}



#[derive(Debug, Clone)]
pub struct ChunkRenderPipeline {
    pub full: RenderPipeline,
    pub low: RenderPipeline,
}
assert_impl_all!(ChunkRenderPipeline: Send, Sync);

impl ChunkRenderPipeline {
    const PRIMITIVE_STATE: PrimitiveState = PrimitiveState {
        cull_mode: Some(Face::Back),
        ..const_default()
    };

    pub async fn make_material<V: Vertex>(device: &Device, shader_file_name: &str) -> Arc<dyn Material> {
        let source = ShaderSource::from_file(shader_file_name).await
            .unwrap_or_else(|err| panic!("failed to load shader from file {shader_file_name}: {err}"));

        let shader = Shader::new(device, source, vec![V::BUFFER_LAYOUT]);

        StandartMaterial::from(shader).to_arc()
    }

    pub async fn make_full(device: &Device, layout: &PipelineLayout) -> RenderPipeline {
        use crate::{graphics::RenderPipelineDescriptor as Desc, terrain::chunk::mesh::FullVertex};

        RenderPipeline::new(Desc {
            device,
            layout,
            material: Self::make_material::<FullVertex>(device, "chunks_full.wgsl").await.as_ref(),
            primitive_state: Self::PRIMITIVE_STATE,
            label: Some("chunk_array_full_detail_render_pipeline".into()),
        })
    }

    pub async fn make_low(device: &Device, layout: &PipelineLayout) -> RenderPipeline {
        use crate::{graphics::RenderPipelineDescriptor as Desc, terrain::chunk::mesh::LowVertex};

        RenderPipeline::new(Desc {
            device,
            layout,
            material: Self::make_material::<LowVertex>(device, "chunks_low.wgsl").await.as_ref(),
            primitive_state: Self::PRIMITIVE_STATE,
            label: Some("chunk_array_full_detail_render_pipeline".into()),
        })
    }

    pub async fn new(device: &Device, layout: &ChunkPipelineLayout) -> Self {
        let (full, low) = tokio::join!(
            Self::make_full(device, &layout.full),
            Self::make_low(device, &layout.low),
        );

        Self { full, low }
    }
}
