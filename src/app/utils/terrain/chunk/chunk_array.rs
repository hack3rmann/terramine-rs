use {
    crate::{
        prelude::*,
        terrain::{
            chunk::{
                prelude::*, EditError, Sides, Id,
                tasks::{FullTask, LowTask, Task, GenTask, PartitionTask},
                mesh::ChunkMesh,
            },
            voxel::{self, Voxel, voxel_data::data::*},
        },
        saves::Save,
        graphics::camera_resource::Camera,
    },
    math_linear::math::ray::space_3d::Line,
    std::{io, mem, sync::Mutex},
    glium::{self as gl, backend::Facade},
    tokio::task::{JoinHandle, JoinError},
};

pub static GENERATOR_SIZES: Mutex<[usize; 3]> = Mutex::new(USize3::ZERO.as_array());

#[derive(Clone, Copy, Debug)]
enum ChunkArrSaveType {
    Sizes,
    Array,
}

impl From<ChunkArrSaveType> for u64 {
    fn from(value: ChunkArrSaveType) -> Self { value as u64 }
}

pub type ReadingHandle = JoinHandle<io::Result<(USize3, Vec<(Vec<Atomic<Id>>, FillType)>)>>;

/// Represents 3d array of [`Chunk`]s. Can control their mesh generation, etc.
#[derive(Debug, SmartDefault)]
pub struct ChunkArray {
    pub chunks: Vec<ChunkRef>,
    pub meshes: Vec<MeshRef>,
    pub sizes: USize3,

    pub full_tasks: HashMap<Int3, FullTask>,
    pub low_tasks: HashMap<(Int3, Lod), LowTask>,
    pub voxels_gen_tasks: HashMap<Int3, GenTask>,
    pub partition_tasks: HashMap<Int3, PartitionTask>,

    #[default = 5.8]
    pub lod_threashold: f32,

    pub reading_handle: Option<ReadingHandle>,
    pub saving_handle: Option<JoinHandle<io::Result<()>>>,
}

impl ChunkArray {
    const MAX_TRACE_STEPS: usize = 1024;

    /// Generates new chunks.
    /// # Panic
    /// Panics if `sizes` is not valid. See `ChunkArray::validate_sizes()`.
    pub fn new(sizes: USize3) -> Result<Self, UserFacingError> {
        Self::validate_sizes(sizes)?;
        let (start_pos, end_pos) = Self::pos_bounds(sizes);

        let chunks = SpaceIter::new(start_pos..end_pos)
            .map(move |pos| Chunk::new(pos, sizes))
            .map(Arc::new)
            .collect();

        Self::from_chunks(sizes, chunks)
    }

    /// Constructs [`ChunkArray`] with passed in chunks.
    /// # Panic
    /// Panics if `sizes` is not valid. See `ChunkArray::validate_sizes()`.
    pub fn from_chunks(sizes: USize3, chunks: Vec<Arc<Chunk>>) -> Result<Self, UserFacingError> {
        Self::validate_sizes(sizes)?;
        let volume = Self::volume(sizes);
        if chunks.len() != volume {
            return Err(UserFacingError::new("sizes are not match with data")
                .help(format!(
                    "passed in chunk `Vec` should have same size as
                    passed in sizes, but sizes: {sizes}, len: {len}",
                    len = chunks.len(),
                ))
            )
        }

        let meshes = (0..chunks.len())
            .map(|_| Rc::new(RefCell::new(ChunkMesh::default())))
            .collect();
        
        Ok(Self { chunks, sizes, meshes, ..default() })
    }

    /// Constructs [`ChunkArray`] with empty chunks.
    /// # Panic
    /// Panics if `sizes` is not valid. See `ChunkArray::validate_sizes()`.
    pub fn new_empty_chunks(sizes: USize3) -> Result<Self, UserFacingError> {
        Self::validate_sizes(sizes)?;
        let (start_pos, end_pos) = Self::pos_bounds(sizes);

        let chunks = SpaceIter::new(start_pos..end_pos)
            .map(Chunk::new_empty)
            .map(Arc::new)
            .collect();

        Self::from_chunks(sizes, chunks)
    }

    /// Computes start and end poses from chunk array sizes.
    pub fn pos_bounds(sizes: USize3) -> (Int3, Int3) {
        (
            Self::coord_idx_to_pos(sizes, USize3::ZERO),
            Self::coord_idx_to_pos(sizes, sizes),
        )
    }

    /// Checks that sizes is valid.
    /// # Panic
    /// Panics if `sizes.x * sizes.y * sizes.z` > `MAX_CHUNKS`.
    pub fn validate_sizes(sizes: USize3) -> Result<(), UserFacingError> {
        let volume = Self::volume(sizes);
        match volume <= cfg::terrain::MAX_CHUNKS {
            false => Err(UserFacingError::new("too many chunks")
                .reason(format!("cannot allocate too many chunks: {volume}"))),
            true => Ok(()),
        }
    }

    /// Gives empty [`ChunkArray`].
    pub fn new_empty() -> Self {
        Self::default()
    }

