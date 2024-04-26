pub mod voxel_data;
pub mod atlas;
pub mod generator;

use {
    crate::{
        prelude::*,
        terrain::chunk::mesh::{HiResVertex, LowResVertex},
    },
    voxel_data::{data::*, VoxelData, VoxelId},
};

/// Represents voxel.
#[derive(Debug, Clone, Copy, PartialEq, Display)]
#[display("{data.name} with id = {data.id} in {pos}")]
pub struct Voxel {
    pub data: &'static VoxelData,
    pub pos: Int3,
}

impl Voxel {
    pub const SIZE: f32 = cfg::terrain::VOXEL_SIZE;

    /// Voxel constructor.
    pub const fn new(position: Int3, data: &'static VoxelData) -> Self {
        Voxel { data, pos: position }
    }

    pub const fn is_air(&self) -> bool {
        self.data.id == AIR_VOXEL_DATA.id
    }
}

pub fn is_id_valid(id: VoxelId) -> bool {
    #![allow(unused_comparisons)]
    #![allow(clippy::absurd_extreme_comparisons)]
    let id = id as usize;
    0 <= id && id < VOXEL_DATA.len()
}

/// Generalization of voxel details.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoxelColor {
    Transparent,
    Colored(Color),
}



impl AsBytes for Voxel {
    fn as_bytes(&self) -> Vec<u8> {
        compose! {
            self.data.id.as_bytes(),
            self.pos.as_bytes(),
        }.collect()
    }
}

impl FromBytes for Voxel {
    fn from_bytes(source: &[u8]) -> Result<Self, ReinterpretError> {
        read! {
            source,
            let id: VoxelId,
            let pos,
        }

        Ok(Self { pos, data: &VOXEL_DATA[id as usize] })
    }
}

impl const StaticSize for Voxel {
    fn static_size() -> usize { 16 }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reinterpret_voxel1() {
        let before = Voxel::new(Int3::new(123, 4212, 11), STONE_VOXEL_DATA);
        let after = Voxel::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn reinterpret_voxel2() {
        let before = Voxel::new(Int3::new(-213, 4212, 11), LOG_VOXEL_DATA);
        let after = Voxel::from_bytes(&before.as_bytes()).unwrap();

        assert_eq!(before, after);
    }
}



pub mod shape {
    use {
        super::{*, atlas::UV},
        cfg::terrain::{
            BACK_IDX, FRONT_IDX, RIGHT_IDX, LEFT_IDX, TOP_IDX, BOTTOM_IDX,
        },
    };

    #[derive(Debug)]
    pub struct CubeDetailed<'c> {
        data: &'c VoxelData,
        half_size: f32,
    }

    #[derive(Debug)]
    pub struct CubeLowered {
        half_size: f32,
    }

    impl<'c> CubeDetailed<'c> {
        /// Constructs new cube maker with filled voxel data.
        pub fn new(data: &'c VoxelData) -> Self {
            Self { data, half_size: Voxel::SIZE * 0.5 }
        }

        /// Edit default size.
        pub fn size(mut self, new_size: f32) -> Self {
            self.half_size = new_size * 0.5;
            self
        }

        pub fn by_offset<const N: usize>(&self, offset: Int3, position: vec3, vertices: &mut SmallVec<[HiResVertex; N]>) {
            let position = 2.0 * self.half_size * position;
            match offset.as_tuple() {
                ( 1,  0,  0) => self.back(position, vertices),
                (-1,  0,  0) => self.front(position, vertices),
                ( 0,  1,  0) => self.top(position, vertices),
                ( 0, -1,  0) => self.bottom(position, vertices),
                ( 0,  0,  1) => self.right(position, vertices),
                ( 0,  0, -1) => self.left(position, vertices),
                _ => panic!("There's no offset {:?}", offset),
            }
        }

