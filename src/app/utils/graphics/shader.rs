use crate::{
    prelude::*,
    graphics::VertexBufferLayout,
};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderSource {
    pub source: ShaderSourceCode,
}
assert_impl_all!(ShaderSource: Send, Sync);

impl From<ShaderSourceCode> for ShaderSource {
    fn from(value: ShaderSourceCode) -> Self {
        Self { source: value }
    }
}



#[derive(Debug)]
pub struct Shader {
    pub module: wgpu::ShaderModule,
    pub vertex_layout: Vec<VertexBufferLayout<'static>>,
}
assert_impl_all!(Shader: Send, Sync);



pub type ShaderSourceCode = StaticStr;