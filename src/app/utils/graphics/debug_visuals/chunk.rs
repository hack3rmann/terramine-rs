use crate::app::utils::terrain::chunk::ChunkEnvironment;

/**
 * Debug visuals for [`Chunk`]
 */

use {
    crate::app::utils::{
        cfg,
        werror::prelude::*,
        terrain::chunk::{
            self,
            MeshedChunk,
            MeshlessChunk as Chunk,
        },
        graphics::{
            mesh::Mesh,
            shader::Shader,
            vertex_buffer::VertexBuffer,
            camera::Camera,
        }
    },
    super::*,
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
    std::{
        marker::PhantomData,
        sync::atomic::Ordering
    },
};

pub mod data {
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

    #[allow(dead_code)]
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
                    line_width: Some(2.0),
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
}

impl DebugVisualized<MeshedChunk> {
    pub fn new_meshed_chunk(chunk: MeshedChunk, display: &Display) -> Self {
        let mesh = {
            const BIAS: f32 = cfg::topology::Z_FIGHTING_BIAS;
            const SIZE: f32 = Chunk::SIZE as f32 + BIAS;
            let pos = chunk::chunk_coords_to_min_world_int3(chunk.inner.pos);
            let lll = [ -0.5 + pos.x() as f32 - BIAS, -0.5 + pos.y() as f32 - BIAS, -0.5 + pos.z() as f32 - BIAS ];
            let llh = [ -0.5 + pos.x() as f32 - BIAS, -0.5 + pos.y() as f32 - BIAS, -0.5 + pos.z() as f32 + SIZE ];
            let lhl = [ -0.5 + pos.x() as f32 - BIAS, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 - BIAS ];
            let lhh = [ -0.5 + pos.x() as f32 - BIAS, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 + SIZE ];
            let hll = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 - BIAS, -0.5 + pos.z() as f32 - BIAS ];
            let hlh = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 - BIAS, -0.5 + pos.z() as f32 + SIZE ];
            let hhl = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 - BIAS ];
            let hhh = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 + SIZE ];

            let color = if chunk.inner.is_empty() {
                [0.5, 0.1, 0.1, 0.5]
            } else if chunk.inner.is_filled() {
                [0.1, 0.1, 0.5, 0.5]
            } else {
                [0.3, 0.3, 0.3, 0.5]
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
        
        DebugVisualized { inner: chunk, mesh, static_data: data::get(display) }
    }

    pub fn render_debug_meshed_chunks(&self, target: &mut Frame, uniforms: &impl Uniforms, camera: &Camera) -> Result<(), DrawError> {
        match ENABLED.load(Ordering::Relaxed) && self.inner.is_visible(camera) {
            true => self.mesh.render(target, self.static_data.shader, self.static_data.draw_params, uniforms),
            false => Ok(()),
        }
    }

    pub fn render_meshed_chunks(&self, target: &mut Frame, full_shader: &Shader, low_shader: &Shader, uniforms: &impl Uniforms, draw_params: &DrawParameters, camera: &Camera) -> Result<(), DrawError> {
        self.inner.render(target, full_shader, low_shader, uniforms, draw_params, camera)?;
        self.render_debug_meshed_chunks(target, uniforms, camera)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update_details(&mut self, display: &Display, env: &ChunkEnvironment, camera: &Camera) {
        if !self.update_details_data(camera) { return }
        self.refresh_mesh(display, env)
    }

    pub fn update_details_data(&mut self, camera: &Camera) -> bool {
        self.inner.update_details_data(camera)
    }

    pub fn refresh_mesh(&mut self, display: &Display, env: &ChunkEnvironment) {
        self.inner.refresh_mesh(display, env)
    }
}