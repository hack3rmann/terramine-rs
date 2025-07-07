use {
    super::*,
    crate::app::utils::{
        cfg,
        graphics::{camera::Camera, mesh::UnindexedMesh},
    },
    glium::{
        BackfaceCullingMode, Depth, DepthTest, DrawError, VertexBuffer, index::PrimitiveType,
        uniforms::Uniforms,
    },
    lazy_static::lazy_static,
    std::sync::atomic::Ordering,
};

pub mod data {
    use super::*;
    use glium::backend::Facade;
    use std::mem::MaybeUninit;

    static IS_INIT: AtomicBool = AtomicBool::new(false);
    static mut SHADER: MaybeUninit<ShaderWrapper> = MaybeUninit::uninit();

    lazy_static! {
        static ref DRAW_PARAMS: DrawParametersWrapper<'static> =
            DrawParametersWrapper(DrawParameters {
                polygon_mode: glium::PolygonMode::Line,
                line_width: Some(2.0),
                depth: Depth {
                    test: DepthTest::IfLessOrEqual,
                    write: true,
                    ..Default::default()
                },
                backface_culling: BackfaceCullingMode::CullingDisabled,
                ..Default::default()
            });
    }

    pub fn get<'s>(facade: &dyn Facade) -> DebugVisualsStatics<'s, Camera> {
        cond_init(facade);
        unsafe { get_unchecked() }
    }

    /// # Safety
    ///
    /// Shader should be initialized
    unsafe fn get_unchecked<'s>() -> DebugVisualsStatics<'s, Camera> {
        DebugVisualsStatics {
            shader: unsafe {
                (&raw const SHADER)
                    .cast::<Shader>()
                    .as_ref()
                    .unwrap_unchecked()
            },
            draw_params: &DRAW_PARAMS.0,
            _phantom: PhantomData,
        }
    }

    fn cond_init(facade: &dyn Facade) {
        if !IS_INIT.fetch_or(true, Ordering::SeqCst) {
            let shader =
                Shader::new("debug_lines", "debug_lines", facade).expect("failed to make shader");

            let value = MaybeUninit::new(ShaderWrapper(shader));

            unsafe { (&raw mut SHADER).write(value) }
        }
    }

    pub fn construct_mesh(camera: &mut Camera, facade: &dyn Facade) -> UnindexedMesh<Vertex> {
        let color = [0.5; 4];
        let rays = camera.get_frustum().courner_rays;
        const LEN: f32 = cfg::camera::FRUSTUM_EDGE_LINE_LENGTH;
        let vertices = [
            Vertex {
                pos: [rays[0].origin.x, rays[0].origin.y, rays[0].origin.z],
                color,
            },
            Vertex {
                pos: [
                    rays[0].origin.x + LEN * rays[0].direction.x,
                    rays[0].origin.y + LEN * rays[0].direction.y,
                    rays[0].origin.z + LEN * rays[0].direction.z,
                ],
                color,
            },
            Vertex {
                pos: [rays[1].origin.x, rays[1].origin.y, rays[1].origin.z],
                color,
            },
            Vertex {
                pos: [
                    rays[1].origin.x + LEN * rays[1].direction.x,
                    rays[1].origin.y + LEN * rays[1].direction.y,
                    rays[1].origin.z + LEN * rays[1].direction.z,
                ],
                color,
            },
            Vertex {
                pos: [rays[2].origin.x, rays[2].origin.y, rays[2].origin.z],
                color,
            },
            Vertex {
                pos: [
                    rays[2].origin.x + LEN * rays[2].direction.x,
                    rays[2].origin.y + LEN * rays[2].direction.y,
                    rays[2].origin.z + LEN * rays[2].direction.z,
                ],
                color,
            },
            Vertex {
                pos: [rays[3].origin.x, rays[3].origin.y, rays[3].origin.z],
                color,
            },
            Vertex {
                pos: [
                    rays[3].origin.x + LEN * rays[3].direction.x,
                    rays[3].origin.y + LEN * rays[3].direction.y,
                    rays[3].origin.z + LEN * rays[3].direction.z,
                ],
                color,
            },
        ];

        let vertices =
            VertexBuffer::new(facade, &vertices).expect("failed to create vertex buffer");
        UnindexedMesh::new_unindexed(vertices, PrimitiveType::LinesList)
    }
}

impl DebugVisualized<'_, Camera> {
    pub fn new_camera(camera: Camera, facade: &dyn glium::backend::Facade) -> Self {
        let mesh = UnindexedMesh::new_empty(facade, PrimitiveType::LinesList)
            .expect("failed to create mesh");

        Self {
            inner: camera,
            mesh,
            static_data: data::get(facade),
        }
    }

    pub fn render_camera_debug_visuals(
        &mut self,
        facade: &dyn glium::backend::Facade,
        target: &mut impl glium::Surface,
        uniforms: &impl Uniforms,
    ) -> Result<(), DrawError> {
        if ENABLED.load(Ordering::Relaxed) {
            let mesh = data::construct_mesh(&mut self.inner, facade);
            mesh.render(
                target,
                self.static_data.shader,
                self.static_data.draw_params,
                uniforms,
            )
        } else {
            Ok(())
        }
    }
}
