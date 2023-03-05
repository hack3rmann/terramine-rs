use {
    crate::app::utils::{
        terrain::{
            chunk::{
                FillType,
                Chunk,
                ChunkRef,
                ChunkAdj,
                iterator::{SpaceIter, self},
                Lod, Id,
                ChunkDrawBundle,
                ChunkRenderError,
                tasks::{FullTask, LowTask, Task, GenTask},
            },
            voxel::Voxel,
        },
        saves::Save,
        reinterpreter::*,
        graphics::camera::Camera,
    },
    math_linear::prelude::*,
    std::{
        ptr::NonNull,
        slice::{Iter, IterMut},
        collections::HashMap,
        io,
        mem,
    },
    glium as gl,
};

#[derive(Clone, Copy, Debug)]
enum ChunkArrSaveType {
    Sizes,
    Array,
}

impl From<ChunkArrSaveType> for u64 {
    fn from(value: ChunkArrSaveType) -> Self { value as u64 }
}

/// Represents 3d array of [`Chunk`]s. Can control their mesh generation, etc.
#[derive(Debug)]
pub struct ChunkArray {
    pub chunks: Vec<Chunk>,
    pub sizes: USize3,

    pub full_tasks: HashMap<Int3, FullTask>,
    pub low_tasks: HashMap<(Int3, Lod), LowTask>,
    pub voxels_gen_tasks: HashMap<Int3, GenTask>,

    pub lod_dist_threashold: f32,
}

impl Default for ChunkArray {
    fn default() -> Self {
        Self {
            chunks: Default::default(),
            sizes: Default::default(),
            full_tasks: Default::default(),
            low_tasks: Default::default(),
            voxels_gen_tasks: Default::default(),
            lod_dist_threashold: 5.8,
        }
    }
}

impl ChunkArray {
    /// Generates new chunks.
    pub fn new(sizes: USize3) -> Self {
        let (start_pos, end_pos) = Self::pos_bounds(sizes);

        let chunks = SpaceIter::new(start_pos..end_pos)
            .map(|pos| Chunk::new(pos))
            .collect();

        Self::from_chunks(sizes, chunks)
    }

    /// Constructs [`ChunkArray`] with passed in chunks.
    pub fn from_chunks(sizes: USize3, chunks: Vec<Chunk>) -> Self {
        let volume = Self::volume(sizes);
        assert_eq!(
            chunks.len(), volume,
            "passed in chunk `Vec` should have same size as passed in sizes, but sizes: {sizes}, len: {len}",
            len = chunks.len(),
        );
        
        Self { chunks, sizes, ..Default::default() }
    }

