/// Represents shared data for group of voxels
#[allow(dead_code)]
pub struct VoxelData {
	name: &'static str,
	id: u32,

	textures: TextureSides
}

/// Represents textured sides of the voxel.
#[allow(dead_code)]
pub struct TextureSides {
	front:	u16,
	back:	u16,
	left:	u16,
	right:	u16,
	up:		u16,
	bottom:	u16
}

impl TextureSides {
	/// Constructs new voxel sides data
	pub const fn new(front: u16, back: u16, left: u16, right: u16, up: u16, bottom: u16) -> Self {
		TextureSides { front, back, left, right, up, bottom }
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

pub static GRASS_VOXEL_DATA: VoxelData = VoxelData { name: "Grass block", id: 1, textures: TextureSides::all(0) };