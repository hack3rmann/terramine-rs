use {
    crate::app::utils::terrain::{
        chunk::{
            Chunk,
            ChunkAdj,
            iterator::SpaceIter,
            Lod,
        },
        voxel::Voxel,
    },
    math_linear::prelude::*,
    std::{
        ptr::NonNull,
        iter::Zip,
        slice::{Iter, IterMut},
    },
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

    pub fn coord_idx_to_pos(sizes: USize3, coord_idx: USize3) -> Int3 {
        Int3::from(coord_idx) - Int3::from(sizes) / 2
    }

    pub fn pos_to_coord_idx(sizes: USize3, pos: Int3) -> Option<USize3> {
        let sizes = Int3::from(sizes);
        let shifted = pos + sizes / 2;

        match (Int3::ZERO..sizes).contains(&shifted) {
            true  => Some(shifted.into()),
            false => None
        }
    }

    pub fn coord_idx_to_idx(sizes: USize3, coord_idx: USize3) -> usize {
        sdex::get_index(&coord_idx.as_array(), &sizes.as_array())
    }

    pub fn pos_to_idx(sizes: USize3, pos: Int3) -> Option<usize> {
        Some(Self::coord_idx_to_idx(sizes, Self::pos_to_coord_idx(sizes, pos)?))
    }

    pub fn get_chunk_by_pos(&self, pos: Int3) -> Option<&Chunk> {
        Some(&self.chunks[Self::pos_to_idx(self.sizes, pos)?])
    }

    pub fn get_adj_chunks(&self, pos: Int3) -> ChunkAdj<'_> {
        let mut adj = ChunkAdj::none();
        let adjs = SpaceIter::adj_iter(pos)
            .filter_map(|off|
                Some((off, self.get_chunk_by_pos(pos + off)?))
            );

        for (offset, chunk) in adjs {
            adj.set(offset, NonNull::from(chunk)).expect("failed to set adj");
        }

        adj
    }

    pub fn pos_iter(sizes: USize3) -> SpaceIter {
        let start = Self::coord_idx_to_pos(sizes, USize3::ZERO);
        let end   = Self::coord_idx_to_pos(sizes, sizes);

        SpaceIter::new(start..end)
    }

    // TODO: make mut and shared versions of ChunkAdj
    pub fn adj_iter(&self) -> impl Iterator<Item = ChunkAdj<'_>> {
        Self::pos_iter(self.sizes)
            .map(move |pos| self.get_adj_chunks(pos))
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

    pub fn chunks_with_adj(&self) -> Zip<Iter<'_, Chunk>, impl Iterator<Item = ChunkAdj<'_>>> {
        Iterator::zip(self.chunks(), self.adj_iter())
    }

    pub fn chunks_with_adj_mut(&mut self) -> Zip<IterMut<'_, Chunk>, impl Iterator<Item = ChunkAdj<'_>>> {
        // * Safe bacause shared adjacent chunks are not aliasing current mutable chunk
        // * and the reference is made of a pointer that is made from that reference.
        let adj_iter = unsafe { NonNull::new_unchecked(self as *mut Self).as_ref() }.adj_iter();
        Iterator::zip(self.chunks_mut(), adj_iter)
    }

    pub fn generate_meshes(&mut self, lod: impl Fn(Int3) -> Lod, display: &glium::Display) {
        for (chunk, adj) in self.chunks_with_adj_mut() {
            chunk.generate_mesh(lod(chunk.pos), adj, display)
        }
    }
}