    /// Constructs [`ChunkArray`] with empty chunks.
    pub fn new_empty_chunks(sizes: USize3) -> Self {
        let (start_pos, end_pos) = Self::pos_bounds(sizes);

        let chunks = SpaceIter::new(start_pos..end_pos)
            .map(|pos| Chunk::new_empty(pos))
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

    /// Gives empty [`ChunkArray`].
    pub fn new_empty() -> Self {
        Self::default()
    }

    pub fn save_to_file(&self, save_name: &str, save_path: &str) -> io::Result<()> {
        let volume = Self::volume(self.sizes);

        Save::new(save_name)
            .create(save_path)?
            .write(&self.sizes, ChunkArrSaveType::Sizes)
            .pointer_array(volume, ChunkArrSaveType::Array, |i| {
                match self.chunks[i].info.fill_type {
                    FillType::AllSame(id) => FillType::AllSame(id)
                        .reinterpret_as_bytes()
                        .into_iter()
                        .chain(self.chunks[i].pos.reinterpret_as_bytes())
                        .collect(),

                    FillType::Default => {
                        assert_eq!(
                            self.chunks.len(), Chunk::VOLUME,
                            "cannot save unknown-sized chunk with size {size}",
                            size = self.chunks.len(),
                        );

                        FillType::Default
                            .reinterpret_as_bytes()
                    }
                }
            })
            .save()?;

        Ok(())
    }

    pub fn read_from_file(&mut self, save_name: &str, save_path: &str) -> io::Result<()> {
        let save = Save::new(save_name).open(save_path)?;

        self.sizes = save.read(ChunkArrSaveType::Sizes);

        self.chunks = save.read_pointer_array(ChunkArrSaveType::Array, |i, mut bytes| {
            let chunk_pos = Self::idx_to_pos(i, self.sizes);

            let fill_type = FillType::reinterpret_from_bytes(bytes);
            bytes = &bytes[FillType::static_size()..];

            match fill_type {
                FillType::Default => {
                    let voxel_ids = Vec::<Id>::reinterpret_from_bytes(bytes);
                    Chunk::from_voxels(voxel_ids, chunk_pos)
                },

                FillType::AllSame(id) =>
                    Chunk::new_same_filled(chunk_pos, id),
            }
        });

        Ok(())
    }

    /// Gives chunk count.
    pub fn volume(arr_sizes: USize3) -> usize {
        arr_sizes.x * arr_sizes.y * arr_sizes.z
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

    /// Convertes chunk pos to an array index.
    pub fn pos_to_idx(sizes: USize3, pos: Int3) -> Option<usize> {
        Some(
            Self::coord_idx_to_idx(sizes, Self::pos_to_coord_idx(sizes, pos)?)
        )
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

    /// Gives reference to chunk by it's position.
    pub fn get_chunk_by_pos(&self, pos: Int3) -> Option<ChunkRef<'_>> {
        let idx = Self::pos_to_idx(self.sizes, pos)?;
        Some(self.chunks[idx].make_ref())
    }

    /// Gives reference to chunk by it's position.
    pub fn get_chunk_mut_by_pos(&mut self, pos: Int3) -> Option<&mut Chunk> {
        let idx = Self::pos_to_idx(self.sizes, pos)?;
        Some(&mut self.chunks[idx])
    }

    /// Gives adjacent chunks references by center chunk position.
    pub fn get_adj_chunks(&self, pos: Int3) -> ChunkAdj<'_> {
        let mut adj = ChunkAdj::none();
        let adjs = SpaceIter::adj_iter(Int3::ZERO)
            .filter_map(|off|
                Some((off, self.get_chunk_by_pos(pos + off)?))
            );

        for (offset, chunk) in adjs {
            adj.sides.set(offset, Some(chunk))
                .expect("offset should be adjacent (see SpaceIter::adj_iter())");
        }

        adj
    }

    /// Gives iterator over chunk coordinates.
    pub fn pos_iter(sizes: USize3) -> SpaceIter {
        let (start, end) = Self::pos_bounds(sizes);
        SpaceIter::new(start..end)
    }

    /// Gives iterator over all chunk's adjacents.
    pub fn adj_iter(&self) -> impl Iterator<Item = ChunkAdj<'_>> {
        Self::pos_iter(self.sizes)
            .map(move |pos| self.get_adj_chunks(pos))
    }

    /// Gives iterator over desired LOD for each chunk.
    pub fn lod_iter(chunk_array_sizes: USize3, cam_pos: vec3, threashold: f32) -> impl Iterator<Item = Lod> {
        Self::pos_iter(chunk_array_sizes)
            .map(move |chunk_pos| {
                let chunk_size = Chunk::GLOBAL_SIZE;
                let cam_pos_in_chunks = cam_pos / chunk_size;
                let chunk_pos = vec3::from(chunk_pos);

                let dist = (chunk_pos - cam_pos_in_chunks + vec3::all(0.5)).len();
                Lod::min(
                    (dist / threashold).floor() as Lod,
                    Chunk::SIZE.ilog2() as Lod,
                )
            })
    }

