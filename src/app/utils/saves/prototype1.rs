fn save() {
	let mut save = Save::new(world_dir("world"))
		.write(SaveVersion::new(1, 3, 1, 4))
		.write(&Int3::all(1))
		.pointer(&Chunk::new())
		.array(CHUNK_ARRAY_VOLUME, |i| &CHUNKS[i])
		.dynamic(&Chunk::new(), ChunkState::Full as u64)
		.dyn_array(CHUNK_ARRAY_VOLUME, |i| (
			ChunkState::AllSame as u64,
			&ChunkFiller::new(position_function(i)) as &dyn Reinterpret
		));

	let sv: DynArray<Dynamic<Array<Pointer<Write<Write<EmptySave, SaveVersion>, Int3>, Chunk>, Chunk>, Chunk>, impl Fn(usize) -> &dyn Reinterpret>;
}