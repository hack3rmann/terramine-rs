use crate::prelude::*;



/// IDs type.
pub type VoxelId = u16;



/// Represents shared data for group of voxels
#[derive(Debug, PartialEq)]
pub struct VoxelData {
    pub name: &'static str,
    pub id: VoxelId,

    pub textures: TextureSides,
    pub avarage_color: Vec3,
}
assert_impl_all!(VoxelData: Send, Sync);

impl VoxelData {
    pub const fn new(name: &'static str, id: VoxelId, color: Vec3, textures: TextureSides) -> Self {
        Self { name, id, textures, avarage_color: color }
    }
}



/// Represents textured sides of the voxel.
#[derive(Debug, PartialEq)]
pub struct TextureSides {
    pub front:	u16,
    pub back:	u16,
    pub left:	u16,
    pub right:	u16,
    pub top:	u16,
    pub bottom:	u16
}

impl TextureSides {
    /// Constructs new voxel sides data
    pub const fn new(front: u16, back: u16, left: u16, right: u16, up: u16, bottom: u16) -> Self {
        TextureSides { front, back, left, right, top: up, bottom }
    }

    /// Makes all sides to given id
    pub const fn all(id: u16) -> Self {
        Self::new(id, id, id, id, id, id)
    }

    /// Sides and up/bottom
    #[allow(dead_code)]
    pub const fn vertical(sides: VoxelId, top: VoxelId, bottom: VoxelId) -> Self {
        Self::new(sides, sides, sides, sides, top, bottom)
    }

    /// Front, up/bottom and other sides
    #[allow(dead_code)]
    pub const fn vertical_one_side(front: u16, up_bottom: u16, other_sides: u16) -> Self {
        Self::new(front, other_sides, other_sides, other_sides, up_bottom, up_bottom)
    }
}



pub mod data {
    use super::*;
    
    pub use crate::cfg::terrain::voxel_types::VOXEL_DATA;
    
    pub const AIR_VOXEL_DATA:    		&VoxelData = &VOXEL_DATA[0];
    pub const LOG_VOXEL_DATA:			&VoxelData = &VOXEL_DATA[1];
    pub const STONE_VOXEL_DATA:			&VoxelData = &VOXEL_DATA[2];
    pub const GRASS_VOXEL_DATA:         &VoxelData = &VOXEL_DATA[3];
    pub const DIRT_VOXEL_DATA:          &VoxelData = &VOXEL_DATA[4];
}