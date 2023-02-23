pub mod chunk;
pub mod camera;

use {
    crate::app::utils::graphics::{
        mesh::UnindexedMesh,
        shader::Shader,
    },
    glium::{
        DrawParameters,
        implement_vertex,
    },
    std::{
        marker::PhantomData,
        sync::atomic::{AtomicBool, Ordering}
    },
};

/// Adds debug visuals to type `T`.
#[derive(Debug)]
pub struct DebugVisualized<T> {
    pub inner: T,
    pub mesh: UnindexedMesh<Vertex>,
    pub static_data: DebugVisualsStatics<T>,
}

#[derive(Debug)]
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
#[derive(Debug)]
struct ShaderWrapper(Shader);

unsafe impl Send for ShaderWrapper { }
unsafe impl Sync for ShaderWrapper { }

#[repr(transparent)]
#[derive(Debug)]
struct DrawParametersWrapper<'a>(DrawParameters<'a>);

unsafe impl<'a> Send for DrawParametersWrapper<'a> { }
unsafe impl<'a> Sync for DrawParametersWrapper<'a> { }

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
}

implement_vertex!(Vertex, pos, color);