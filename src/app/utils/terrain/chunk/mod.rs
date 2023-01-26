pub mod chunk_array;
pub mod iterator;

use {
	crate::app::utils::{
		cfg,
		werror::prelude::*,
		math::prelude::*,
		graphics::{
			Graphics,
			mesh::{Mesh, UnindexedMesh},
			shader::Shader,
			vertex_buffer::VertexBuffer,
			camera::Camera,
		},
		reinterpreter::*,
	},
	super::voxel::{
		Voxel,
		LoweredVoxel,
		shape::{CubeDetailed, CubeLowered},
		voxel_data::*,
		generator,
	},
	glium::{
		DrawError,
		uniforms::Uniforms,
		Frame,
		index::PrimitiveType,
		DrawParameters,
		implement_vertex,
	},
	std::{
		cell::RefCell, marker::PhantomData,
		borrow::Cow, num::NonZeroU32, fmt::Display
	},
	iterator::{CubeBorder, SpaceIter, Sides},
};

/// Full-detailed vertex.
#[derive(Copy, Clone)]
pub struct DetailedVertex {
	pub position: [f32; 3],
	pub tex_coords: [f32; 2],
	pub light: f32
}

/// Low-detailed vertex.
#[derive(Copy, Clone)]
pub struct LoweredVertex {
	pub position: [f32; 3],
	pub color: [f32; 3],
	pub light: f32
}

/* Implement Vertex structs as glium intended */
implement_vertex!(DetailedVertex, position, tex_coords, light);
implement_vertex!(LoweredVertex, position, color, light);

/**
 * Index cheatsheet!
 * 
 * i = d(hx + y) + z
 * 
 * x = (i / d) / h
 * y = (i / d) % h
 * z = i % d
 */

/// Type of voxel array. May be something different during progress.
type VoxelArray = Vec<Id>;

pub enum FindOptions {
	OutOfBounds,
	InChunkNothing,
	InChunkDetailed(Voxel),
	InChunkLowered(Voxel, NonZeroU32),
}

