//! 
//! Debug visuals for [`Chunk`]
//! 

use {
    crate::app::utils::{
        cfg,
        terrain::chunk::{
            Chunk,
        },
        graphics::{
            mesh::Mesh,
            shader::Shader,
            vertex_buffer::VertexBuffer,
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
    },
    std::{
        marker::PhantomData,
    },
};

pub mod data {
    use super::*;

    static mut SHADER: Option<ShaderWrapper> = None;
    static mut DRAW_PARAMS: Option<DrawParametersWrapper> = None;

    pub fn get<'s>(display: &Display) -> DebugVisualsStatics<'s, Chunk> {
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
    pub fn get_unchecked<'s>() -> DebugVisualsStatics<'s, Chunk> {
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
            if SHADER.is_none() {
                let shader = Shader::new("debug_lines", "debug_lines", display);
                SHADER.replace(ShaderWrapper(shader));
            }

            if DRAW_PARAMS.is_none() {
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

impl DebugVisualized<'_, Chunk> {
    pub fn new_meshed_chunk(chunk: Chunk, display: &Display) -> Self {
        let mesh = {
            const BIAS: f32 = cfg::topology::Z_FIGHTING_BIAS;
            const SIZE: f32 = Chunk::SIZE as f32 + BIAS;
            let pos = Chunk::global_pos(chunk.pos);
            let lll = [ -0.5 + pos.x() as f32 - BIAS, -0.5 + pos.y() as f32 - BIAS, -0.5 + pos.z() as f32 - BIAS ];
            let llh = [ -0.5 + pos.x() as f32 - BIAS, -0.5 + pos.y() as f32 - BIAS, -0.5 + pos.z() as f32 + SIZE ];
            let lhl = [ -0.5 + pos.x() as f32 - BIAS, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 - BIAS ];
            let lhh = [ -0.5 + pos.x() as f32 - BIAS, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 + SIZE ];
            let hll = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 - BIAS, -0.5 + pos.z() as f32 - BIAS ];
            let hlh = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 - BIAS, -0.5 + pos.z() as f32 + SIZE ];
            let hhl = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 - BIAS ];
            let hhh = [ -0.5 + pos.x() as f32 + SIZE, -0.5 + pos.y() as f32 + SIZE, -0.5 + pos.z() as f32 + SIZE ];

            let color = if chunk.is_empty() {
                [0.5, 0.1, 0.1, 0.5]
            } else if chunk.is_same_filled() {
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
}