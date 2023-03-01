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
                tasks::Task,
            },
            voxel::Voxel,
        },
        cfg,
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
    pub tasks: HashMap<Int3, Task>,
}

impl ChunkArray {
    pub fn new(sizes: USize3) -> Self {
        let start_pos = Self::coord_idx_to_pos(sizes, USize3::ZERO);
        let end_pos   = Self::coord_idx_to_pos(sizes, sizes);

        let chunks = SpaceIter::new(start_pos..end_pos)
            .map(|pos| Chunk::new(pos))
            .collect();

        Self { chunks, sizes, tasks: HashMap::new() }
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
                let cam_pos_in_chunks = cam_pos / (Chunk::SIZE as f32 * cfg::terrain::VOXEL_SIZE);
                let chunk_pos = vec3::from(chunk_pos);

                Lod::min(
                    (chunk_pos - cam_pos_in_chunks).len().floor() as Lod,
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

    pub fn render(
        &mut self, target: &mut gl::Frame, draw_bundle: &ChunkDrawBundle,
        uniforms: &impl gl::uniforms::Uniforms, display: &gl::Display, cam_pos: vec3,
    ) -> Result<(), ChunkRenderError> {
        let sizes = self.sizes;
        if sizes == USize3::ZERO { return Ok(()) }

        let iter = self.chunks_with_adj_mut()
            .zip(Self::lod_iter(sizes, cam_pos));

        // TODO: draw chunk with desired lod.
        for ((chunk, chunk_adj), lod) in iter {
            if chunk.try_set_active_lod(lod).is_err() {
                chunk.generate_mesh(lod, chunk_adj, display);
                chunk.set_active_lod(lod);
            }

            chunk.render(target, &draw_bundle, uniforms, chunk.active_lod)?
        }

        Ok(())
    }
}
