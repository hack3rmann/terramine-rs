pub mod render_device;
pub mod bind_group;
pub mod buffer;
pub mod storage_buffer;
pub mod bind_group_layout;
pub mod texture;
pub mod macros;
pub mod buffer_vec;
pub mod uniform_buffer;
pub mod pipeline;
pub mod shader;
pub mod pipeline_cache;
pub mod render_pass;

use wgpu::{VertexBufferLayout, BufferAddress, VertexStepMode, VertexAttribute};

pub use {
    render_device::*,
    bind_group::*,
    bind_group_layout::*,
    buffer::*,
    storage_buffer::*,
    texture::*,
    buffer_vec::*,
    uniform_buffer::*,
    shader::*,
    pipeline::*,
    pipeline_cache::*,
};

/// Describes how the vertex buffer is interpreted.
///
/// For use in [`VertexState`][wgpu::VertexState].
///
/// Corresponds to [WebGPU `GPUVertexBufferLayout`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurenderpassdescriptor).
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct OwnedVertexBufferLayout {
    /// The stride, in bytes, between elements of this buffer.
    pub array_stride: BufferAddress,
    /// How often this vertex buffer is "stepped" forward.
    pub step_mode: VertexStepMode,
    /// The list of attributes which comprise a single vertex.
    pub attributes: Vec<VertexAttribute>,
}
static_assertions::assert_impl_all!(OwnedVertexBufferLayout: Send, Sync);

impl OwnedVertexBufferLayout {
    pub fn get_borrowed(&self) -> VertexBufferLayout<'_> {
        VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes: &self.attributes,
        }
    }
}