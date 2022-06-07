use super::{Chunk, ChunkEnvironment as ChunkEnv};
use crate::app::utils::{
	graphics::Graphics,
	math::vector::{Int3, swizzle::*}
};
use glium::{
	uniforms::Uniforms,
	DrawError,
	Frame
};

/// Represents self-controlling chunk array.
/// * Width is bigger if you go to x+ direction
/// * Height is bigger if you go to y+ direction
/// * Depth is bigger if you go to z+ direction
#[allow(dead_code)]
pub struct ChunkArray<'a> {
	/* Size */
	width:	usize,
	height:	usize,
	depth:	usize,

	/* Chunk array itself */
	chunks: Vec<Chunk<'a>>,
}

impl<'a> ChunkArray<'a> {
	pub fn new(graphics: &Graphics, width: usize, height: usize, depth: usize) -> Self {
		/* Amount of voxels in chunks */
		let volume = width * height * depth;

		/* Array borders */
		let x_lo: i32 = -(width  as i32) / 2;
		let y_lo: i32 = -(height as i32) / 2;
		let z_lo: i32 = -(depth  as i32) / 2;
		let x_hi: i32 = (width  / 2 + width  % 2) as i32;
		let y_hi: i32 = (height / 2 + height % 2) as i32;
		let z_hi: i32 = (depth  / 2 + depth  % 2) as i32;

		/* Initialize vector */
		let mut chunks = Vec::<Chunk>::with_capacity(volume);
		let mut env = vec![ChunkEnv::none(); volume];

		/* Fill vector with chunks with no mesh attached */
		for x in x_lo..x_hi {
		for y in y_lo..y_hi {
		for z in z_lo..z_hi {
			chunks.push(Chunk::new(&graphics, Int3::new(x, y, z), false));
		}}}

		/* Fill environments with references to chunk array */
		for x in 0..width {
		for y in 0..height {
		for z in 0..depth {
			/* Reference to current environment variable */
			let env = &mut env[(z * depth + y) * height + x];

			/* For `front` side */
			if x as i32 - 1 >= 0 {
				let index = (z * depth + y) * height + (x - 1);
				env.left = Some(&chunks[index]);
			}

			/* For `back` side */
			if x + 1 < width {
				let index = (z * depth + y) * height + (x + 1);
				env.right = Some(&chunks[index]);
			}

			/* For `bottom` side */
			if y as i32 - 1 >= 0 {
				let index = (z * depth + (y - 1)) * height + x;
				env.bottom = Some(&chunks[index]);
			}
		
			/* For `top` side */
			if y + 1 < height {
				let index = (z * depth + (y + 1)) * height + x;
				env.top = Some(&chunks[index]);
			}

			/* For `left` side */
			if z as i32 - 1 >= 0 {
				let index = ((z - 1) * depth + y) * height + x;
				env.front = Some(&chunks[index]);
			}

			/* For `right` side */
			if z + 1 < depth {
				let index = ((z + 1) * depth + y) * height + x;
				env.back = Some(&chunks[index]);
			}
		}}}

		/* Create mesh for each chunk */
		let mut env_iter = env.iter();
		chunks.iter().for_each(|chunk| chunk.update_mesh(&graphics, env_iter.next().unwrap()));

		ChunkArray { width, height, depth, chunks }
	}

	/// Renders chunks.
	pub fn render<U: Uniforms>(&mut self, target: &mut Frame, uniforms: &U) -> Result<(), DrawError> {
		/* Iterating through array */
		for chunk in self.chunks.iter_mut() {
			chunk.render(target, uniforms)?
		}
		Ok(())
	}
}