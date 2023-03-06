use {
    crate::app::utils::{
        cfg,
        graphics::{
            camera::Camera,
            vertex_buffer::VertexBuffer,
            mesh::UnindexedMesh,
        },
        terrain::{
            chunk::{
                Chunk,
                chunk_array::ChunkArray,
                ChunkDrawBundle,
                ChunkRenderError,
            },
            voxel::Voxel,
        },
        profiler::prelude::*,
    },
    super::*,
    glium::{
        Display, Depth, DepthTest, BackfaceCullingMode, Frame,
        index::PrimitiveType,
        uniforms::Uniforms,
    },
    math_linear::prelude::*,
};

pub mod data {
    use super::*;

    static mut SHADER: Option<ShaderWrapper> = None;
    static mut DRAW_PARAMS: Option<DrawParametersWrapper> = None;

    pub fn get<'s>(display: &Display) -> DebugVisualsStatics<'s, ChunkArray> {
        cond_init(display);
        get_unchecked()
    }

    pub fn get_unchecked<'s>() -> DebugVisualsStatics<'s, ChunkArray> {
        unsafe {
            let err_msg = "debug visuals statics should been initialized";

            let ShaderWrapper(shader) = &SHADER
                .as_ref()
                .expect(err_msg);

            let DrawParametersWrapper(draw_params) = &DRAW_PARAMS
                .as_ref()
                .expect(err_msg);
            
            DebugVisualsStatics { shader, draw_params, _phantom: PhantomData }
        }
    }

    pub fn cond_init(display: &Display) {
        unsafe {
            /* Check if uninitialized */
            if SHADER.is_none() {
                let shader = Shader::new("debug_lines", "debug_lines", display);
                SHADER.replace(ShaderWrapper(shader));
            }

            if DRAW_PARAMS.is_none() {
                let draw_params = DrawParameters {
                    polygon_mode: glium::PolygonMode::Line,
                    line_width: Some(1.5),
                    depth: Depth {
                        test: DepthTest::IfLessOrEqual,
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

    pub fn construct_mesh(chunk_arr: &ChunkArray, display: &Display) -> UnindexedMesh<Vertex> {
        let vertices: Vec<_> = chunk_arr.chunks()
            .flat_map(|chunk| {
                let bias = cfg::topology::Z_FIGHTING_BIAS
                         * (chunk.info.active_lod as f32 * 80.0 + 1.0);
                let size = Chunk::GLOBAL_SIZE as f32 + bias;

                let pos = vec3::from(Chunk::global_pos(chunk.pos)) * Voxel::SIZE
                        - vec3::all(0.5 * Voxel::SIZE);
                        
                let lll = [ pos.x - bias, pos.y - bias, pos.z - bias ];
                let llh = [ pos.x - bias, pos.y - bias, pos.z + size ];
                let lhl = [ pos.x - bias, pos.y + size, pos.z - bias ];
                let lhh = [ pos.x - bias, pos.y + size, pos.z + size ];
                let hll = [ pos.x + size, pos.y - bias, pos.z - bias ];
                let hlh = [ pos.x + size, pos.y - bias, pos.z + size ];
                let hhl = [ pos.x + size, pos.y + size, pos.z - bias ];
                let hhh = [ pos.x + size, pos.y + size, pos.z + size ];

                let color = if !chunk.is_generated() {
                    [0.1, 0.0, 0.0, 0.5]
                } else if chunk.is_empty() {
                    [0.5, 0.1, 0.1, 0.5]
                } else if chunk.is_same_filled() {
                    [0.1, 0.1, 0.5, 0.5]
                } else {
                    [0.3, 0.3, 0.3, 0.5]
                };

                let color = color.map(|c| {
                    let lod_coef = 1.0
                                 - chunk.info.active_lod as f32
                                     / Chunk::N_LODS as f32
                                 + 0.001;
                    c * (lod_coef * 0.7 + 0.3)
                });

                [
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
                ]
            })
            .collect();

        let vbuffer = VertexBuffer::no_indices(display, &vertices, PrimitiveType::LinesList);
        UnindexedMesh::new(vbuffer)
    }
}

impl DebugVisualized<'_, ChunkArray> {
    pub fn new_chunk_array(chunk_array: ChunkArray, display: &Display) -> Self {
        let mesh = data::construct_mesh(&chunk_array, display);
        Self { inner: chunk_array, mesh, static_data: data::get(display) }
    }

    #[profile]
    pub async fn render_chunk_array(
        &mut self, target: &mut Frame, draw_bundle: &ChunkDrawBundle<'_>,
        uniforms: &impl Uniforms, display: &Display, cam: &Camera,
    ) -> Result<(), ChunkRenderError>
    where
        Self: 'static,
    {
        self.render(target, draw_bundle, uniforms, display, cam).await?;

        if ENABLED.load(Ordering::Relaxed) {
            self.mesh = data::construct_mesh(self, display);
        
            let shader = data::get(display).shader;
            let draw_params = data::get(display).draw_params;
            self.mesh.render(target, shader, draw_params, uniforms)?;
        }

        Ok(())
    }
}