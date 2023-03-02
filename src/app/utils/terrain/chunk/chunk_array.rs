use {
    crate::app::utils::{
        terrain::{
            chunk::{
                Chunk,
                ChunkRef,
                ChunkAdj,
                iterator::SpaceIter,
                Lod,
                ChunkDrawBundle,
                ChunkRenderError,
                tasks::{FullTask, LowTask, Task},
            },
            voxel::Voxel,
        },
        cfg,
        profiler::prelude::*,
    },
    math_linear::prelude::*,
    std::{
        ptr::NonNull,
        slice::{Iter, IterMut},
        collections::HashMap,
    },
    glium as gl,
};

#[derive(Debug)]
pub struct ChunkArray {
    pub chunks: Vec<Chunk>,
    pub sizes: USize3,

    pub full_tasks: HashMap<Int3, FullTask>,
    pub low_tasks: HashMap<(Int3, Lod), LowTask>,
}

impl ChunkArray {
    pub fn new(sizes: USize3) -> Self {
        let start_pos = Self::coord_idx_to_pos(sizes, USize3::ZERO);
        let end_pos   = Self::coord_idx_to_pos(sizes, sizes);

        let chunks = SpaceIter::new(start_pos..end_pos)
            .map(|pos| Chunk::new(pos))
            .collect();

        Self { chunks, sizes, full_tasks: HashMap::new(), low_tasks: HashMap::new() }
    }

    pub fn new_empty() -> Self {
        Self::new(USize3::ZERO)
    }

    pub fn coord_idx_to_pos(sizes: USize3, coord_idx: USize3) -> Int3 {
        Int3::from(coord_idx) - Int3::from(sizes) / 2
    }

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

    pub fn coord_idx_to_idx(sizes: USize3, coord_idx: USize3) -> usize {
        sdex::get_index(&coord_idx.as_array(), &sizes.as_array())
    }

    pub fn pos_to_idx(sizes: USize3, pos: Int3) -> Option<usize> {
        Some(
            Self::coord_idx_to_idx(sizes, Self::pos_to_coord_idx(sizes, pos)?)
        )
    }

    pub fn get_chunk_by_pos(&self, pos: Int3) -> Option<ChunkRef> {
        Some(
            self.chunks[Self::pos_to_idx(self.sizes, pos)?].make_ref()
        )
    }

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

    pub fn pos_iter(sizes: USize3) -> SpaceIter {
        let start = Self::coord_idx_to_pos(sizes, USize3::ZERO);
        let end   = Self::coord_idx_to_pos(sizes, sizes);

        SpaceIter::new(start..end)
    }

    pub fn adj_iter(&self) -> impl Iterator<Item = ChunkAdj<'_>> {
        Self::pos_iter(self.sizes)
            .map(move |pos| self.get_adj_chunks(pos))
    }

    pub fn lod_iter(chunk_array_sizes: USize3, cam_pos: vec3) -> impl Iterator<Item = Lod> {
        Self::pos_iter(chunk_array_sizes)
            .map(move |chunk_pos| {
                let chunk_size = Chunk::SIZE as f32 * cfg::terrain::VOXEL_SIZE;
                let cam_pos_in_chunks = cam_pos / chunk_size;
                let chunk_pos = vec3::from(chunk_pos);

                let dist = (chunk_pos - cam_pos_in_chunks + vec3::all(0.5)).len();
                Lod::min(
                    (dist / 2.0).floor() as Lod,
                    Chunk::SIZE.ilog2() as Lod,
                )
            })
    }

    pub fn chunks(&self) -> Iter<'_, Chunk> {
        self.chunks.iter()
    }

    pub fn voxels(&self) -> impl Iterator<Item = Voxel> + '_ {
        self.chunks().flat_map(|chunk| chunk.voxels())
    }

    pub fn chunks_mut(&mut self) -> IterMut<'_, Chunk> {
        self.chunks.iter_mut()
    }

    pub fn chunks_with_adj_mut(&mut self) -> impl Iterator<Item = (&mut Chunk, ChunkAdj<'_>)> + '_ {
        // * Safe bacause shared adjacent chunks are not aliasing current mutable chunk
        // * and the reference is made of a pointer that is made from that reference.
        let aliased = unsafe { NonNull::new_unchecked(self as *mut Self).as_ref() };

        let adj_iter = aliased.adj_iter();
        self.chunks_mut()
            .zip(adj_iter)
    }

    pub fn generate_meshes(&mut self, lod: impl Fn(Int3) -> Lod, display: &gl::Display) {
        for (chunk, adj) in self.chunks_with_adj_mut() {
            let active_lod = lod(chunk.pos);
            chunk.generate_mesh(active_lod, adj, display);
            chunk.set_active_lod(active_lod);
        }
    }

    #[profile]
    pub async fn render(
        &mut self, target: &mut gl::Frame, draw_bundle: &ChunkDrawBundle<'_>,
        uniforms: &impl gl::uniforms::Uniforms, display: &gl::Display, cam_pos: vec3,
    ) -> Result<(), ChunkRenderError> {
        let sizes = self.sizes;
        if sizes == USize3::ZERO { return Ok(()) }

        // FIXME:
        let aliased_mut = unsafe { NonNull::new_unchecked(self as *mut Self).as_mut() };

        let iter = self.chunks_mut()
            .zip(Self::lod_iter(sizes, cam_pos));

        for (chunk, lod) in iter {
            if aliased_mut.is_task_running(chunk.pos, lod) &&
               aliased_mut.try_finish_task(chunk.pos, lod, chunk, display).await.is_ok() ||
               chunk.get_available_lods().contains(&lod)
            {
                chunk.set_active_lod(lod)
            } else {
                aliased_mut.start_task_generate_vertices(lod, chunk.pos)
            }

            if chunk.can_render_active_lod() {
                chunk.render(target, &draw_bundle, uniforms, chunk.active_lod)?
            }
        }

        Ok(())
    }

    #[profile]
    pub fn is_task_running(&self, pos: Int3, lod: Lod) -> bool {
        match lod {
            0 =>
                self.full_tasks.contains_key(&pos),
            lod =>
                self.low_tasks.contains_key(&(pos, lod)),
        }
    }

    #[profile]
    pub fn start_task_generate_vertices(&mut self, lod: Lod, pos: Int3) {
        // FIXME: fix this UB:
        let aliased_mut = unsafe { NonNull::new_unchecked(self as *mut Self).as_mut() };
        let aliased_ref = unsafe { NonNull::new_unchecked(self as *mut Self).as_ref() };

        let chunk = aliased_ref.get_chunk_by_pos(pos)
            .expect(&format!("chunk with pos {pos:?} should exist"));
        let adj = aliased_ref.get_adj_chunks(pos);

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

    #[profile]
    pub async fn try_finish_task(&mut self, pos: Int3, lod: Lod, chunk: &mut Chunk, display: &gl::Display) -> Result<(), ()> {
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
}