        /// Cube front face vertex array.
        pub fn front<const N: usize>(&self, position: vec3, vertices: &mut SmallVec<[HiResVertex; N]>) {
            /* UVs for front face */
            let uv = UV::new(self.data.textures.front);
            
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = FRONT_IDX as u32;
            let hs = self.half_size;

            vertices.push(HiResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), vecf!(uv.hi.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), vecf!(uv.hi.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), vecf!(uv.lo.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), vecf!(uv.hi.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), vecf!(uv.lo.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x, -hs + y,  hs + z), vecf!(uv.lo.x, uv.hi.y), face_idx));
        }

        /// Cube back face vertex array.
        pub fn back<const N: usize>(&self, position: vec3, vertices: &mut SmallVec<[HiResVertex; N]>) {
            /* UVs for back face */
            let uv = UV::new(self.data.textures.back);
            
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = BACK_IDX as u32;
            let hs = self.half_size;

            vertices.push(HiResVertex::new(vecf!(hs + x, -hs + y, -hs + z), vecf!(uv.lo.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(hs + x, -hs + y,  hs + z), vecf!(uv.hi.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(hs + x,  hs + y,  hs + z), vecf!(uv.hi.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(hs + x, -hs + y, -hs + z), vecf!(uv.lo.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(hs + x,  hs + y,  hs + z), vecf!(uv.hi.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(hs + x,  hs + y, -hs + z), vecf!(uv.lo.x, uv.lo.y), face_idx));
        }

        /// Cube top face vertex array.
        pub fn top<const N: usize>(&self, position: vec3, vertices: &mut SmallVec<[HiResVertex; N]>) {
            /* UVs for top face */
            let uv = UV::new(self.data.textures.top);
            
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = TOP_IDX as u32;
            let hs = self.half_size;

            vertices.push(HiResVertex::new(vecf!( hs + x,  hs + y, -hs + z), vecf!(uv.lo.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!( hs + x,  hs + y,  hs + z), vecf!(uv.hi.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), vecf!(uv.lo.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), vecf!(uv.lo.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!( hs + x,  hs + y,  hs + z), vecf!(uv.hi.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), vecf!(uv.hi.x, uv.lo.y), face_idx));
        }

        /// Cube bottom face vertex array.
        pub fn bottom<const N: usize>(&self, position: vec3, vertices: &mut SmallVec<[HiResVertex; N]>) {
            /* UVs for bottom face */
            let uv = UV::new(self.data.textures.bottom);
            
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = BOTTOM_IDX as u32;
            let hs = self.half_size;

            vertices.push(HiResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), vecf!(uv.lo.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!( hs + x, -hs + y,  hs + z), vecf!(uv.hi.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!( hs + x, -hs + y, -hs + z), vecf!(uv.lo.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), vecf!(uv.lo.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x, -hs + y,  hs + z), vecf!(uv.hi.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!( hs + x, -hs + y,  hs + z), vecf!(uv.hi.x, uv.hi.y), face_idx));
        }

        /// Cube left face vertex array.
        pub fn left<const N: usize>(&self, position: vec3, vertices: &mut SmallVec<[HiResVertex; N]>) {
            /* UVs for left face */
            let uv = UV::new(self.data.textures.left);
            
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = LEFT_IDX as u32;
            let hs = self.half_size;

            vertices.push(HiResVertex::new(vecf!( hs + x, -hs + y, -hs + z), vecf!(uv.lo.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!( hs + x,  hs + y, -hs + z), vecf!(uv.lo.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), vecf!(uv.hi.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!( hs + x, -hs + y, -hs + z), vecf!(uv.lo.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), vecf!(uv.hi.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), vecf!(uv.hi.x, uv.hi.y), face_idx));
        }

        /// Cube right face vertex array.
        pub fn right<const N: usize>(&self, position: vec3, vertices: &mut SmallVec<[HiResVertex; N]>) {
            /* UVs for right face */
            let uv = UV::new(self.data.textures.right);
            
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = RIGHT_IDX as u32;
            let hs = self.half_size;

            vertices.push(HiResVertex::new(vecf!( hs + x, -hs + y,  hs + z), vecf!(uv.lo.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), vecf!(uv.hi.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!( hs + x,  hs + y,  hs + z), vecf!(uv.lo.x, uv.lo.y), face_idx));
            vertices.push(HiResVertex::new(vecf!( hs + x, -hs + y,  hs + z), vecf!(uv.lo.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x, -hs + y,  hs + z), vecf!(uv.hi.x, uv.hi.y), face_idx));
            vertices.push(HiResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), vecf!(uv.hi.x, uv.lo.y), face_idx));
        }

        /// Cube all sides.
        #[allow(dead_code)]
        pub fn all<const N: usize>(&self, position: vec3, vertices: &mut SmallVec<[HiResVertex; N]>) {
            self.left(position, vertices);
            self.right(position, vertices);
            self.front(position, vertices);
            self.back(position, vertices);
            self.top(position, vertices);
            self.bottom(position, vertices);
        }
    }

    impl CubeLowered {
        pub fn new(size: f32) -> Self {
            Self { half_size: 0.5 * size }
        }

        pub fn by_offset(&self, offset: Int3, position: vec3, color: Color, vertices: &mut Vec<LowResVertex>) {
            match offset.as_tuple() {
                ( 1,  0,  0) => self.back(position, color, vertices),
                (-1,  0,  0) => self.front(position, color, vertices),
                ( 0,  1,  0) => self.top(position, color, vertices),
                ( 0, -1,  0) => self.bottom(position, color, vertices),
                ( 0,  0,  1) => self.right(position, color, vertices),
                ( 0,  0, -1) => self.left(position, color, vertices),
                _ => panic!("There's no offset {:?}", offset),
            }
        }

        /// Cube front face vertex array.
        pub fn front(&self, position: vec3, color: Color, vertices: &mut Vec<LowResVertex>) {
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = FRONT_IDX as u32;
            let hs = self.half_size;

            vertices.push(LowResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x, -hs + y,  hs + z), color, face_idx));
        }

        /// Cube back face vertex array.
        pub fn back(&self, position: vec3, color: Color, vertices: &mut Vec<LowResVertex>) {
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = BACK_IDX as u32;
            let hs = self.half_size;

            vertices.push(LowResVertex::new(vecf!(hs + x, -hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(hs + x, -hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(hs + x,  hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(hs + x, -hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(hs + x,  hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(hs + x,  hs + y, -hs + z), color, face_idx));
        }

        /// Cube top face vertex array.
        pub fn top(&self, position: vec3, color: Color, vertices: &mut Vec<LowResVertex>) {
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = TOP_IDX as u32;
            let hs = self.half_size;

            vertices.push(LowResVertex::new(vecf!( hs + x,  hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!( hs + x,  hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!( hs + x,  hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), color, face_idx));
        }

        /// Cube bottom face vertex array.
        pub fn bottom(&self, position: vec3, color: Color, vertices: &mut Vec<LowResVertex>) {
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = BOTTOM_IDX as u32;
            let hs = self.half_size;

            vertices.push(LowResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!( hs + x, -hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!( hs + x, -hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x, -hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!( hs + x, -hs + y,  hs + z), color, face_idx));
        }

        /// Cube left face vertex array.
        pub fn left(&self, position: vec3, color: Color, vertices: &mut Vec<LowResVertex>) {
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = LEFT_IDX as u32;
            let hs = self.half_size;

            vertices.push(LowResVertex::new(vecf!( hs + x, -hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!( hs + x,  hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!( hs + x, -hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y, -hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x, -hs + y, -hs + z), color, face_idx));
        }

        /// Cube right face vertex array.
        pub fn right(&self, position: vec3, color: Color, vertices: &mut Vec<LowResVertex>) {
            /* Shortcuts */
            let (x, y, z) = position.as_tuple();
            let face_idx = RIGHT_IDX as u32;
            let hs = self.half_size;

            vertices.push(LowResVertex::new(vecf!( hs + x, -hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!( hs + x,  hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!( hs + x, -hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x, -hs + y,  hs + z), color, face_idx));
            vertices.push(LowResVertex::new(vecf!(-hs + x,  hs + y,  hs + z), color, face_idx));
        }

        /// Cube all sides.
        #[allow(dead_code)]
        pub fn all(&self, position: vec3, color: Color, vertices: &mut Vec<LowResVertex>) {
            self.left(position, color, vertices);
            self.right(position, color, vertices);
            self.front(position, color, vertices);
            self.back(position, color, vertices);
            self.top(position, color, vertices);
            self.bottom(position, color, vertices);
        }}
}
