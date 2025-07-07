use {
    super::*,
    crate::{
        graphics::mesh::Mesh,
        prelude::*,
        terrain::{
            chunk::{Chunk, chunk_array::ChunkArray},
            voxel::Voxel,
        },
    },
    glium::{
        BackfaceCullingMode, Depth, DepthTest, VertexBuffer, index::PrimitiveType,
        uniforms::Uniforms,
    },
};

pub mod data {
    use glium::backend::Facade;

    use super::*;
    use std::mem::MaybeUninit;

    static IS_INIT: AtomicBool = AtomicBool::new(false);
    static mut SHADER: MaybeUninit<ShaderWrapper> = MaybeUninit::uninit();

    lazy_static! {
        static ref DRAW_PARAMS: DrawParametersWrapper<'static> =
            DrawParametersWrapper(DrawParameters {
                polygon_mode: glium::PolygonMode::Line,
                line_width: Some(1.5),
                depth: Depth {
                    test: DepthTest::IfLessOrEqual,
                    write: true,
                    ..Default::default()
                },
                backface_culling: BackfaceCullingMode::CullingDisabled,
                ..Default::default()
            });
    }

    pub fn get<'s>(facade: &dyn Facade) -> DebugVisualsStatics<'s, ChunkArray> {
        cond_init(facade);
        unsafe { get_unchecked() }
    }

    /// # Safety
    ///
    /// Shader should be initialized
    unsafe fn get_unchecked<'s>() -> DebugVisualsStatics<'s, ChunkArray> {
        let shader = unsafe {
            (&raw const SHADER)
                .cast::<Shader>()
                .as_ref()
                .unwrap_unchecked()
        };

        DebugVisualsStatics {
            shader,
            draw_params: &DRAW_PARAMS.0,
            _phantom: PhantomData,
        }
    }

    fn cond_init(facade: &dyn Facade) {
        if !IS_INIT.fetch_or(true, Ordering::SeqCst) {
            let shader = Shader::new("debug_lines", "debug_lines", facade)
                .expect("failed to make shader");

            let value = MaybeUninit::new(ShaderWrapper(shader));

            unsafe { (&raw mut SHADER).write(value) };
        }
    }

    pub async fn construct_mesh(
        chunk_arr: &ChunkArray,
        facade: &dyn Facade,
    ) -> UnindexedMesh<Vertex> {
        let mut vertices = SmallVec::<[_; 24]>::new();

        for (chunk, chunk_mesh) in chunk_arr.chunks.iter().zip(chunk_arr.meshes.iter()) {
            let active_lod = chunk.info.load(Relaxed).active_lod.unwrap_or(0);
            let chunk_pos = chunk.pos.load(Relaxed);
            let is_generated = chunk.is_generated();
            let is_partitioned = chunk_mesh.borrow().is_partitioned();
            let is_empty = chunk.is_empty();
            let is_same_filled = chunk.is_same_filled();

            let bias = cfg::topology::Z_FIGHTING_BIAS * (active_lod as f32 * 80.0 + 1.0);
            let size = Chunk::GLOBAL_SIZE + bias;

            let pos = vec3::from(Chunk::global_pos(chunk_pos)) * Voxel::SIZE
                - vec3::all(0.5 * Voxel::SIZE);

            let lll = [pos.x - bias, pos.y - bias, pos.z - bias];
            let llh = [pos.x - bias, pos.y - bias, pos.z + size];
            let lhl = [pos.x - bias, pos.y + size, pos.z - bias];
            let lhh = [pos.x - bias, pos.y + size, pos.z + size];
            let hll = [pos.x + size, pos.y - bias, pos.z - bias];
            let hlh = [pos.x + size, pos.y - bias, pos.z + size];
            let hhl = [pos.x + size, pos.y + size, pos.z - bias];
            let hhh = [pos.x + size, pos.y + size, pos.z + size];

            let color = if !is_generated {
                [0.1, 0.0, 0.0, 0.5]
            } else if is_partitioned {
                [0.1, 0.5, 0.0, 0.5]
            } else if is_empty {
                [0.5, 0.1, 0.1, 0.5]
            } else if is_same_filled {
                [0.1, 0.1, 0.5, 0.5]
            } else {
                [0.3, 0.3, 0.3, 0.5]
            };

            let color = color.map(|c| {
                let lod_coef = 1.0 - active_lod as f32 / Chunk::N_LODS as f32 + 0.001;
                c * (lod_coef * 0.7 + 0.3)
            });

            vertices.append(&mut smallvec![
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
            ] as &mut SmallVec<[_; 24]>);
        }

        let vbuffer = VertexBuffer::new(facade, &vertices).expect("failed to create vertex buffer");

        Mesh::new_unindexed(vbuffer, PrimitiveType::LinesList)
    }
}

impl<'s> DebugVisualized<'s, ChunkArray> {
    pub async fn new_chunk_array(
        chunk_array: ChunkArray,
        facade: &dyn glium::backend::Facade,
    ) -> DebugVisualized<'s, ChunkArray> {
        let mesh = data::construct_mesh(&chunk_array, facade).await;
        Self {
            inner: chunk_array,
            mesh,
            static_data: data::get(facade),
        }
    }

    pub async fn render_chunk_debug(
        &mut self,
        facade: &dyn glium::backend::Facade,
        target: &mut impl glium::Surface,
        uniforms: &impl Uniforms,
    ) -> Result<(), glium::DrawError> {
        if ENABLED.load(Ordering::Relaxed) {
            self.mesh = data::construct_mesh(self, facade).await;

            let shader = data::get(facade).shader;
            let draw_params = data::get(facade).draw_params;
            self.mesh.render(target, shader, draw_params, uniforms)?;
        }

        Ok(())
    }
}
