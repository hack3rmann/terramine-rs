use {
	crate::app::utils::{
		math::prelude::*,
		werror::prelude::*,
		terrain::{
			chunk::{MeshedChunk, CHUNK_SIZE, self},
			voxel::voxel_data::NOTHING_VOXEL_DATA,
		},
		graphics::{
			mesh::{UnindexedMesh, Mesh},
			shader::Shader,
			vertex_buffer::VertexBuffer,
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
		implement_vertex,
	},
	std::{
		marker::PhantomData,
		sync::atomic::{AtomicBool, Ordering}
	},
};

/// Adds debug visuals to type `T`.
pub struct DebugVisualized<T> {
	pub inner: T,
	pub mesh: UnindexedMesh<Vertex>,
	pub static_data: DebugVisualsStatics<T>,
}

pub struct DebugVisualsStatics<T> {
	pub shader: &'static Shader,
	pub draw_params: &'static DrawParameters<'static>,

	_phantom: PhantomData<T>
}

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn switch_enable() {
	ENABLED.store(!ENABLED.load(Ordering::Acquire), Ordering::Release);
}

#[repr(transparent)]
struct ShaderWrapper(Shader);

unsafe impl Send for ShaderWrapper { }
unsafe impl Sync for ShaderWrapper { }

#[repr(transparent)]
struct DrawParametersWrapper<'a>(DrawParameters<'a>);

unsafe impl<'a> Send for DrawParametersWrapper<'a> { }
unsafe impl<'a> Sync for DrawParametersWrapper<'a> { }

#[derive(Clone, Copy, PartialEq)]
pub struct Vertex {
	pos: [f32; 3],
	color: [f32; 4],
}

implement_vertex!(Vertex, pos, color);

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
				let shader = Shader::new("debug_lines", "debug_lines", display);
				SHADER.replace(ShaderWrapper(shader));
			}
			if let None = DRAW_PARAMS.as_ref() {
				let draw_params = DrawParameters {
					polygon_mode: glium::PolygonMode::Line,
					depth: Depth {
						test: DepthTest::IfLess,
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
			let lll = [ -0.5 + pos.x() as f32		, -0.5 + pos.y() as f32		  , -0.5 + pos.z() as f32 ];
			let llh = [ -0.5 + pos.x() as f32		, -0.5 + pos.y() as f32		  , -0.5 + pos.z() as f32 + SIZE ];
			let lhl = [ -0.5 + pos.x() as f32		, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 ];
			let lhh = [ -0.5 + pos.x() as f32		, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 + SIZE ];
			let hll = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32		  , -0.5 + pos.z() as f32 ];
			let hlh = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32		  , -0.5 + pos.z() as f32 + SIZE ];
			let hhl = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 ];
			let hhh = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 + SIZE ];

			let color = if chunk.inner.voxels.iter().all(|&id| id == NOTHING_VOXEL_DATA.id) {
				[0.5, 0.1, 0.1, 1.0]
			} else {
				[0.3, 0.3, 0.3, 1.0]
			};

			let vertices = [
				Vertex { pos: lll, color },
				Vertex { pos: lhl, color },
				
				Vertex { pos: llh, color },
				Vertex { pos: lhh, color },
				
				Vertex { pos: hlh, color },
				Vertex { pos: hhh, color },
				
				Vertex { pos: hll, color },
				Vertex { pos: hhl, color },
				

				Vertex { pos: lll, color },
				Vertex { pos: hll, color },
				
				Vertex { pos: lhl, color },
				Vertex { pos: hhl, color },
				
				Vertex { pos: lhh, color },
				Vertex { pos: hhh, color },
				
				Vertex { pos: llh, color },
				Vertex { pos: hlh, color },
				
				
				Vertex { pos: lll, color },
				Vertex { pos: llh, color },
				
				Vertex { pos: hll, color },
				Vertex { pos: hlh, color },
				
				Vertex { pos: hhl, color },
				Vertex { pos: hhh, color },
				
				Vertex { pos: lhl, color },
				Vertex { pos: lhh, color },
			];

			let vbuffer = VertexBuffer::no_indices(display, &vertices, PrimitiveType::LinesList);
			Mesh::new(vbuffer)
		};
		
		DebugVisualized { inner: chunk, mesh, static_data: chunk_data::get(display) }
	}

	pub fn render_debug(&self, target: &mut Frame, uniforms: &impl Uniforms) -> Result<(), DrawError> {
		if ENABLED.load(Ordering::Relaxed) {
			self.mesh.render(target, chunk_data::get_unchecked().shader, chunk_data::get_unchecked().draw_params, uniforms)
		} else { Ok(()) }
	}
}