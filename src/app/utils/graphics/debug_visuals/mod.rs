use std::ops::Deref;

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
pub struct DebugVisualized<'s, T> {
    pub inner: T,
    pub mesh: UnindexedMesh<Vertex>,
    pub static_data: DebugVisualsStatics<'s, T>,
}

impl<T> Deref for DebugVisualized<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for DebugVisualized<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

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