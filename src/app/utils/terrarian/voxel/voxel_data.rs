#[allow(dead_code)]
pub struct VoxelData {
	name: &'static str,
	id: u32,
}

pub(super) static FIRST_VOXEL_DATA: VoxelData = VoxelData { name: "First ever!", id: 0 };