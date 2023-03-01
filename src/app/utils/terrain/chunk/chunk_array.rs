use {
    crate::app::utils::terrain::{
        chunk::{
            Chunk,
            ChunkAdj,
            iterator::SpaceIter,
            Lod,
            ChunkDrawBundle,
        },
        voxel::Voxel,
    },
    math_linear::prelude::*,
    std::{
        ptr::NonNull,
        slice::{Iter, IterMut},
    },
    glium as gl,
};

#[derive(Debug)]
pub struct ChunkArray {
    pub chunks: Vec<Chunk>,
    pub sizes: USize3,
}

impl ChunkArray {
    pub fn new(sizes: USize3) -> Self {
        let start_pos = Self::coord_idx_to_pos(sizes, USize3::ZERO);
        let end_pos   = Self::coord_idx_to_pos(sizes, sizes);

        let chunks = SpaceIter::new(start_pos..end_pos)
            .map(|pos| Chunk::new(pos))
            .collect();

        Self { chunks, sizes }
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

        // FIXME: (see another fixme)
        //match (Int3::ZERO..sizes).contains(&shifted){
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

    pub fn get_chunk_by_pos(&self, pos: Int3) -> Option<&Chunk> {
        Some(
            &self.chunks[Self::pos_to_idx(self.sizes, pos)?]
        )
    }

    pub fn get_adj_chunks(&self, pos: Int3) -> ChunkAdj<'_> {
        let mut adj = ChunkAdj::none();
        let adjs = SpaceIter::adj_iter(Int3::ZERO)
            .filter_map(|off|
                Some((off, self.get_chunk_by_pos(pos + off)?))
            );

        for (offset, chunk) in adjs {
            adj.sides.set(offset, Some(NonNull::from(chunk)))
                .expect("offset should be adjacent (see SpaceIter::adj_iter())");
        }

        adj
    }

    pub fn pos_iter(sizes: USize3) -> SpaceIter {
        let start = Self::coord_idx_to_pos(sizes, USize3::ZERO);
        let end   = Self::coord_idx_to_pos(sizes, sizes);

        SpaceIter::new(start..end)
    }

    // TODO: make mut and shared versions of ChunkAdj.
    pub fn adj_iter(&self) -> impl Iterator<Item = ChunkAdj<'_>> {
        Self::pos_iter(self.sizes)
            .map(move |pos| self.get_adj_chunks(pos))
    }

    pub fn lod_iter(&self, _cam_pos: vec3) -> impl Iterator<Item = Lod> {
        //todo!();
        //std::iter::empty()
        std::iter::once(1).cycle()
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
            .map(|(chunk, adj)|
                (chunk, adj)
            )
    }

    pub fn generate_meshes(&mut self, lod: impl Fn(Int3) -> Lod, display: &gl::Display) {
        for (chunk, adj) in self.chunks_with_adj_mut() {
            let active_lod = lod(chunk.pos);
            chunk.generate_mesh(active_lod, adj, display);
            chunk.set_active_lod(active_lod);
        }
    }

    pub fn render(
        &self, target: &mut gl::Frame, draw_bundle: &ChunkDrawBundle,
        uniforms: &impl gl::uniforms::Uniforms, cam_pos: vec3,
    ) -> Result<(), gl::DrawError> {
        if self.sizes == USize3::ZERO { return Ok(()) }

        for (chunk, _lod) in self.chunks().zip(self.lod_iter(cam_pos)) {
            // FIXME:
            let lod = chunk.get_available_lods().first()
                .copied()
                .expect("chunk mesh should be with at least one LOD");
            chunk.render(target, &draw_bundle, uniforms, lod)?
        }

        Ok(())
    }
}
