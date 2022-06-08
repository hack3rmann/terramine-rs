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

		/* Initialize vector */
		let mut chunks = Vec::<Chunk>::with_capacity(volume);
		let mut env = vec![ChunkEnv::none(); volume];

		/* Array borders */
		let x_lo: isize = -(width  as isize) / 2;
		let y_lo: isize = -(height as isize) / 2;
		let z_lo: isize = -(depth  as isize) / 2;
		let x_hi: isize = (width  / 2 + width  % 2) as isize;
		let y_hi: isize = (height / 2 + height % 2) as isize;
		let z_hi: isize = (depth  / 2 + depth  % 2) as isize;

		/* Fill vector with chunks with no mesh attached */
		for x in x_lo..x_hi {
		for y in y_lo..y_hi {
		for z in z_lo..z_hi {
			chunks.push(Chunk::new(&graphics, Int3::new(x as i32, y as i32, z as i32), false));
		}}}

		/* Fill environments with references to chunk array */
		for x in 0..width {
		for y in 0..height {
		for z in 0..depth {
			/* Local index function */
			let index = |x: usize, y: usize, z: usize| -> usize {
				(x * height + y) * depth + z
			};

			/* Reference to current environment variable */
			let env = &mut env[index(x, y, z)];

			/* For `front` side */
			if x as isize - 1 >= 0 {
				env.front	= Some(&chunks[index(x - 1, y, z)]);
			}

			/* For `back` side */
			if x + 1 < width {
				env.back	= Some(&chunks[index(x + 1, y, z)]);
			}

			/* For `bottom` side */
			if y as isize - 1 >= 0 {
				env.bottom	= Some(&chunks[index(x, y - 1, z)]);
			}
		
			/* For `top` side */
			if y + 1 < height {
				env.top		= Some(&chunks[index(x, y + 1, z)]);
			}

			/* For `left` side */
			if z as isize - 1 >= 0 {
				env.left	= Some(&chunks[index(x, y, z - 1)]);
			}

			/* For `right` side */
			if z + 1 < depth {
				env.right	= Some(&chunks[index(x, y, z + 1)]);
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