    pub async fn save_to_file(
        sizes: USize3, chunks: Vec<ChunkRef>, save_name: impl Into<String>, save_path: &'static str,
    ) -> io::Result<()> {
        let save_name = save_name.into();

        let _work_guard = logger::work("chunk-array", format!("saving to {save_name} in {save_path}"));

        let is_all_generated = {
            let mut result = true;
            for chunk in chunks.iter() {
                if !chunk.is_generated() {
                    result = false;
                    break
                }
            }
            result
        };

        assert!(is_all_generated, "chunks should be generated to save them to file");

        let volume = Self::volume(sizes);
        assert_eq!(volume, chunks.len(), "chunks should have same length as sizes volume");

        let loading = loading::start_new("Saving chunks");

        Save::builder(save_name.clone())
            .create(save_path).await?
            .write(&sizes, ChunkArrSaveType::Sizes).await
            .pointer_array(volume, ChunkArrSaveType::Array, |i| {
                let chunks = &chunks;
                let loading = &loading;

                async move {
                    loading.refresh(i as f32 / (volume - 1) as f32);
                    Self::chunk_as_bytes(&chunks[i])
                }
            }).await
            .save()
            .await?;

        Ok(())
    }

    pub async fn read_from_file(
        save_name: &str, save_path: &str,
    ) -> io::Result<(USize3, Vec<(Vec<Atomic<Id>>, FillType)>)> {
        let _work_guard = logger::work("chunk-array", format!("reading chunks from {save_name} in {save_path}"));

        let loading = loading::start_new("Reading chunks");

        let mut save = Save::builder(save_name)
            .open(save_path)
            .await?;
        
        let sizes = save.read(ChunkArrSaveType::Sizes).await;

        let chunks = save.read_pointer_array(ChunkArrSaveType::Array, |i, bytes| {
            let loading = &loading;

            async move {
                loading.refresh(i as f32 / (Self::volume(sizes) - 1) as f32);
                Self::array_filltype_from_bytes(&bytes)
            }
        }).await;

        Ok((sizes, chunks))
    }

