use super::mesh::Mesh;

use {
	crate::app::utils::{
		math::prelude::*,
		werror::prelude::*,
		terrain::chunk::{MeshedChunk, CHUNK_SIZE, self},
		graphics::{
			mesh::UnindexedMesh,
			shader::Shader,
			vertex_buffer::VertexBuffer,
			Vertex,
		}
	},
	glium::{
		DrawParameters,
		Display,
		Depth,
		DepthTest,
		BackfaceCullingMode,
		index::PrimitiveType,
		uniforms::Uniforms,
		Frame,
		DrawError,
	},
	std::marker::PhantomData,
};

/// Adds debug visuals to type `T`
pub struct DebugVisualized<T> {
	pub inner: T,
	pub mesh: UnindexedMesh,
	pub static_data: DebugVisualsStatics<T>,
}

pub struct DebugVisualsStatics<T> {
	pub shader: &'static Shader,
	pub draw_params: &'static DrawParameters<'static>,

	_phantom: PhantomData<T>
}

#[repr(transparent)]
struct ShaderWrapper(Shader);

unsafe impl Send for ShaderWrapper { }
unsafe impl Sync for ShaderWrapper { }

#[repr(transparent)]
struct DrawParametersWrapper<'a>(DrawParameters<'a>);

unsafe impl<'a> Send for DrawParametersWrapper<'a> { }
unsafe impl<'a> Sync for DrawParametersWrapper<'a> { }

/**
 * Debug visuals for [`Chunk`]
 */

pub mod chunk_data {
	use super::*;

	static mut SHADER: Option<ShaderWrapper> = None;
	static mut DRAW_PARAMS: Option<DrawParametersWrapper> = None;

	pub fn get(display: &Display) -> DebugVisualsStatics<MeshedChunk> {
		unsafe {
			cond_init(display);

			DebugVisualsStatics {
				shader: &SHADER.as_ref().wunwrap().0,
				draw_params: &DRAW_PARAMS.as_ref().wunwrap().0,
				_phantom: PhantomData
			}
		}
	}

	pub fn get_unchecked() -> DebugVisualsStatics<MeshedChunk> {
		unsafe {
			DebugVisualsStatics {
				shader: &SHADER.as_ref().wunwrap().0,
				draw_params: &DRAW_PARAMS.as_ref().wunwrap().0,
				_phantom: PhantomData
			}
		}
	}

	pub fn cond_init(display: &Display) {
		unsafe {
			/* Check if uninitialyzed */
			if let None = SHADER.as_ref() {
				let shader = Shader::new("vertex_shader", "fragment_shader", display);
				SHADER.replace(ShaderWrapper(shader));
			}
			if let None = DRAW_PARAMS.as_ref() {
				let draw_params = DrawParameters {
					depth: Depth {
						test: DepthTest::Overwrite,
						write: true,
						.. Default::default()
					},
					backface_culling: BackfaceCullingMode::CullingDisabled,
					.. Default::default()
				};
				DRAW_PARAMS.replace(DrawParametersWrapper(draw_params));
			}
		}
	}
}

impl DebugVisualized<MeshedChunk> {
	pub fn new(chunk: MeshedChunk, display: &Display) -> Self {
		let mesh = {
			const SIZE: f32 = CHUNK_SIZE as f32;
			let pos = chunk::chunk_cords_to_min_world(chunk.inner.pos);
			let lll = [ pos.x() as f32		 , pos.y() as f32		, pos.z() as f32 ];
			let llh = [ pos.x() as f32		 , pos.y() as f32		, pos.z() as f32 + SIZE ];
			let lhl = [ pos.x() as f32		 , pos.y() as f32 + SIZE, pos.z() as f32 ];
			let lhh = [ pos.x() as f32		 , pos.y() as f32 + SIZE, pos.z() as f32 + SIZE ];
			let hll = [ pos.x() as f32 + SIZE, pos.y() as f32		, pos.z() as f32 ];
			let hlh = [ pos.x() as f32 + SIZE, pos.y() as f32		, pos.z() as f32 + SIZE ];
			let hhl = [ pos.x() as f32 + SIZE, pos.y() as f32 + SIZE, pos.z() as f32 ];
			let hhh = [ pos.x() as f32 + SIZE, pos.y() as f32 + SIZE, pos.z() as f32 + SIZE ];

			let tex_coords = [0.033, 0.001];
			let vertices = vec![
				Vertex { position: lll, tex_coords, light: 1.0 },
				Vertex { position: lhl, tex_coords, light: 1.0 },
				
				Vertex { position: llh, tex_coords, light: 1.0 },
				Vertex { position: lhh, tex_coords, light: 1.0 },
				
				Vertex { position: hlh, tex_coords, light: 1.0 },
				Vertex { position: hhh, tex_coords, light: 1.0 },
				
				Vertex { position: hll, tex_coords, light: 1.0 },
				Vertex { position: hhl, tex_coords, light: 1.0 },
				

				Vertex { position: lll, tex_coords, light: 1.0 },
				Vertex { position: hll, tex_coords, light: 1.0 },
				
				Vertex { position: lhl, tex_coords, light: 1.0 },
				Vertex { position: hhl, tex_coords, light: 1.0 },
				
				Vertex { position: lhh, tex_coords, light: 1.0 },
				Vertex { position: hhh, tex_coords, light: 1.0 },
				
				Vertex { position: llh, tex_coords, light: 1.0 },
				Vertex { position: hlh, tex_coords, light: 1.0 },
				
				
				Vertex { position: lll, tex_coords, light: 1.0 },
				Vertex { position: llh, tex_coords, light: 1.0 },
				
				Vertex { position: hll, tex_coords, light: 1.0 },
				Vertex { position: hlh, tex_coords, light: 1.0 },
				
				Vertex { position: hhl, tex_coords, light: 1.0 },
				Vertex { position: hhh, tex_coords, light: 1.0 },
				
				Vertex { position: lhl, tex_coords, light: 1.0 },
				Vertex { position: lhh, tex_coords, light: 1.0 },
			];

			let vbuffer = VertexBuffer::no_indices(display, &vertices, PrimitiveType::LinesList);
			Mesh::new(vbuffer)
		};
		
		DebugVisualized { inner: chunk, mesh, static_data: chunk_data::get(display) }
	}

	pub fn render_debug(&self, target: &mut Frame, uniforms: &impl Uniforms) -> Result<(), DrawError> {
		self.mesh.render(target, chunk_data::get_unchecked().shader, chunk_data::get_unchecked().draw_params, uniforms)
	}
}