pub enum ChunkOptional<Item> {
	OutsideChunk,
	Item(Item, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChunkFill {
	#[default]
	Standart,

	Empty,
	All(Id),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChunkDetails {
	#[default]
	Full,

	/// Low details with factor that represents how much chunk devided by 2.
	/// It means that side of chunk is devided by 2^factor.
	Low(NonZeroU32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChunkDetailsError {
	NoMoreDivisible { size: usize, level: u32 },
	FirstDivisionFailed { level: u32 },
	NotEnoughInformation,
}

impl Display for ChunkDetailsError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Self::NoMoreDivisible { size, level } =>
				write!(f, "Chunk can not be lowered: chunk size is {size}, but detail level is {level}!"),
			Self::FirstDivisionFailed { level } =>
				write!(f, "First division failed: chunk size is {size}, but detail level is {level}!", size = MeshlessChunk::SIZE),
			Self::NotEnoughInformation =>
				write!(f, "Not enougth information passed!"),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Addition {
	#[default]
	NothingToKnow,

	Know {
		fill: Option<ChunkFill>,
		details: ChunkDetails,
	},
}

impl AsRef<Self> for Addition {
	fn as_ref(&self) -> &Self { self }
}

/// Chunk struct.
pub struct MeshlessChunk {
	pub voxel_ids: VoxelArray,
	pub pos: Int3,
	pub additional_data: Addition,
}

impl MeshlessChunk {
	/// Predefined chunk values.
	pub const SIZE:	  usize = cfg::terrain::CHUNK_SIZE;
	pub const VOLUME: usize = Self::SIZE.pow(3);

	/// Constructs new chunk in given position 
	pub fn new(pos: Int3) -> Self {
		/* Voxel array initialization */
		let mut voxels = VoxelArray::with_capacity(Self::VOLUME);

		/* Additional data */
		let mut fill = ChunkFill::Empty;

		/* Iterating in the chunk */
		for curr in Self::pos_iter() {
			let global_pos = pos_in_chunk_to_world_int3(curr, pos);

			/* Update addidional chunk data */
			let mut update_data = |id| {
				match fill {
					ChunkFill::Empty => {
						/* Check for first iteration */
						fill = if curr != Int3::zero() {
							ChunkFill::Standart
						} else {
							ChunkFill::All(id)
						}
					},
					ChunkFill::All(all_id) => {
						if all_id != id {
							fill = ChunkFill::Standart;
						}
					}
					ChunkFill::Standart => (),
				}
			};

			/* Kind of trees generation */
			if generator::trees(global_pos) {
				let id = LOG_VOXEL_DATA.id;
				update_data(id);
				voxels.push(id);
			}
			
			/* Sine-like floor */
			else if generator::sine(global_pos) {
				let id = STONE_VOXEL_DATA.id;
				update_data(id);
				voxels.push(id);
			}
			
			/* Air */
			else {
				if let ChunkFill::All(_) = fill {
					fill = ChunkFill::Standart
				}

				voxels.push(AIR_VOXEL_DATA.id)
			}
		}

		/* Chunk is empty so array can be empty */
		if let ChunkFill::Empty   = fill { voxels = vec![  ] }
		if let ChunkFill::All(id) = fill { voxels = vec![id] }
		
		// FIXME: Remove this:
		let mut chunk = MeshlessChunk {
			voxel_ids: voxels, pos,
			additional_data: Addition::Know {
				fill: Some(fill), details: ChunkDetails::Full,
			}
		};
		let abs = Float4::from(pos).abs() as u32;
		if abs >= 1 {
			chunk.lower_detail(abs.min((Self::SIZE as f32).log2() as u32)).wunwrap();
		}
		return chunk
	}

	/// Constructs new empty chunk.
	pub fn new_empty(pos: Int3) -> Self {
		Self {
			pos, voxel_ids: vec![],
			additional_data: Addition::Know {
				fill: Some(ChunkFill::Empty),
				details: ChunkDetails::Full
			}
		}
	}

	/// Constructs new filled chunk.
	pub fn new_filled(pos: Int3, id: Id) -> Self {
		Self {
			pos, voxel_ids: vec![id],
			additional_data: Addition::Know {
				fill: Some(ChunkFill::All(id)),
				details: ChunkDetails::Full
			}
		}
	}

	/// Lowers details of chunk to given value.
	pub fn lower_detail(&mut self, level: u32) -> Result<(), ChunkDetailsError> {
		/* Do nothing if level is zero */
		//* Also, this required by Safety arguments below.
		if level == 0 { return Ok(()) }

		//* Safety: this operation is safe until level is not zero.
		//* In previous step level had been checked so it is not zero.
		let level = unsafe { NonZeroU32::new_unchecked(level) };

		match self.additional_data.as_ref() {
			/* First division */
			Addition::Know { details: ChunkDetails::Full, fill } => {
				/* Check that chunk can be lowered */
				if Self::SIZE as u32 % 2_u32.pow(level.get()) != 0 {
					return Err(ChunkDetailsError::FirstDivisionFailed { level: level.get() })
				}

				self.additional_data = Addition::Know { details: ChunkDetails::Low(level), fill: *fill };
			},

			/* Further more divisions */
			Addition::Know { details: ChunkDetails::Low(old_level), fill } => {
				/* Calculate old chunk size */
				let old_size = Self::SIZE / 2_usize.pow(old_level.get());

				/* Check that chunk can be lowered */
				if old_size as u32 % 2_u32.pow(level.get()) != 0 {
					return Err(ChunkDetailsError::NoMoreDivisible { size: old_size, level: level.get() })
				}

				//* Safety: this operation is safe until level is not zero.
				//* Given value is not zero because it is the sum of positive values.
				let new_level = unsafe { NonZeroU32::new_unchecked(old_level.get() + level.get()) };

				self.additional_data = Addition::Know { details: ChunkDetails::Low(new_level), fill: *fill };
			}
			
			/* Nothing presented */
			Addition::NothingToKnow => return Err(ChunkDetailsError::NotEnoughInformation),
		}

		return Ok(())
	}

	/// Checks if chunk is empty by its data.
	pub fn is_empty(&self) -> bool {
		match self.additional_data {
			Addition::Know { fill: Some(ChunkFill::Empty), .. } => true,
			_ => false,
		}
	}

	/// Checks if chunk is empty by its data.
	pub fn is_filled(&self) -> bool {
		match self.additional_data {
			Addition::Know { fill: Some(ChunkFill::All(_)), .. } => true,
			_ => false,
		}
	}

	/// Gives fill id if available.
	pub fn fill_id(&self) -> Option<Id> {
		if let Addition::Know { fill: Some(ChunkFill::All(id)), .. } = self.additional_data {
			Some(id)
		} else { None }
	}

	pub fn make_full_detailed_vertices_filled(&self, env: &ChunkEnvironment, id: Id) -> DetailedVertexVec {
		/* Cycle over all coordinates in chunk */
		let mut vertices = vec![];
		for pos in CubeBorder::new(Self::SIZE as i32) {
			self.to_triangles_inner_detailed(pos, id, env, &mut vertices);
		}

		/* Shrink vector */
		vertices.shrink_to_fit();

		return Full(vertices)
	}

	pub fn make_full_detailed_vertices_standart(&self, env: &ChunkEnvironment) -> DetailedVertexVec {
		/* Construct vertex array */
		let mut vertices = vec![];

		let iter = self.voxel_ids.iter()
			.enumerate()
			.map(|(i, &id)|
				return (position_function(i), id)
			);

		for (pos, id) in iter {
			self.to_triangles_inner_detailed(pos, id, env, &mut vertices);
		}

		/* Shrink vector */
		vertices.shrink_to_fit();

		return Full(vertices)
	}

	pub fn make_lowered_detail_vertices_filled(&self, env: &ChunkEnvironment, lod: NonZeroU32) -> DetailedVertexVec {
		let mut vertices = vec![];
		let lod_size = 2_usize.pow(lod.get()) as i32;
		let outer_size = Self::SIZE as i32 / lod_size;

		/* Go for all sub-chunks in this chunk */
		for outer in SpaceIter::zeroed_cubed(outer_size) {
			let mut avarage_voxel = LoweredVoxel::Transparent;
			let mut avarage_count: usize = 0;
			
			let outer_bordered = iterator::is_bordered(outer, Int3::zero()..Int3::all(outer_size));
			let mut blocked = Sides::all(outer_bordered);

			/* Go for all voxels in lod-subchunks */
			for inner in SpaceIter::zeroed_cubed(lod_size) {
				let (pos, ref new_color, id) = {
					let pos = outer * lod_size + inner;
					let world = pos_in_chunk_to_world_int3(pos, self.pos);
					let i  = pos_to_idx(pos);
					let id = self.voxel_ids[i];

					(world, VOXEL_DATA[id as usize].avarage_color, id)
				};

				/* Poses for different sides */
				let back_pos   = pos - Int3::new(-1,  0,  0);
				let front_pos  = pos - Int3::new( 1,  0,  0);
				let top_pos    = pos - Int3::new( 0, -1,  0);
				let bottom_pos = pos - Int3::new( 0, -1,  0);
				let right_pos  = pos - Int3::new( 0,  0, -1);
				let left_pos   = pos - Int3::new( 0,  0, -1);

				/* Compute color of low-res voxel */
				match avarage_voxel {
					/* For first color */
					LoweredVoxel::Transparent if id != AIR_VOXEL_DATA.id => {
						avarage_voxel = LoweredVoxel::Colored(*new_color);
						avarage_count = 1;
					},

					/* If no color had been and no more given */
					LoweredVoxel::Transparent => continue,

					/* For next color */
					LoweredVoxel::Colored(ref mut color) => {
						/* Add new color */
						let [r, g, b] = color;
						*r += new_color[0];
						*g += new_color[1];
						*b += new_color[2];

						/* Increment counter */
						avarage_count += 1;
					},
				}

				/* Compute bloking of sides */
				//?! For back
				match self.get_voxel_optional(back_pos) {
					/* Inside-chunk case. Nothing to do with the border. */
					FindOptions::InChunkNothing if !outer_bordered => {
						*blocked.back_mut() = false;
					},

					// TODO: Other checks.

					_ => todo!()
				}
			}

			/* Divide voxel color by level count */
			if let LoweredVoxel::Colored(ref mut color) = avarage_voxel {
				let [r, g, b] = color;
				*r /= avarage_count as f32;
				*g /= avarage_count as f32;
				*b /= avarage_count as f32;
			}
			
			/* Cube sampler */
			let cube = CubeLowered::new(lod_size as f32);

			/* Calculate center position of low-res cube */
			let pos = {
				let entire_min = outer * lod_size;
				let world = pos_in_chunk_to_world_int3(entire_min, self.pos);

				Float4::from(world) + Float4::all(0.5 * cfg::terrain::VOXEL_SIZE)
			};

			/* Add vertices if side is not blocked */
			if let LoweredVoxel::Colored(color) = avarage_voxel {
				if !blocked.back()   { cube.back  (pos, color, &mut vertices) }
				if !blocked.front()  { cube.front (pos, color, &mut vertices) }
				if !blocked.top()    { cube.top   (pos, color, &mut vertices) }
				if !blocked.bottom() { cube.bottom(pos, color, &mut vertices) }
				if !blocked.right()  { cube.right (pos, color, &mut vertices) }
				if !blocked.left()   { cube.left  (pos, color, &mut vertices) }
			}
		}

		return Low(vertices)
	}

	pub fn make_lowered_detail_vertices_standart(&self, env: &ChunkEnvironment, lod: NonZeroU32) -> DetailedVertexVec {
		/* Resulting vertex array */
		let mut vertices = vec![];
		
		/* Side length of low-res voxel in world units */
		let voxel_size = 2_usize.pow(lod.get());
		
		/* Side length of low-res chunk in low-res voxels */
		let low_side_len = Self::SIZE / voxel_size;

		/* Array of low-res voxels */
		let lowered = self.get_lowered_voxels(lod).wunwrap();

		let iter = lowered.iter()
			.enumerate()
			.map(|(i, lowered_voxel)| {
				use cfg::terrain::VOXEL_SIZE;

				/* Find `position` in 'smaller chunk' */
				let pos_low = general_position(i, low_side_len, low_side_len);
				let mut pos = Float4::from(pos_low);

				/* Then stretch it and center */
				pos += Float4::all(VOXEL_SIZE * 0.5);
				pos *= voxel_size as f32;
				pos -= Float4::all(VOXEL_SIZE * 0.5);

				/* Move to world coordinates */
				let pos = pos_in_chunk_to_world_float4(pos, self.pos.into());

				/* Return lowered chunk position in world, 
				|* position in lowered array and lowered chunk itself */
				return (pos, pos_low, lowered_voxel)
			});

		/* Goes for each low-res voxel and generates its mesh */
		for (pos, pos_low, low) in iter {
			//self.to_triangles_inner_lowered_deprecated(pos, pos_low, voxel_size, low, env, &mut vertices);
			self.to_triangles_inner_lowered(pos, pos_low, lod, low, env, &mut vertices);
		}

		return Low(vertices)
	}

	/// Creates trianlges Vec from Chunk and its environment.
	pub fn to_triangles(&self, env: &ChunkEnvironment) -> DetailedVertexVec {
		match self.additional_data.as_ref() {
			/* Empty chunk passed in */
			Addition::Know { fill: Some(ChunkFill::Empty), details } => return match details {
				ChunkDetails::Full => Full(vec![]),
				ChunkDetails::Low(_) => Low(vec![]),
			},

			/* "Filled" chunk with full details passed in */
			Addition::Know { fill: Some(ChunkFill::All(id)), details: ChunkDetails::Full } =>
				return self.make_full_detailed_vertices_filled(env, *id),

			/* Standart chunk with full details passed in */
			Addition::Know { fill: Some(ChunkFill::Standart), details: ChunkDetails::Full } =>
				self.make_full_detailed_vertices_standart(env),

			/* Lowered details uniplemented */
			// TODO: Separate filled and standart chunk impl.
			Addition::Know { details: ChunkDetails::Low(lod), fill: _ } =>
				self.make_lowered_detail_vertices_standart(env, *lod),

			/* Lowered details uniplemented */
			// TODO: Separate filled and standart chunk impl.
			Addition::Know { details: ChunkDetails::Low(lod), fill: Some(ChunkFill::Standart) } =>
				self.make_lowered_detail_vertices_filled(env, *lod),

			Addition::Know { details: ChunkDetails::Low(lod), fill: Some(ChunkFill::All(id)) } =>
				return todo!("Filled low-res impl"),

			/* Not enough information */
			not_enough @ Addition::NothingToKnow | not_enough @ Addition::Know { fill: None, .. } =>
				panic!("No required information passed into mesh builder! {addition:?}", addition = not_enough),
		}
	}

	fn to_triangles_inner_detailed(&self, in_chunk_pos: Int3, id: Id, env: &ChunkEnvironment, vertices: &mut Vec<DetailedVertex>) {
		if id == AIR_VOXEL_DATA.id { return }
		
		/* Get position from index */
		let position = pos_in_chunk_to_world_int3(in_chunk_pos, self.pos);
		
		/* Draw checker */
		let is_drawable = |input: Option<Voxel>| -> bool {
			match input {
				None => true,
				Some(voxel) => voxel.data == AIR_VOXEL_DATA,
			}
		};
		
		/* Mesh builder */
		let build = |bias, env: Option<*const MeshlessChunk>| {
			if is_drawable(self.get_voxel_or_none(position + bias)) {
				match env {
					None => true,
					Some(chunk) =>
					//* Safety: Safe because environment chunks lives as long as other chunks or that given chunk.
					//* And it also needs only at chunk generation stage.
					is_drawable(unsafe { chunk.as_ref().wunwrap().get_voxel_or_none(position + bias) }),
				}
			} else { false }
		};

		/* Cube vertices generator */
		let cube = CubeDetailed::new(&VOXEL_DATA[id as usize]);
		
		/* Build all sides separately */
		if build(Int3::new( 1,  0,  0), env.back)   { cube.back  (position, vertices) };
		if build(Int3::new(-1,  0,  0), env.front)  { cube.front (position, vertices) };
		if build(Int3::new( 0,  1,  0), env.top)    { cube.top   (position, vertices) };
		if build(Int3::new( 0, -1,  0), env.bottom) { cube.bottom(position, vertices) };
		if build(Int3::new( 0,  0,  1), env.right)  { cube.right (position, vertices) };
		if build(Int3::new( 0,  0, -1), env.left)   { cube.left  (position, vertices) };
	}

	fn is_blocked_with_lod(&self, low_pos: Int3, side: Int3, lod: NonZeroU32, neighbor: Option<*const MeshlessChunk>) -> bool {
		let voxel_size = 2_usize.pow(lod.get()) as i32;
		let neighbor_min_pos = (low_pos + side) * voxel_size;
		let neighbor_max_pos = (low_pos + side) * voxel_size + Int3::all(voxel_size);
		
		let neighbor_lod = {
			let neighbor_min_world_pos = pos_in_chunk_to_world_int3(neighbor_min_pos, self.pos);
			match self.get_voxel(neighbor_min_world_pos) {
				ChunkOptional::Item(_, lod) => lod,
				ChunkOptional::OutsideChunk => match neighbor {
					/* No voxel found means no LOD */
					None => 0,

					// TODO: add safety argument.
					Some(chunk) => match unsafe { chunk.as_ref() }.unwrap().get_voxel(neighbor_min_world_pos) {
						ChunkOptional::Item(_, lod) => lod,
						ChunkOptional::OutsideChunk => 0,
					},
				},
			}
		};

		let small_blocked = |pos: Int3| -> bool {
			match self.get_voxel(pos) {
				ChunkOptional::Item(voxel, _) => voxel.data != AIR_VOXEL_DATA,
				ChunkOptional::OutsideChunk => match neighbor {
					None => false,

					Some(chunk) => match unsafe { chunk.as_ref() }.unwrap().get_voxel(pos) {
						ChunkOptional::Item(voxel, _) => voxel.data != AIR_VOXEL_DATA,
						ChunkOptional::OutsideChunk => false,
					},
				}
			}
		};
		
		let mut pos_iter = SpaceIter::new(neighbor_min_pos..neighbor_max_pos)
			.map(|pos| pos_in_chunk_to_world_int3(pos, self.pos));

		if lod.get() <= neighbor_lod {
			pos_iter.any(small_blocked)
		} else {
			pos_iter.all(small_blocked)
		}
	}

	fn to_triangles_inner_lowered(
		&self, build_pos: Float4, low_pos: Int3, lod: NonZeroU32,
		low_voxel: &LoweredVoxel, env: &ChunkEnvironment, vertices: &mut Vec<LoweredVertex>
	) {
		let voxel_color = match low_voxel {
			LoweredVoxel::Transparent => return,
			LoweredVoxel::Colored(color) => *color,
		};

		let voxel_size_f32 = cfg::terrain::VOXEL_SIZE * 2_usize.pow(lod.get()) as f32;
		let cube_mesher = CubeLowered::new(voxel_size_f32);

		if !self.is_blocked_with_lod(low_pos, Int3::new( 1,  0,  0), lod, env.back)   { cube_mesher  .back(build_pos, voxel_color, vertices); }
		if !self.is_blocked_with_lod(low_pos, Int3::new(-1,  0,  0), lod, env.front)  { cube_mesher .front(build_pos, voxel_color, vertices); }
		if !self.is_blocked_with_lod(low_pos, Int3::new( 0,  1,  0), lod, env.top)    { cube_mesher   .top(build_pos, voxel_color, vertices); }
		if !self.is_blocked_with_lod(low_pos, Int3::new( 0, -1,  0), lod, env.bottom) { cube_mesher.bottom(build_pos, voxel_color, vertices); }
		if !self.is_blocked_with_lod(low_pos, Int3::new( 0,  0,  1), lod, env.right)  { cube_mesher .right(build_pos, voxel_color, vertices); }
		if !self.is_blocked_with_lod(low_pos, Int3::new( 0,  0, -1), lod, env.left)   { cube_mesher  .left(build_pos, voxel_color, vertices); }
	}

	#[deprecated]
	fn to_triangles_inner_lowered_deprecated(
		&self, build_pos: Float4, low_pos: Int3, voxel_size: usize,
		lowered: &LoweredVoxel, env: &ChunkEnvironment, vertices: &mut Vec<LoweredVertex>
	) {
		/* Cube mesh builder */
		let cube = CubeLowered::new(voxel_size as f32);

		// TODO: Optimize this:

		if let LoweredVoxel::Colored(color) = lowered {
			let size = voxel_size as i32;

			let is_blocked = |pos: Int3, bias: Int3, env: Option<*const MeshlessChunk>| -> bool {
				match self.get_voxel_optional(pos + bias) {
					FindOptions::OutOfBounds => {
						match env {
							None => false,
							Some(chunk) => {
								// * Safety: Safe because environment chunks lives as long as other chunks or that given chunk.
								// * And it also needs only at chunk generation stage.
								match unsafe { chunk.as_ref().wunwrap().get_voxel_optional(pos + bias) } {
									FindOptions::InChunkNothing => false,
									FindOptions::OutOfBounds => unreachable!(),
									FindOptions::InChunkDetailed(voxel) | FindOptions::InChunkLowered(voxel, _) =>
										voxel.data != AIR_VOXEL_DATA,
								}
							},
						}
					},
					FindOptions::InChunkNothing => false,
					FindOptions::InChunkDetailed(voxel) | FindOptions::InChunkLowered(voxel, _) =>
						voxel.data != AIR_VOXEL_DATA,
				}
			};

			let is_border =
				low_pos.x() == 0 || low_pos.x() == (Self::SIZE / voxel_size) as i32 - 1 ||
				low_pos.y() == 0 || low_pos.y() == (Self::SIZE / voxel_size) as i32 - 1 ||
				low_pos.z() == 0 || low_pos.z() == (Self::SIZE / voxel_size) as i32 - 1;

			if !is_border {
				let mut back_free   = true;
				let mut front_free  = true;
				let mut top_free    = true;
				let mut bottom_free = true;
				let mut right_free  = true;
				let mut left_free   = true;

				for curr in SpaceIter::new(Int3::zero()..Int3::all(size)) {
					let high_pos = low_pos * size + curr;
					let world_pos = pos_in_chunk_to_world_int3(high_pos, self.pos);

					if is_blocked(world_pos, Int3::new( size, 0, 0), env.back)   { back_free   = false }
					if is_blocked(world_pos, Int3::new(-size, 0, 0), env.front)  { front_free  = false }
					if is_blocked(world_pos, Int3::new(0,  size, 0), env.top)    { top_free    = false }
					if is_blocked(world_pos, Int3::new(0, -size, 0), env.bottom) { bottom_free = false }
					if is_blocked(world_pos, Int3::new(0, 0,  size), env.right)  { right_free  = false }
					if is_blocked(world_pos, Int3::new(0, 0, -size), env.left)   { left_free   = false }
				}

				if back_free   { cube  .back(build_pos, *color, vertices) }
				if front_free  { cube .front(build_pos, *color, vertices) }
				if top_free    { cube   .top(build_pos, *color, vertices) }
				if bottom_free { cube.bottom(build_pos, *color, vertices) }
				if right_free  { cube .right(build_pos, *color, vertices) }
				if left_free   { cube  .left(build_pos, *color, vertices) }
			} else {
				let mut back_free   = false;
				let mut front_free  = false;
				let mut top_free    = false;
				let mut bottom_free = false;
				let mut right_free  = false;
				let mut left_free   = false;

				for curr in SpaceIter::new(Int3::zero()..Int3::all(size)) {
					let high_pos = low_pos * size + curr;
					let world_pos = pos_in_chunk_to_world_int3(high_pos, self.pos);

					if !is_blocked(world_pos, Int3::new( size, 0, 0), env.back)   { back_free   = true }
					if !is_blocked(world_pos, Int3::new(-size, 0, 0), env.front)  { front_free  = true }
					if !is_blocked(world_pos, Int3::new(0,  size, 0), env.top)    { top_free    = true }
					if !is_blocked(world_pos, Int3::new(0, -size, 0), env.bottom) { bottom_free = true }
					if !is_blocked(world_pos, Int3::new(0, 0,  size), env.right)  { right_free  = true }
					if !is_blocked(world_pos, Int3::new(0, 0, -size), env.left)   { left_free   = true }
				}

				if back_free   { cube  .back(build_pos, *color, vertices) }
				if front_free  { cube .front(build_pos, *color, vertices) }
				if top_free    { cube   .top(build_pos, *color, vertices) }
				if bottom_free { cube.bottom(build_pos, *color, vertices) }
				if right_free  { cube .right(build_pos, *color, vertices) }
				if left_free   { cube  .left(build_pos, *color, vertices) }
			}
		}
	}

	fn get_lowered_voxels(&self, lod: NonZeroU32) -> Result<Vec<LoweredVoxel>, String> {
		/* Divide factor of chunk */
		let factor = 2_usize.pow(lod.get());

		/* Side length of low-res chunk in low-res voxels */
		let new_size = Self::SIZE / factor;

		/* Volume of low-res voxels array in low-res voxels */
		let new_volume = Self::VOLUME / factor.pow(3);
		
		match self.additional_data.as_ref() {
			/* Chunk was empty */
			Addition::Know { fill: Some(ChunkFill::Empty), details: _ } =>
				return Ok(vec![]),

			/* Chunk was filled with the save voxel */
			Addition::Know { fill: Some(ChunkFill::All(id)), details: _ } => {
				let voxel = LoweredVoxel::Colored(VOXEL_DATA[*id as usize].avarage_color);
				return Ok(vec![voxel; new_volume])
			},

			/* Chunk was standart-filled */
			Addition::Know { fill: Some(ChunkFill::Standart), details: _ } => {
				/* Resulting array of loweres voxels */
				let mut result = vec![LoweredVoxel::Transparent; new_volume];

				/* Write count array each element of is the denominator of avarage count */
				let mut n_writes = vec![0; new_volume];

				let iter = (0..Self::VOLUME).map(|i| {
					let pos = position_function(i);
					let id = self.voxel_ids[i];
					return (id, pos / factor as i32)
				});

				/* Low-res array index function */
				let low_index = |pos: Int3|
					sdex::get_index(&[pos.x() as usize, pos.y() as usize, pos.z() as usize], &[new_size; 3]);

				for (id, new_pos) in iter {
					/* Lowered voxels index shortcut */
					let low_i = low_index(new_pos);

					/* Writes count shortcut */
					let count = n_writes[low_i] as f32;

					/* Air blocks are not in count.
					|* Leaves empty voxels as [`LoweredVoxel::Transparent`] */
					if id != AIR_VOXEL_DATA.id {
						match &mut result[low_i] {
							/* If voxel is already initialyzed with some color */
							LoweredVoxel::Colored(color) => {
								/* Color shortcut */
								let [old_r, old_g, old_b] = color;
								let [new_r, new_g, new_b] = VOXEL_DATA[id as usize].avarage_color;

								/* Calculate new avarage color */
								*old_r = (*old_r * count + new_r) / (count + 1.0);
								*old_g = (*old_g * count + new_g) / (count + 1.0);
								*old_b = (*old_b * count + new_b) / (count + 1.0);

								/* Increment writes count */
								n_writes[low_i] += 1;
							},

							/* If voxel going to be initialyzed */
							LoweredVoxel::Transparent => {
								result[low_i] = LoweredVoxel::Colored(VOXEL_DATA[id as usize].avarage_color);
								n_writes[low_i] = 1;
							}
						}
					}
				}

				return Ok(result)
			},

			/* Not enough information */
			not_enough @ Addition::Know { fill: None, details: _ } | not_enough @ Addition::NothingToKnow =>
				return Err(format!("Not enough information! Addition was: {addition:?}", addition = not_enough)),
		}
	}

	/// Gives voxel by world coordinate.
	pub fn get_voxel_optional(&self, global_pos: Int3) -> FindOptions {
		/* Transform to local */
		let pos = world_coords_to_in_some_chunk(global_pos, self.pos);
		
		if pos.x() < 0 || pos.x() >= Self::SIZE as i32 ||
		   pos.y() < 0 || pos.y() >= Self::SIZE as i32 ||
		   pos.z() < 0 || pos.z() >= Self::SIZE as i32
		{ return FindOptions::OutOfBounds }
		
		match self.additional_data.as_ref() {
			/* For empty chunks */
			Addition::Know { fill: Some(ChunkFill::Empty), .. } => 
				FindOptions::InChunkNothing,

			/* For known-filled chunks */
			Addition::Know { fill: Some(fill), details } => {
				let voxel = match fill {
					ChunkFill::Standart =>
						Voxel::new(global_pos, &VOXEL_DATA[self.voxel_ids[pos_to_idx(pos)] as usize]),
						
					ChunkFill::All(id) =>
						Voxel::new(global_pos, &VOXEL_DATA[*id as usize]),

					ChunkFill::Empty =>
						unreachable!("Empty branch checked in previous pattern"),
				};

				match details {
					ChunkDetails::Full => FindOptions::InChunkDetailed(voxel),
					ChunkDetails::Low(lod) => FindOptions::InChunkLowered(voxel, *lod),
				}
			}

			/* No information */
			Addition::NothingToKnow => panic!("Not enough information!"),

			/* Other types */
			addition => panic!("Unresolvable chunk Addition {:?}!", addition),
		}
	}

	/// Gives voxel by world coordinate.
	pub fn get_voxel(&self, global_pos: Int3) -> ChunkOptional<Voxel> {
		/* Transform to local */
		let pos = world_coords_to_in_some_chunk(global_pos, self.pos);
		
		if pos.x() < 0 || pos.x() >= Self::SIZE as i32 ||
		   pos.y() < 0 || pos.y() >= Self::SIZE as i32 ||
		   pos.z() < 0 || pos.z() >= Self::SIZE as i32
		{ return ChunkOptional::OutsideChunk }
		
		match self.additional_data.as_ref() {
			/* For empty chunks */
			Addition::Know { fill: Some(ChunkFill::Empty), details } => {
				let voxel = Voxel::new(global_pos, &AIR_VOXEL_DATA);
				let lod = match details {
					ChunkDetails::Low(lod) => lod.get(),
					ChunkDetails::Full     => 0,
				};

				return ChunkOptional::Item(voxel, lod)
			}

			/* For known-filled chunks */
			Addition::Know { fill: Some(fill), details } => {
				let voxel = match fill {
					ChunkFill::Standart =>
						Voxel::new(global_pos, &VOXEL_DATA[self.voxel_ids[pos_to_idx(pos)] as usize]),
						
					ChunkFill::All(id) =>
						Voxel::new(global_pos, &VOXEL_DATA[*id as usize]),

					ChunkFill::Empty =>
						Voxel::new(global_pos, &AIR_VOXEL_DATA),
				};

				let lod = match details {
					ChunkDetails::Full     => 0,
					ChunkDetails::Low(lod) => lod.get(),
				};

				return ChunkOptional::Item(voxel, lod)
			}

			/* No information */
			Addition::NothingToKnow => panic!("Not enough information!"),

			/* Other types */
			addition => panic!("Unresolvable chunk Addition {:?}!", addition),
		}
	}

	/// Gives voxel by world coordinate.
	pub fn get_voxel_or_none(&self, pos: Int3) -> Option<Voxel> {
		match self.get_voxel_optional(pos) {
			FindOptions::OutOfBounds | FindOptions::InChunkNothing => None,
			FindOptions::InChunkDetailed(chunk) | FindOptions::InChunkLowered(chunk, _) => Some(chunk)
		}
	}

	/// Checks if chunk is in camera view.
	pub fn is_visible(&self, camera: &Camera) -> bool {
		/* AABB init */
		let mut lo = Float4::from(chunk_cords_to_min_world_int3(self.pos));
		let mut hi = lo + Int3::all(Self::SIZE as i32).into();

		/* Bias (voxel centration) */
		const BIAS: f32 = cfg::terrain::VOXEL_SIZE * 0.5;

		/* Biasing */
		lo -= Float4::all(BIAS);
		hi -= Float4::all(BIAS);

		/* Frustum check */
		camera.is_aabb_in_view(AABB::from_float4(lo, hi))
	}

	/// Upgrades chunk to be meshed.
	#[allow(dead_code)]
	pub fn envs_upgrade(self, graphics: &Graphics, env: &ChunkEnvironment) -> MeshedChunk {
		MeshedChunk::from_meshless_envs(graphics, self, env)
	}

	/// Upgrades chunk to be meshed.
	pub fn triangles_upgrade(self, graphics: &Graphics, triangles: DetailedVertexSlice) -> MeshedChunk {
		MeshedChunk::from_meshless_triangles(graphics, self, triangles)
	}

	/// Gives position iterator that gives position for all voxels in chunk.
	/// Internally, calls [`SpaceIter`]`::zeroed_cubed(CHUNK_SIZE as i32)`.
	pub fn pos_iter() -> SpaceIter {
		SpaceIter::zeroed_cubed(Self::SIZE as i32)
	}
}

/// Describes blocked chunks by environent or not.
#[derive(Clone, Default)]
pub struct ChunkEnvironment<'l> {
	// TODO: Make them NonNull because of Option.
	pub top:	Option<*const MeshlessChunk>,
	pub bottom:	Option<*const MeshlessChunk>,
	pub front:	Option<*const MeshlessChunk>,
	pub back:	Option<*const MeshlessChunk>,
	pub left:	Option<*const MeshlessChunk>,
	pub right:	Option<*const MeshlessChunk>,

	pub _phantom: PhantomData<&'l MeshlessChunk>
}

impl<'c> ChunkEnvironment<'c> {
	/// Empty description.
	pub fn none() -> Self {
		ChunkEnvironment { top: None, bottom: None, front: None, back: None, left: None, right: None, _phantom: PhantomData }
	}
}

use Detailed::*;
pub enum Detailed<Full, Low> {
	Full(Full),
	Low(Low),
}

pub type DetailedVertexSlice<'v> = Detailed<&'v [DetailedVertex], &'v [LoweredVertex]>;
pub type DetailedVertexVec = Detailed<Vec<DetailedVertex>, Vec<LoweredVertex>>;

pub struct ChunkMesh(Detailed<
	RefCell<UnindexedMesh<DetailedVertex>>,
	RefCell<UnindexedMesh<LoweredVertex>>
>);

impl ChunkMesh {
	/// Render mesh.
	pub fn render(
		&self, target: &mut Frame, full_shader: &Shader, low_shader: &Shader,
		draw_params: &DrawParameters<'_>, uniforms: &impl Uniforms) -> Result<(), DrawError>
	{
		match &self.0 {
			Full(mesh) => mesh.borrow().render(target, full_shader, draw_params, uniforms),
			Low(mesh)  => mesh.borrow().render(target, low_shader,  draw_params, uniforms),
		}
	}

	/// Checks if mesh is empty.
	pub fn is_empty(&self) -> bool {
		match &self.0 {
			Full(mesh) => mesh.borrow().is_empty(),
			Low(mesh)  => mesh.borrow().is_empty()
		}
	}
}

/// Chunk struct.
pub struct MeshedChunk {
	pub inner: MeshlessChunk,
	pub mesh: ChunkMesh,
}

impl MeshedChunk {
	/// Constructs new chunk in given position.
	#[allow(dead_code)]
	pub fn from_envs(graphics: &Graphics, pos: Int3, env: &ChunkEnvironment) -> Self {
		/* Construct new meshless */
		let meshless = MeshlessChunk::new(pos);
		
		/* Upgrade it */
		Self::from_meshless_envs(graphics, meshless, env)
	}

	/// Constructs mesh for meshless chunk.
	#[allow(dead_code)]
	pub fn from_meshless_envs(graphics: &Graphics, meshless: MeshlessChunk, env: &ChunkEnvironment) -> Self {
		/* Create chunk */
		let triangles = meshless.to_triangles(env);
		let triangles = match &triangles {
			Full(vec) => Full(&vec[..]),
			Low(vec) => Low(&vec[..]),
		};

		MeshedChunk {
			inner: meshless,
			mesh: Self::make_mesh(&graphics.display, triangles)
		}
	}

	/// Constructs mesh for meshless chunk.
	pub fn from_meshless_triangles(graphics: &Graphics, meshless: MeshlessChunk, triangles: Detailed<&[DetailedVertex], &[LoweredVertex]>) -> Self {
		MeshedChunk {
			inner: meshless,
			mesh: Self::make_mesh(&graphics.display, triangles)
		}
	}

	/// Renders chunk.
	/// * Mesh should be constructed before this function call.
	pub fn render<U: Uniforms>(
		&self, target: &mut Frame, full_shader: &Shader, low_shader: &Shader,
		uniforms: &U, draw_params: &glium::DrawParameters, camera: &Camera) -> Result<(), DrawError>
	{
		/* Check if vertex array is empty */
		if !self.mesh.is_empty() && self.is_visible(camera) {
			self.mesh.render(target, full_shader, low_shader, draw_params, uniforms)
		} else { Ok(()) }
	}

	/// Updates mesh.
	pub fn make_mesh(display: &glium::Display, vertices: Detailed<&[DetailedVertex], &[LoweredVertex]>) -> ChunkMesh {
		match vertices {
			Full(vertices) => {
				/* Vertex buffer for chunks */
				let vertex_buffer = VertexBuffer::no_indices(display, vertices, PrimitiveType::TrianglesList);

				ChunkMesh(Full(RefCell::new(Mesh::new(vertex_buffer))))
			},
			Low(vertices) => {
				/* Vertex buffer for chunks */
				let vertex_buffer = VertexBuffer::no_indices(display, vertices, PrimitiveType::TrianglesList);

				ChunkMesh(Low(RefCell::new(Mesh::new(vertex_buffer))))
			}
		}
		
	}

	/// Creates trianlges Vec from Chunk and its environment.
	#[allow(dead_code)]
	pub fn to_triangles(&self, env: &ChunkEnvironment) -> Detailed<Vec<DetailedVertex>, Vec<LoweredVertex>> {
		self.inner.to_triangles(env)
	}

	/// Gives voxel by world coordinate.
	#[allow(dead_code)]
	pub fn get_voxel_optional(&self, global_pos: Int3) -> FindOptions {
		self.inner.get_voxel_optional(global_pos)
	}

	/// Gives voxel by world coordinate.
	#[allow(dead_code)]
	pub fn get_voxel_or_none(&self, pos: Int3) -> Option<Voxel> {
		self.inner.get_voxel_or_none(pos)
	}

	/// Checks if chunk is in camera view.
	pub fn is_visible(&self, camera: &Camera) -> bool {
		self.inner.is_visible(camera)
	}
}

/// FIXME: turn into free function to prevent from conflicts, because [`VoxelArray`] = [`Vec<u16>`].
unsafe impl StaticSize for VoxelArray {
	fn static_size() -> usize {
		MeshlessChunk::VOLUME * u16::static_size()
	}
}

unsafe impl Reinterpret for MeshlessChunk { }

unsafe impl ReinterpretAsBytes for MeshlessChunk {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		let voxels = match self.additional_data.as_ref() {
			Addition::Know { fill: Some(ChunkFill::Empty), .. } => {
				Cow::Owned(vec![AIR_VOXEL_DATA.id; MeshlessChunk::VOLUME])
			},
			Addition::Know { fill: Some(ChunkFill::All(id)), .. } => {
				Cow::Owned(vec![*id; MeshlessChunk::VOLUME])
			},
			addition => {
				assert_eq!(
					self.voxel_ids.len(), MeshlessChunk::VOLUME,
					"Unresolvable array! Addition is {:?}", addition
				);
				Cow::Borrowed(&self.voxel_ids)
			}
		};

		let mut bytes = Vec::with_capacity(Self::static_size());

		bytes.append(&mut voxels.reinterpret_as_bytes());
		bytes.append(&mut self.pos.reinterpret_as_bytes());

		return bytes
	}
}

unsafe impl ReinterpretFromBytes for MeshlessChunk {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		let voxel_array_size: usize = VoxelArray::static_size();

		let voxels = VoxelArray::reinterpret_from_bytes(&source[.. voxel_array_size]);
		let pos = Int3::reinterpret_from_bytes(&source[voxel_array_size .. voxel_array_size + Int3::static_size()]);

		MeshlessChunk { voxel_ids: voxels, pos, additional_data: Default::default() }
	}
}

unsafe impl ReinterpretSize for MeshlessChunk {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for MeshlessChunk {
	fn static_size() -> usize {
		VoxelArray::static_size() + Int3::static_size() + 1
	}
}

unsafe impl Reinterpret for ChunkFill { }

unsafe impl ReinterpretAsBytes for ChunkFill {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		match self {
			Self::Empty => vec![0; Self::static_size()],
			Self::Standart => {
				let mut result = vec![0; Self::static_size()];
				result[0] = 2;

				return result
			},
			Self::All(id) => {
				let mut result = vec![1];
				result.append(&mut id.reinterpret_as_bytes());

				return result
			},
		}
	}
}

unsafe impl ReinterpretFromBytes for ChunkFill {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		match source[0] {
			0 => Self::Empty,
			1 => {
				let id = Id::reinterpret_from_bytes(&source[1..]);
				return Self::All(id)
			},
			2 => Self::Standart,
			_ => unreachable!("There's no ChunkFill variant that matches with {}!", source[0])
		}
	}
}

unsafe impl ReinterpretSize for ChunkFill {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for ChunkFill {
	fn static_size() -> usize { u8::static_size() + Id::static_size() }
}

#[cfg(test)]
mod reinterpret_test {
	use super::*;

	#[test]
	fn reinterpret_chunk() {
		let before = MeshlessChunk::new(Int3::new(12, 12, 11));
		let after = MeshlessChunk::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before.voxel_ids, after.voxel_ids);
		assert_eq!(before.pos, after.pos);
	}
}



/// Transforms world coordinates to chunk.
#[allow(dead_code)]
pub fn world_coords_to_chunk(pos: Int3) -> Int3 {
	pos / MeshlessChunk::SIZE as i32
}

/// Transforms chunk coords to world.
pub fn chunk_cords_to_min_world_int3(pos: Int3) -> Int3 {
	pos * MeshlessChunk::SIZE as i32
}

/// Transforms chunk coords to world.
pub fn chunk_cords_to_min_world_float4(pos: Float4) -> Float4 {
	pos * MeshlessChunk::SIZE as f32
}

/// Transforms in-chunk coords to world.
pub fn pos_in_chunk_to_world_int3(in_chunk: Int3, chunk: Int3) -> Int3 {
	chunk_cords_to_min_world_int3(chunk) + in_chunk
}

/// Transforms in-chunk coords to world.
pub fn pos_in_chunk_to_world_float4(in_chunk: Float4, chunk: Float4) -> Float4 {
	chunk_cords_to_min_world_float4(chunk) + in_chunk
}

/// Transforms world coordinates to chunk.
#[allow(dead_code)]
pub fn world_coords_to_in_chunk(pos: Int3) -> Int3 {
	/* Take voxel coordinates to near-zero */
	let x = pos.x() % MeshlessChunk::SIZE as i32;
	let y = pos.y() % MeshlessChunk::SIZE as i32;
	let z = pos.z() % MeshlessChunk::SIZE as i32;

	/* If negative then convert to positive */

	let x = if x < 0 {
		x + MeshlessChunk::SIZE as i32
	} else { x };

	let y = if y < 0 {
		y + MeshlessChunk::SIZE as i32
	} else { y };

	let z = if z < 0 {
		z + MeshlessChunk::SIZE as i32
	} else { z };

	Int3::new(x, y, z)
}

/// Transforms world coordinates to chunk.
pub fn world_coords_to_in_some_chunk(pos: Int3, chunk: Int3) -> Int3 {
	pos - chunk_cords_to_min_world_int3(chunk)
}

/// Index function.
pub fn pos_to_idx(pos: Int3) -> usize {
	sdex::get_index(&[pos.x() as usize, pos.y() as usize, pos.z() as usize], &[MeshlessChunk::SIZE; 3])
}

/// Position function.
pub fn position_function(i: usize) -> Int3 {
	general_position(i, MeshlessChunk::SIZE, MeshlessChunk::SIZE)
}

/// Position function.
pub fn general_position(i: usize, height: usize, depth: usize) -> Int3 {
	let xy = i / depth;

	let z =  i % depth;
	let y = xy % height;
	let x = xy / height;

	Int3::new(x as i32, y as i32, z as i32)
}