enum Type {
	Version,
	Dimensions,
}

fn save() {
	let save = Save::new(world_dir("world1"))
		.write(&Version::new(12, 4, 1, 4), Type::Version)
		.write(&Int3::all(1), Type::Dimensions);
}