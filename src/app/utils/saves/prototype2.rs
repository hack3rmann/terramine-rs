enum Type {
	Version,
	Dimensions,
}

impl Into<u64> for Type {
	fn into(self) -> u64 { self as u64 }
}

fn save() {
	let save = Save::new(world_dir("world1"))
		.write(&Version::new(12, 4, 1, 4), Type::Version)
		.write(&Int3::all(1), Type::Dimensions);
}