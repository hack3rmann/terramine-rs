#![allow(dead_code)]

/// IDs type.
pub type Id = u16;

/// Represents shared data for group of voxels
#[derive(Debug, PartialEq)]
pub struct VoxelData {
	pub name: &'static str,
	pub id: Id,

	pub textures: TextureSides
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
	pub const fn vertical(sides: u16, up_bottom: u16) -> Self {
		Self::new(sides, sides, sides, sides, up_bottom, up_bottom)
	}

	/// Front, up/bottom and other sides
	#[allow(dead_code)]
	pub const fn vertical_one_side(front: u16, up_bottom: u16, other_sides: u16) -> Self {
		Self::new(front, other_sides, other_sides, other_sides, up_bottom, up_bottom)
	}
}

pub const NOTHING_VOXEL_DATA:		&VoxelData = &VOXEL_DATA[0];
pub const LOG_VOXEL_DATA:			&VoxelData = &VOXEL_DATA[1];
pub const STONE_VOXEL_DATA:			&VoxelData = &VOXEL_DATA[2];

pub const VOXEL_DATA: [VoxelData; 3] = [
	VoxelData { name: "Air",					id: 0, textures: TextureSides::all(0) },
	VoxelData { name: "Log block",				id: 1, textures: TextureSides::vertical(3, 1) },
	VoxelData { name: "Stone block",			id: 2, textures: TextureSides::all(2) },
];