pub mod camera;
pub mod chunk_array;

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
    derive_deref_rs::Deref,
};

/// Adds debug visuals to type `T`.
#[derive(Debug, Deref)]
pub struct DebugVisualized<'s, T> {
    #[deref]
    pub inner: T,
    
    pub mesh: UnindexedMesh<Vertex>,
    pub static_data: DebugVisualsStatics<'s, T>,
}

/// [`DebugVisualized`] with `'static` lifetime of debug visuals.
pub type DebugVisualizedStatic<T> = DebugVisualized<'static, T>;

#[derive(Debug)]
pub struct DebugVisualsStatics<'s, T> {
    pub shader: &'s Shader,
    pub draw_params: &'s DrawParameters<'s>,

    _phantom: PhantomData<fn() -> T>
}

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn switch_enable() {
    let is_enabled = ENABLED.load(Ordering::Acquire);
    ENABLED.store(!is_enabled, Ordering::Release);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
}

implement_vertex!(Vertex, pos, color);

#[repr(transparent)]
#[derive(Debug, Deref)]
struct ShaderWrapper(Shader);

unsafe impl Send for ShaderWrapper { }
unsafe impl Sync for ShaderWrapper { }

#[repr(transparent)]
#[derive(Debug, Deref)]
struct DrawParametersWrapper<'a>(DrawParameters<'a>);

unsafe impl<'a> Send for DrawParametersWrapper<'a> { }
unsafe impl<'a> Sync for DrawParametersWrapper<'a> { }