    /// Gives iterator over chunks.
    pub fn chunks(&self) -> Iter<'_, Chunk> {
        self.chunks.iter()
    }

    /// Gives mutable iterator over chunks.
    pub fn chunks_mut(&mut self) -> IterMut<'_, Chunk> {
        self.chunks.iter_mut()
    }

    /// Gives iterator over all voxels in [`ChunkArray`].
    pub fn voxels(&self) -> impl Iterator<Item = Voxel> + '_ {
        self.chunks().flat_map(|chunk| chunk.voxels())
    }

    /// Gives iterator over mutable chunks and their adjacents.
    pub fn chunks_with_adj_mut(&mut self) -> impl Iterator<Item = (&mut Chunk, ChunkAdj<'_>)> + '_ {
        // * Safe bacause shared adjacent chunks are not aliasing current mutable chunk
        // * and the reference is made of a pointer that is made from that reference.
        let aliased = unsafe { NonNull::new_unchecked(self as *mut Self).as_ref() };

        let adj_iter = aliased.adj_iter();
        self.chunks_mut()
            .zip(adj_iter)
    }

    /// Generates mesh for each chunk.
    pub fn generate_meshes(&mut self, lod: impl Fn(Int3) -> Lod, display: &gl::Display) {
        for (chunk, adj) in self.chunks_with_adj_mut() {
            let active_lod = lod(chunk.pos);
            chunk.generate_mesh(active_lod, adj, display);
            chunk.set_active_lod(active_lod);
        }
    }

    /// Renders all chunks. If chunk should have another LOD then it will start async
    /// task that generates desired mesh. If task is incomplete then it will render active LOD
    /// of concrete chunk. If it can't then it will do nothing.
    pub async fn render(
        &mut self, target: &mut gl::Frame, draw_bundle: &ChunkDrawBundle<'_>,
        uniforms: &impl gl::uniforms::Uniforms, display: &gl::Display, cam: &Camera,
    ) -> Result<(), ChunkRenderError> {
        let sizes = self.sizes;
        if sizes == USize3::ZERO { return Ok(()) }

        self.try_finish_all_tasks(display).await;

        // FIXME:
        let aliased_self = unsafe { NonNull::new_unchecked(self as *mut Self).as_mut() };

        let mut chunks: Vec<_> = self.chunks_mut()
            .zip(Self::lod_iter(sizes, cam.pos, aliased_self.lod_dist_threashold))
            .collect();

        chunks.sort_by(|(lhs, _), (rhs, _)| {
            let l_dist = vec3::len(cam.pos - lhs.pos.into());
            let r_dist = vec3::len(cam.pos - rhs.pos.into());

            l_dist.partial_cmp(&r_dist)
                .expect("distance to chunk should be a number")
        });

        for (chunk, lod) in chunks {
            if !chunk.is_generated() {
                if aliased_self.is_voxels_gen_task_running(chunk.pos) {
                    aliased_self.try_finish_voxels_gen_task(chunk.pos, chunk).await
                } else {
                    aliased_self.start_task_gen_voxels(chunk.pos);
                    continue;
                }
            }

            let can_set_new_lod =
                aliased_self.is_mesh_task_running(chunk.pos, lod) &&
                aliased_self.try_finish_mesh_task(chunk.pos, lod, chunk, display).await.is_ok() ||
                chunk.get_available_lods().contains(&lod);

            if can_set_new_lod {
                chunk.set_active_lod(lod)
            } else {
                aliased_self.start_task_gen_vertices(lod, chunk.pos)
            }

            aliased_self.drop_all_useless_tasks(lod, chunk.pos);

            if !chunk.can_render_active_lod() {
                chunk.try_set_best_fit_lod(lod);
            }

            if chunk.can_render_active_lod() && chunk.is_visible_by_camera(cam) {
                chunk.render(target, &draw_bundle, uniforms, chunk.info.active_lod)?
            }
        }

        Ok(())
    }

    pub fn drop_all_useless_tasks(&mut self, useful_lod: Lod, cur_pos: Int3) {
        for lod in Chunk::get_possible_lods() {
            if 2 < lod.abs_diff(useful_lod) {
                self.drop_task(cur_pos, lod);
            }
        }
    }

    pub fn drop_task(&mut self, pos: Int3, lod: Lod) {
        match lod {
            0 => {
                let _ = self.full_tasks.remove(&pos);
            },

            lod => {
                let _ = self.low_tasks.remove(&(pos, lod));
            },
        }
    }

    pub async fn try_finish_full_tasks(&mut self, display: &gl::Display) {
        let full: Vec<_> = self.full_tasks.iter_mut()
            .filter(|(_, task)| match task.handle.as_ref() {
                None => false,
                Some(handle) => handle.is_finished()
            })
            .map(|(&pos, task)|
                (pos, task.take_result())
            )
            .collect();

        let mut new_full = Vec::with_capacity(full.len());
        for (pos, fut) in full {
            new_full.push((pos, fut.await));
        }

        for (pos, vertices) in new_full {
            self.full_tasks.remove(&pos);

            self.get_chunk_mut_by_pos(pos)
                .expect("pos should be valid")
                .upload_full_detail_vertices(&vertices, display);
        }
    }

    pub async fn try_finish_low_tasks(&mut self, display: &gl::Display) {
        let low: Vec<_> = self.low_tasks.iter_mut()
            .filter(|(_, task)| match task.handle.as_ref() {
                None => false,
                Some(handle) => handle.is_finished()
            })
            .map(|(&(pos, lod), task)|
                (pos, lod, task.take_result())
            )
            .collect();

        let mut new_low = Vec::with_capacity(low.len());
        for (pos, lod, fut) in low {
            new_low.push((pos, lod, fut.await));
        }

        for (pos, lod, vertices) in new_low {
            self.low_tasks.remove(&(pos, lod));

            self.get_chunk_mut_by_pos(pos)
                .expect("pos should be valid")
                .upload_low_detail_vertices(&vertices, lod, display);
        }
    }

    pub async fn try_finish_gen_tasks(&mut self) {
        let voxel_futs: Vec<_> = self.voxels_gen_tasks.iter_mut()
            .filter(|(_, task)| match task.handle {
                None => false,
                Some(ref handle) => handle.is_finished(),
            })
            .map(|(&pos, task)| (pos, task.take_result()))
            .collect();

        let mut voxel_vecs = Vec::with_capacity(voxel_futs.len());
        for (pos, fut) in voxel_futs {
            voxel_vecs.push((pos, fut.await));
        }

        for (pos, voxels) in voxel_vecs {
            self.voxels_gen_tasks.remove(&pos);

            let chunk = self.get_chunk_mut_by_pos(pos)
                .expect("pos should be valid");
            *chunk = Chunk::from_voxels(voxels, pos);
        }
    }

    pub async fn try_finish_all_tasks(&mut self, display: &gl::Display) {
        self.try_finish_full_tasks(display).await;
        self.try_finish_low_tasks(display).await;
        self.try_finish_gen_tasks().await;
    }

    pub fn is_voxels_gen_task_running(&self, pos: Int3) -> bool {
        self.voxels_gen_tasks.contains_key(&pos)
    }

    /// Checks if generate mesh task id running.
    pub fn is_mesh_task_running(&self, pos: Int3, lod: Lod) -> bool {
        match lod {
            0 =>
                self.full_tasks.contains_key(&pos),
            lod =>
                self.low_tasks.contains_key(&(pos, lod)),
        }
    }

    pub fn start_task_gen_voxels(&mut self, pos: Int3) {
        let prev_value = self.voxels_gen_tasks.insert(pos, Task::spawn(async move {
            Chunk::generate_voxels(pos)
        }));

        assert!(prev_value.is_none(), "threre should be only one task");
    }

    /// Starts new generate vertices task.
    pub fn start_task_gen_vertices(&mut self, lod: Lod, pos: Int3) {
        // FIXME: fix this UB:
        let aliased_mut = unsafe { NonNull::new_unchecked(self as *mut Self).as_mut() };
        let aliased_ref = unsafe { NonNull::new_unchecked(self as *mut Self).as_ref() };

        let chunk = aliased_ref.get_chunk_by_pos(pos)
            .expect(&format!("chunk with pos {pos:?} should exist"));
        let adj = aliased_ref.get_adj_chunks(pos);

        let is_adj_generated = adj.sides.inner
            .iter()
            .copied()
            .filter_map(std::convert::identity)
            .all(|chunk| chunk.is_generated());

        if !chunk.is_generated() || !is_adj_generated { return }

        match lod {
            0 => if !aliased_mut.full_tasks.contains_key(&pos) {
                let prev = aliased_mut.full_tasks.insert(pos, Task::spawn(async move {
                    chunk.make_vertices_detailed(adj)
                }));
                assert!(prev.is_none(), "there should be only one task");
            },

            lod if !aliased_mut.low_tasks.contains_key(&(pos, lod)) => {
                let prev = aliased_mut.low_tasks.insert((pos, lod), Task::spawn(async move {
                    chunk.make_vertices_low(adj, lod)
                }));
                assert!(prev.is_none(), "there should be only one task");
            },

            _ => (),
        }
    }

    pub async fn try_finish_voxels_gen_task(&mut self, pos: Int3, chunk: &mut Chunk) {
        if let Some(task) = self.voxels_gen_tasks.get_mut(&pos) {
            if let Some(voxel_ids) = task.try_take_result().await {
                *chunk = Chunk::from_voxels(voxel_ids, pos);
                self.voxels_gen_tasks.remove(&pos);
            }
        }
    }

    // TODO: return more informative `Result`
    /// Tries to get mesh from task if it is ready then sets it to chunk.
    /// Otherwise will return `Err(())`.
    pub async fn try_finish_mesh_task(
        &mut self, pos: Int3, lod: Lod,
        chunk: &mut Chunk, display: &gl::Display
    ) -> Result<(), ()> {
        match lod {
            0 => match self.full_tasks.get_mut(&pos) {
                Some(task) => match task.try_take_result().await {
                    None => Err(()),
                    Some(vertices) => {
                        chunk.upload_full_detail_vertices(&vertices, display);
                        let _ = self.full_tasks.remove(&pos)
                            .expect("there should be a task");
                        Ok(())
                    }
                },
                None => Err(()),
            },

            lod => match self.low_tasks.get_mut(&(pos, lod)) {
                Some(task) => match task.try_take_result().await {
                    None => Err(()),
                    Some(vertices) => {
                        chunk.upload_low_detail_vertices(&vertices, lod, display);
                        let _ = self.low_tasks.remove(&(pos, lod))
                            .expect("there should be a task");
                        Ok(())
                    }
                },
                None => Err(())
            }
        }
    }

    pub fn drop_tasks(&mut self) {
        let _ = mem::take(&mut self.full_tasks);
        let _ = mem::take(&mut self.low_tasks);
        let _ = mem::take(&mut self.voxels_gen_tasks);
    }

    pub fn spawn_control_window(&mut self, ui: &imgui::Ui) {
        ui.window("Chunk array")
            .always_auto_resize(true)
            .collapsible(true)
            .movable(true)
            .build(|| {
                ui.text(format!(
                    "{n} chunk generation tasks.",
                    n = self.voxels_gen_tasks.len()
                ));

                ui.text(format!(
                    "{n} mesh generation tasks.",
                    n = self.low_tasks.len() + self.full_tasks.len()
                ));

                ui.slider(
                    "Chunks lod threashold",
                    0.01, 20.0,
                    &mut self.lod_dist_threashold,
                );
            });
    }
}