    /// Reinterprets [chunk][Chunk] as bytes. It uses Huffman's compresstion.
    pub fn chunk_as_bytes(chunk: &Chunk) -> Vec<u8> {
        use { bit_vec::BitVec, huffman_compress as hc };

        match chunk.info.load(Relaxed).fill_type {
            FillType::AllSame(id) =>
                FillType::AllSame(id).as_bytes(),

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

                itertools::chain! {
                    FillType::Unspecified.as_bytes(),
                    freqs.as_bytes(),
                    bits.as_bytes(),
                }.collect()
            }
        }
    }

    /// Reinterprets bytes as [chunk][Chunk] and reads [id][Id] array and [fill type][FillType] from it.
    pub fn array_filltype_from_bytes(bytes: &[u8]) -> (Vec<Atomic<Id>>, FillType) {
        use { bit_vec::BitVec, huffman_compress as hc };

        let mut reader = ByteReader::new(bytes);
        let fill_type: FillType = reader.read()
            .expect("failed to reinterpret bytes");

        match fill_type {
            FillType::Unspecified => {
                let freqs: HashMap<Id, usize> = reader.read()
                    .expect("failed to read frequencies map from bytes");

                let bits: BitVec = reader.read()
                    .expect("failed to read `BitVec` from bytes");

                let (_, tree) = hc::CodeBuilder::from_iter(freqs).finish();
                let voxel_ids: Vec<_> = tree.unbounded_decoder(bits)
                    .map(Atomic::new)
                    .collect();

                let is_id_valid = voxel_ids.iter()
                    .map(|id| id.load(Relaxed))
                    .all(voxel::is_id_valid);

                assert!(is_id_valid, "Voxel ids in voxel array should be valid");
                assert_eq!(voxel_ids.len(), Chunk::VOLUME, "There's should be Chunk::VOLUME voxels");

                (voxel_ids, FillType::Unspecified)
            },

            FillType::AllSame(id) =>
                (vec![], FillType::AllSame(id)),
        }
    }

    /// Sets voxel's id with position `pos` to `new_id` and returns old [`Id`]. If voxel is 
    /// set then this function should drop all its meshes and the neighbor ones.
    /// # Error
    /// Returns [`Err`] if `new_id` is not valid or `pos` is not in this [chunk array][ChunkArray].
    pub fn set_voxel(&mut self, pos: Int3, new_id: Id) -> Result<Id, EditError> {
        let chunk_pos = Chunk::local_pos(pos);
        let chunk_idx = Self::pos_to_idx(self.sizes, chunk_pos)
            .ok_or(EditError::PosIdConversion(pos))?;

        // We know that `chunk_idx` is valid so we can get-by-index.
        let old_id = unsafe {
            Arc::get_mut_unchecked(&mut self.chunks[chunk_idx])
                .set_voxel(pos, new_id)?
        };

        Ok(old_id)
    }

    /// Gives voxel if it is in the [array][ChunkArray].
    pub fn get_voxel(&self, pos: Int3) -> Option<Voxel> {
        let chunk_pos = Chunk::local_pos(pos);
        let chunk_idx = Self::pos_to_idx(self.sizes, chunk_pos)?;

        match self.chunks[chunk_idx].get_voxel_global(pos) {
            ChunkOption::Voxel(voxel) => Some(voxel),
            ChunkOption::OutsideChunk => unreachable!("pos {} is indeed in that chunk", pos),
            ChunkOption::Failed => None,
        }
    }

    /// Fills volume of voxels to same [id][Id] and returnes `is_changed`.
    pub fn fill_voxels(&mut self, pos_from: Int3, pos_to: Int3, new_id: Id) -> Result<bool, EditError> {
        let chunk_pos_from = Chunk::local_pos(pos_from);
        let chunk_pos_to   = Chunk::local_pos(pos_to + Int3::from(Chunk::SIZES) - Int3::ONE);

        Self::pos_to_idx(self.sizes, chunk_pos_from)
            .ok_or(EditError::PosIdConversion(chunk_pos_from))?;

        Self::pos_to_idx(self.sizes, chunk_pos_to - Int3::ONE)
            .ok_or(EditError::PosIdConversion(chunk_pos_to - Int3::ONE))?;

        let mut is_changed = false;

        for chunk_pos in SpaceIter::new(chunk_pos_from..chunk_pos_to) {
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

            let chunk_changed = unsafe {
                Arc::get_mut_unchecked(&mut self.chunks[idx])
                    .fill_voxels(pos_from, pos_to, new_id)?
            };

            if chunk_changed {
                is_changed = true;
                
                for idx in Self::get_adj_chunks_idxs(self.sizes, chunk_pos).as_array().into_iter().flatten() {
                    self.meshes[idx].borrow_mut().drop_all();
                }
            }
        }

        Ok(is_changed)
    }

    /// Drops all meshes from each [chunk][Chunk].
    pub fn drop_all_meshes(&self) {
        for mesh in self.meshes.iter() {
            mesh.borrow_mut().drop_all();
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

    pub fn apply_new(&mut self, sizes: USize3, chunk_arr: Vec<(Vec<Atomic<Id>>, FillType)>) -> Result<(), UserFacingError> {
        if Self::volume(sizes) != chunk_arr.len() {
            return Err(UserFacingError::new("chunk-array should have same len as sizes"));
        }

        let chunks = chunk_arr.into_iter()
            .enumerate()
            .map(|(idx, (voxel_ids, fill_type))| {
                let chunk_pos = Self::idx_to_pos(idx, sizes);
                match fill_type {
                    FillType::Unspecified =>
                        Chunk::from_voxels(voxel_ids, chunk_pos),
                    FillType::AllSame(id) =>
                        Chunk::new_same_filled(chunk_pos, id),
                }
            })
            .map(Arc::new)
            .collect();

        let new_chunks = ChunkArray::from_chunks(sizes, chunks)?;
        self.drop_tasks();
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

        match 0 <= shifted.x && shifted.x < sizes.x &&
              0 <= shifted.y && shifted.y < sizes.y &&
              0 <= shifted.z && shifted.z < sizes.z
        {
            true  => Some(shifted.into()),
            false => None
        }
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

    /// Gives reference to chunk by its position.
    pub fn get_chunk_by_pos(&self, pos: Int3) -> Option<Arc<Chunk>> {
        Self::get_chunk_by_pos_unbounded(&self.chunks, self.sizes, pos)
    }

    fn get_chunk_by_pos_unbounded(chunks: &[Arc<Chunk>], sizes: USize3, pos: Int3) -> Option<Arc<Chunk>> {
        let idx = Self::pos_to_idx(sizes, pos)?;
        Some(Arc::clone(&chunks[idx]))
    }

    /// Gives adjacent chunks references by center chunk position.
    pub fn get_adj_chunks(&self, pos: Int3) -> ChunkAdj {
        Self::get_adj_chunks_unbounded(&self.chunks, self.sizes, pos)
    }

    /// Gives adjacent chunks references by center chunk position.
    fn get_adj_chunks_unbounded(chunks: &[ChunkRef], sizes: USize3, pos: Int3) -> ChunkAdj {
        Self::get_adj_chunks_idxs(sizes, pos)
            .map(|opt| opt.map(|idx| Arc::clone(&chunks[idx])))
    }

    /// Gives '`iterator`' over adjacent to `pos` array indices.
    pub fn get_adj_chunks_idxs(sizes: USize3, pos: Int3) -> Sides<Option<usize>> {
        SpaceIter::adj_iter(pos)
            .map(|pos| Self::pos_to_idx(sizes, pos))
            .collect()
    }

    /// Gives iterator over chunk coordinates.
    pub fn pos_iter(sizes: USize3) -> SpaceIter {
        let (start, end) = Self::pos_bounds(sizes);
        SpaceIter::new(start..end)
    }

    /// Gives iterator over all chunk's adjacents.
    pub fn adj_iter(&self) -> impl ExactSizeIterator<Item = ChunkAdj> + '_ {
        Self::adj_iter_unbounded(&self.chunks, self.sizes)
    }

    /// Gives iterator over all chunk's adjacents.
    fn adj_iter_unbounded(chunks: &[ChunkRef], sizes: USize3) -> impl ExactSizeIterator<Item = ChunkAdj> + '_ {
        Self::pos_iter(sizes)
            .map(move |pos| Self::get_adj_chunks_unbounded(chunks, sizes, pos))
    }

    /// Gives desired [LOD][Lod] value for chunk positioned in `chunk_pos`.
    pub fn desired_lod_at(chunk_pos: Int3, cam_pos: vec3, threashold: f32) -> Lod {
        let chunk_size = Chunk::GLOBAL_SIZE;
        let cam_pos_in_chunks = cam_pos / chunk_size;
        let chunk_pos = vec3::from(chunk_pos);

        let dist = (chunk_pos - cam_pos_in_chunks + vec3::all(0.5)).len();
        Lod::min(
            (dist / threashold).floor() as Lod,
            Chunk::SIZE.ilog2() as Lod,
        )
    }

    /// Gives iterator over desired LOD for each chunk.
    pub fn desired_lod_iter(chunk_array_sizes: USize3, cam_pos: vec3, threashold: f32) -> impl ExactSizeIterator<Item = Lod> {
        Self::pos_iter(chunk_array_sizes)
            .map(move |chunk_pos| Self::desired_lod_at(chunk_pos, cam_pos, threashold))
    }

    /// Gives iterator over all voxels in [`ChunkArray`].
    pub fn voxels(&self) -> impl Iterator<Item = Voxel> + '_ {
        self.chunks.iter()
            .flat_map(|chunk| chunk.voxels())
    }

    /// Gives iterator over mutable chunks and their adjacents.
    pub fn chunks_with_adj(&self) -> impl ExactSizeIterator<Item = (ChunkRef, ChunkAdj)> + '_ {
        Self::chunks_with_adj_unbounded(&self.chunks, self.sizes)
    }

    /// Gives iterator over mutable chunks and their adjacents.
    pub fn chunks_with_adj_unbounded(
        chunks: &[ChunkRef], sizes: USize3,
    ) -> impl ExactSizeIterator<Item = (ChunkRef, ChunkAdj)> + '_ {
        chunks.iter()
            .map(Arc::clone)
            .zip(Self::adj_iter_unbounded(chunks, sizes))
    }

    /// Gives [`Vec`] with [`ChunkRef`]s [`ChunkAdj`]s desired [lod][Lod].
    fn get_targets_sorted(&self, cam_pos: vec3) -> Vec<(ChunkRef, ChunkAdj, MeshRef, Lod)> {
        let mut result: Vec<_> = self.chunks_with_adj()
            .zip(self.meshes.iter().cloned())
            .zip(Self::desired_lod_iter(self.sizes, cam_pos, self.lod_threashold))
            .map(|(((a, b), c), d)| (a, b, c, d))
            .collect();

        result.sort_by_key(|(chunk, _, _, _)| {
            let pos = Chunk::global_pos(chunk.pos.load(Relaxed));
            let dot = vec3::sqr(cam_pos - pos.into());
            
            match NotNan::new(dot) {
                Ok(result) => result,
                Err(err) => {
                    logger::log!(Error, from = "chunk-array", "{err}");
                    NotNan::default()
                }
            }
        });

        result
    }

    /// Renders all [chunk][Chunk]s. If [chunk][Chunk] should have another
    /// [LOD][Lod] then it will start async task that generates desired mesh.
    /// If task is incomplete then it will render active [LOD][Lod]
    /// of concrete [chunk][Chunk]. If it can't then it will do nothing.
    pub async fn render(
        &mut self, target: &mut impl gl::Surface, draw_bundle: &ChunkDrawBundle<'_>,
        uniforms: &impl gl::uniforms::Uniforms, facade: &dyn gl::backend::Facade, cam: &mut Camera,
    ) -> Result<(), ChunkRenderError> {
        #![allow(clippy::await_holding_refcell_ref)]

        let sizes = self.sizes;
        if sizes == USize3::ZERO { return Ok(()) }

        self.try_finish_all_tasks(facade).await;

        let targets = self.get_targets_sorted(cam.pos);

        for (mut chunk, chunk_adj, mesh, lod) in targets {
            let chunk_pos = chunk.pos.load(Relaxed);

            if !chunk.is_generated() {
                if Self::is_voxels_gen_task_running(&self.voxels_gen_tasks, chunk_pos) {
                    if let Some(new_chunk) = Self::try_finish_voxels_gen_task(&mut self.voxels_gen_tasks, chunk_pos).await {
                        Self::drop_reader_tasks(&mut self.full_tasks, &mut self.low_tasks, chunk_pos);

                        // * Safety:
                        // * Safe, because there's no chunk readers due to tasks drop above
                        unsafe {
                            let _ = mem::replace(Arc::get_mut_unchecked(&mut chunk), new_chunk);
                        }
                    }
                }
                
                else if self.can_start_tasks() {
                    Self::start_task_gen_voxels(&mut self.voxels_gen_tasks, chunk_pos, sizes);
                    continue;
                }

                else {
                    continue;
                }
            }

            const CHUNK_MESH_PARTITION_DIST: f32 = 128.0;

            let chunk_is_close_to_be_partitioned = vec3::len(
                vec3::from(Chunk::global_pos(chunk_pos))
                - cam.pos + vec3::from(Chunk::SIZES / 2)
            ) <= CHUNK_MESH_PARTITION_DIST;

            if chunk_is_close_to_be_partitioned &&
               !self.partition_tasks.contains_key(&chunk_pos) &&
               !mesh.borrow().is_partitioned()
            {
                Self::start_task_partitioning(&mut self.partition_tasks, Arc::clone(&chunk), chunk_adj.clone());
            }

            let chunnk_can_be_connected =
                lod == 0 &&
                mesh.borrow().is_partitioned() &&
                !chunk_is_close_to_be_partitioned;

            if chunnk_can_be_connected {
                mesh.borrow_mut().connect_partitions(facade);
            }

            let can_set_new_lod =
                mesh.borrow().get_available_lods().contains(&lod) ||
                Self::is_mesh_task_running(&self.full_tasks, &self.low_tasks, chunk_pos, lod) &&
                Self::try_finish_mesh_task(
                    &mut self.full_tasks, &mut self.low_tasks,
                    chunk_pos, lod, &mut mesh.borrow_mut(), facade,
                ).await.is_ok();

            if can_set_new_lod {
                chunk.set_active_lod(&mesh.borrow(), lod);
            }
            
            else if self.can_start_tasks() {
                Self::start_task_gen_vertices(
                    &mut self.full_tasks,
                    &mut self.low_tasks,
                    Arc::clone(&chunk),
                    chunk_adj.clone(),
                    lod,
                ).await;
            }

            Self::drop_all_useless_tasks(&mut self.full_tasks, &mut self.low_tasks, lod, chunk_pos);

            if !chunk.can_render_active_lod(&mesh.borrow()) {
                chunk.try_set_best_fit_lod(&mesh.borrow(), lod);
            }

            // FIXME: make cam vis-check for light.
            if chunk.can_render_active_lod(&mesh.borrow()) && chunk.is_visible_by_camera(cam) {
                let active_lod = chunk.info.load(Relaxed).active_lod.unwrap();
                chunk.render(&mut mesh.borrow_mut(), target, draw_bundle, uniforms, active_lod)?
            }
        }

        Ok(())
    }

    pub fn drop_all_useless_tasks(
        full_tasks: &mut HashMap<Int3, FullTask>,
        low_tasks: &mut HashMap<(Int3, Lod), LowTask>,
        useful_lod: Lod, cur_pos: Int3,
    ) {
        for lod in Chunk::get_possible_lods() {
            if 2 < lod.abs_diff(useful_lod) {
                Self::drop_task(full_tasks, low_tasks, cur_pos, lod);
            }
        }
    }

    pub fn drop_task(
        full_tasks: &mut HashMap<Int3, FullTask>,
        low_tasks: &mut HashMap<(Int3, Lod), LowTask>,
        pos: Int3, lod: Lod,
    ) {
        match lod {
            0 =>   drop(full_tasks.remove(&pos)),
            lod => drop(low_tasks.remove(&(pos, lod))),
        }
    }

    pub fn drop_reader_tasks(
        full_tasks: &mut HashMap<Int3, FullTask>,
        low_tasks: &mut HashMap<(Int3, Lod), LowTask>,
        pos: Int3,
    ) {
        let vals_to_be_dropped = Chunk::get_possible_lods()
            .into_iter()
            .cartesian_product(SpaceIter::adj_iter(pos)
                .chain(std::iter::once(pos))
            );
        
        for (lod, pos) in vals_to_be_dropped {
            Self::drop_task(full_tasks, low_tasks, pos, lod);
        }
    }

    pub async fn try_finish_full_tasks(&mut self, facade: &dyn Facade) {
        let iter = self.full_tasks.iter_mut()
            .map(|(&pos, task)| (pos, task));

        for (pos, vertices) in Task::try_take_results(iter).await {
            self.full_tasks.remove(&pos);

            let idx = Self::pos_to_idx(self.sizes, pos)
                .expect("pos should be valid");

            self.meshes[idx].borrow_mut()
                .upload_full_detail_vertices(&vertices, facade);
        }
    }

    pub async fn try_finish_low_tasks(&mut self, facade: &dyn Facade) {
        let iter = self.low_tasks.iter_mut()
            .map(|(&idx, task)| (idx, task));

        for ((pos, lod), vertices) in Task::try_take_results(iter).await {
            self.low_tasks.remove(&(pos, lod));

            let idx = Self::pos_to_idx(self.sizes, pos)
                .expect("pos should be valid");

            self.meshes[idx].borrow_mut()
                .upload_low_detail_vertices(&vertices, lod, facade);
        }
    }

    pub async fn try_finish_gen_tasks(&mut self) {
        let iter = self.voxels_gen_tasks.iter_mut()
            .map(|(&pos, task)| (pos, task));

        for (pos, voxels) in Task::try_take_results(iter).await {
            self.voxels_gen_tasks.remove(&pos);

            let mut chunk = self.get_chunk_by_pos(pos)
                .expect("pos should be valid");

            Self::drop_reader_tasks(&mut self.full_tasks, &mut self.low_tasks, pos);
            
            // * Safety:
            // * Safe, because there's no chunk readers due to tasks drop above.
            unsafe {
                let _ = mem::replace(Arc::get_mut_unchecked(&mut chunk), Chunk::from_voxels(voxels, pos));
            }
        }
    }

    pub async fn try_finish_partition_tasks(&mut self, facade: &dyn Facade) {
        let iter = self.partition_tasks.iter_mut()
            .map(|(&pos, task)| (pos, task));

        for (pos, partitions) in Task::try_take_results(iter).await {
            self.partition_tasks.remove(&pos);

            let partitions = array_init(|i| partitions[i].as_slice());

            let idx = Self::pos_to_idx(self.sizes, pos)
                .expect("pos should be valid");

            self.meshes[idx].borrow_mut()
                .upload_partitioned_vertices(partitions, facade);
        }
    }

    pub async fn try_finish_all_tasks(&mut self, facade: &dyn Facade) {
        self.try_finish_full_tasks(facade).await;
        self.try_finish_low_tasks(facade).await;
        self.try_finish_gen_tasks().await;
        self.try_finish_partition_tasks(facade).await;
    }

    pub fn is_voxels_gen_task_running(tasks: &HashMap<Int3, GenTask>, pos: Int3) -> bool {
        tasks.contains_key(&pos)
    }

    /// Checks if generate mesh task id running.
    pub fn is_mesh_task_running(
        full_tasks: &HashMap<Int3, FullTask>,
        low_tasks: &HashMap<(Int3, Lod), LowTask>,
        pos: Int3, lod: Lod
    ) -> bool {
        match lod {
            0  => full_tasks.contains_key(&pos),
            lod => low_tasks.contains_key(&(pos, lod)),
        }
    }

    pub fn start_task_gen_voxels(tasks: &mut HashMap<Int3, GenTask>, pos: Int3, sizes: USize3) {
        let prev_value = tasks.insert(pos, Task::spawn(async move {
            Chunk::generate_voxels(pos, sizes)
        }));

        assert!(prev_value.is_none(), "threre should be only one task");
    }

    /// Checks that [chunk][Chunk] [adjacent][ChunkAdj] are generated.
    pub async fn is_adj_generated(adj: &ChunkAdj) -> bool {
        adj.inner.iter()
            .filter_map(Option::as_ref)
            .all(|chunk| chunk.is_generated())
    }

    /// Starts new generate vertices task.
    pub async fn start_task_gen_vertices(
        full_tasks: &mut HashMap<Int3, FullTask>,
        low_tasks: &mut HashMap<(Int3, Lod), LowTask>,
        chunk: ChunkRef, adj: ChunkAdj, lod: Lod,
    ) {
        let chunk_pos = chunk.pos.load(Relaxed);
        if lod == 0 && full_tasks.contains_key(&chunk_pos) ||
           lod != 0 && low_tasks.contains_key(&(chunk_pos, lod)) ||
           !chunk.is_generated() ||
           !Self::is_adj_generated(&adj).await
        { return }

        match lod {
            0 => {
                let prev = full_tasks.insert(chunk_pos, Task::spawn(async move {
                    chunk.make_vertices_detailed(adj)
                }));
                assert!(prev.is_none(), "there should be only one task");
            },

            lod => {
                let prev = low_tasks.insert((chunk_pos, lod), Task::spawn(async move {
                    chunk.make_vertices_low(adj, lod)
                }));
                assert!(prev.is_none(), "there should be only one task");
            },
        }
    }

    pub fn start_task_partitioning(
        tasks: &mut HashMap<Int3, PartitionTask>,
        chunk: ChunkRef, adj: ChunkAdj,
    ) {
        let prev_value = tasks.insert(chunk.pos.load(Relaxed), Task::spawn(async move {
            chunk.make_partitioned_vertices(adj)
        }));
        assert!(prev_value.is_none(), "there should be only one task");
    }

    pub async fn try_finish_voxels_gen_task(tasks: &mut HashMap<Int3, GenTask>, pos: Int3) -> Option<Chunk> {
        if let Some(task) = tasks.get_mut(&pos) {
            if let Some(voxel_ids) = task.try_take_result().await {
                tasks.remove(&pos);
                return Some(Chunk::from_voxels(voxel_ids, pos))
            }
        }

        None
    }

    /// Tries to get mesh from task if it is ready then sets it to chunk.
    /// Otherwise will return `Err(TaskError)`.
    pub async fn try_finish_mesh_task(
        full_tasks: &mut HashMap<Int3, FullTask>,
        low_tasks: &mut HashMap<(Int3, Lod), LowTask>,
        pos: Int3, lod: Lod,
        mesh: &mut ChunkMesh, facade: &dyn gl::backend::Facade,
    ) -> Result<(), TaskError> {
        match lod {
            0   => Self::try_finish_full_mesh_task(full_tasks, pos, mesh, facade).await,
            lod => Self::try_finish_low_mesh_task(low_tasks, pos, lod, mesh, facade).await,
        }
    }

    pub async fn try_finish_full_mesh_task(
        full_tasks: &mut HashMap<Int3, FullTask>,
        pos: Int3, mesh: &mut ChunkMesh, facade: &dyn gl::backend::Facade,
    ) -> Result<(), TaskError> {
        match full_tasks.get_mut(&pos) {
            Some(task) => match task.try_take_result().await {
                Some(vertices) => {
                    mesh.upload_full_detail_vertices(&vertices, facade);
                    let _ = full_tasks.remove(&pos)
                        .expect("there should be a task");
                    Ok(())
                },
                None => Err(TaskError::TaskNotReady),
            },
            None => Err(TaskError::TaskNotFound { lod: 0, pos }),
        }
    }
    
    pub async fn try_finish_low_mesh_task(
        low_tasks: &mut HashMap<(Int3, Lod), LowTask>,
        pos: Int3, lod: Lod,
        mesh: &mut ChunkMesh, facade: &dyn gl::backend::Facade,
    ) -> Result<(), TaskError> {
        match low_tasks.get_mut(&(pos, lod)) {
            Some(task) => match task.try_take_result().await {
                Some(vertices) => {
                    mesh.upload_low_detail_vertices(&vertices, lod, facade);
                    let _ = low_tasks.remove(&(pos, lod))
                        .expect("there should be a task");
                    Ok(())
                },
                None => Err(TaskError::TaskNotReady),
            },
            None => Err(TaskError::TaskNotFound { lod, pos })
        }
    }

    pub fn can_start_tasks(&self) -> bool {
        self.saving_handle.is_none() && self.reading_handle.is_none() &&
        self.low_tasks.len() + self.full_tasks.len() <= cfg::terrain::MAX_TASKS
    }

    pub fn drop_tasks(&mut self) {
        drop(mem::take(&mut self.full_tasks));
        drop(mem::take(&mut self.low_tasks));
        drop(mem::take(&mut self.voxels_gen_tasks));
        drop(mem::take(&mut self.partition_tasks));
    }

    pub fn any_task_running(&self) -> bool {
        !self.low_tasks.is_empty() ||
        !self.full_tasks.is_empty() ||
        !self.voxels_gen_tasks.is_empty() ||
        !self.partition_tasks.is_empty()
    }

    pub fn spawn_control_window(&mut self, ui: &imgui::Ui) {
        use crate::app::utils::graphics::ui::imgui_constructor::make_window;

        make_window(ui, "Chunk array")
            .always_auto_resize(true)
            .build(|| {
                ui.text(format!(
                    "{n} chunk generation tasks.",
                    n = self.voxels_gen_tasks.len(),
                ));

                ui.text(format!(
                    "{n} mesh generation tasks.",
                    n = self.low_tasks.len() + self.full_tasks.len(),
                ));

                ui.text(format!(
                    "{n} partition generation tasks.",
                    n = self.partition_tasks.len(),
                ));

                ui.slider(
                    "Chunks lod threashold",
                    0.01, 20.0,
                    &mut self.lod_threashold,
                );

                ui.separator();

                ui.text("Generate new");

                let mut sizes = GENERATOR_SIZES.lock()
                    .unwrap();

                ui.input_scalar_n("Sizes", &mut *sizes).build();

                if ui.button("Generate") {
                    self.drop_tasks();
                    match Self::new_empty_chunks(USize3::from(*sizes)) {
                        Ok(new_chunks) => {
                            let _ = mem::replace(self, new_chunks);
                        },
                        Err(err) => logger::log!(Error, from = "chunk-array", "{err}")
                    }
                }
            });
    }

    pub async fn process_commands(&mut self, facade: &dyn Facade) {
        #![allow(clippy::await_holding_lock)]

        use crate::app::utils::terrain::chunk::commands::*;

        let mut commands = COMMAND_CHANNEL.lock().unwrap();
        let mut change_tracker = ChangeTracker::new(self.sizes);

        use Command::*;
        while let Ok(command) = commands.receiver.try_recv() {
            match command {
                SetVoxel { pos, new_id } => {
                    let old_id = self.set_voxel(pos, new_id)
                        .unwrap_or_else(|err| {
                            logger::log!(Error, from = "chunk-array", "failed to set voxel: {err}");
                            0
                        });

                    if old_id != new_id {
                        change_tracker.track_voxel(pos);
                    }
                },

                FillVoxels { pos_from, pos_to, new_id } => {
                    let _is_changed = self.fill_voxels(pos_from, pos_to, new_id)
                        .unwrap_or_else(|err| {
                            logger::log!(Error, from = "chunk-array", "failed to fill voxels: {err}");
                            false
                        });
                }

                DropAllMeshes => self.drop_all_meshes(),
            }
        }

        drop(commands);

        let idxs_to_reload = change_tracker.idxs_to_reload_partitioning();
        let n_changed = idxs_to_reload.len();
        for (idx, partition_idx) in idxs_to_reload {
            self.reload_chunk_partitioning(idx, partition_idx, facade).await;
        }

        if n_changed != 0 {
            logger::log!(Info, from = "chunk-array", "{n_changed} chunks were updated!");
        }
    }

    pub async fn reload_chunk(&self, idx: usize, facade: &dyn Facade) {
        let chunk_pos = Self::idx_to_pos(idx, self.sizes);
        let adj = self.get_adj_chunks(chunk_pos);

        if let Some(chunk) = self.chunks.get(idx) {
            let mut mesh = self.meshes[idx].borrow_mut();
            chunk.generate_mesh(&mut mesh, 0, adj, facade);
        }
    }

    pub async fn reload_chunk_partitioning(
        &self, chunk_idx: usize, partition_idx: usize, facade: &dyn Facade,
    ) {
        let chunk_pos = Self::idx_to_pos(chunk_idx, self.sizes);
        let adj = self.get_adj_chunks(chunk_pos);

        if let Some(chunk) = self.chunks.get(chunk_idx) {
            let mut mesh = self.meshes[chunk_idx].borrow_mut();
            if mesh.is_partitioned() {
                let partial_vertices = chunk.make_partition(&adj, partition_idx);
                mesh.upload_partition(&partial_vertices, partition_idx, facade);
            } else {
                chunk.partition_mesh(&mut mesh, adj, facade);
            }
        }
    }

    pub fn trace_ray(&self, ray: Line, max_steps: usize) -> impl Iterator<Item = Voxel> + '_ {
        (0..max_steps)
            .filter_map(move |i| {
                let pos = ray.point_along(i as f32 * 0.125);
                let pos = Int3::new(
                    pos.x.round() as i32,
                    pos.y.round() as i32,
                    pos.z.round() as i32,
                );

                self.get_voxel(pos)
            })
    }

    pub async fn proccess_camera_input(&mut self, cam: &Camera) {
        use super::commands::{command, Command};

        let first_voxel = self.trace_ray(Line::new(cam.pos, cam.front), Self::MAX_TRACE_STEPS)
            .find(|voxel| !voxel.is_air());

        match first_voxel {
            Some(voxel) if mouse::just_left_pressed() && cam.captures_mouse =>
                command(Command::SetVoxel { pos: voxel.pos, new_id: AIR_VOXEL_DATA.id }),

            _ => (),
        }
    }

    pub async fn update(&mut self, facade: &dyn Facade, cam: &Camera) -> Result<(), UpdateError> {
        self.proccess_camera_input(cam).await;
        self.process_commands(facade).await;

        if keyboard::just_pressed_combo([Key::LControl, Key::S]) {
            let chunks: Vec<_> = self.chunks.iter().map(Arc::clone).collect();
            let handle = tokio::spawn(
                ChunkArray::save_to_file(self.sizes, chunks, "world", "world")
            );
            self.saving_handle = Some(handle);
        }

        if self.saving_handle.is_some() && self.saving_handle.as_ref().unwrap().is_finished() {
            let handle = self.saving_handle.take().unwrap();
            handle.await??;
        }

        if keyboard::just_pressed_combo([Key::LControl, Key::O]) {
            let handle = tokio::spawn(ChunkArray::read_from_file("world", "world"));
            self.reading_handle = Some(handle);
        }

        if self.reading_handle.is_some() && self.reading_handle.as_ref().unwrap().is_finished() {
            let handle = self.reading_handle.take().unwrap();
            let (sizes, arr) = handle.await??;
            self.apply_new(sizes, arr)?;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error(transparent)]
    Join(#[from] JoinError),

    #[error("failed to save chunk array: {0}")]
    Save(#[from] io::Error),

    #[error("error occured: {0}")]
    Other(#[from] UserFacingError),
}

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("task is not already finished")]
    TaskNotReady,

    #[error("there is no task to generate mesh with lod {lod} and pos {pos} in map")]
    TaskNotFound {
        lod: Lod,
        pos: Int3,
    },
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

            let chunk_idx = match ChunkArray::pos_to_idx(self.sizes, chunk_pos) {
                Some(idx) => idx,
                None => continue,
            };

            for offset in iterator::offsets_from_border(local_pos, Int3::ZERO..Int3::from(Chunk::SIZES)) {
                match ChunkArray::pos_to_idx(self.sizes, chunk_pos + offset) {
                    Some(idx) => { result.insert(idx); },
                    None => continue,
                }
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

            let chunk_idx = match ChunkArray::pos_to_idx(self.sizes, chunk_pos) {
                Some(idx) => idx,
                None => continue,
            };

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

                match ChunkArray::pos_to_idx(self.sizes, adj_chunk_pos) {
                    Some(idx) => { result.insert((idx, partition_idx)); },
                    None => continue,
                }
            }
        }

        result
    }
}

pub type ChunkRef = Arc<Chunk>;
pub type MeshRef = Rc<RefCell<ChunkMesh>>;
pub type ChunkAdj = Sides<Option<Arc<Chunk>>>;