use crate::{
    prelude::*,
    graphics::Shader,
};



#[derive(Clone, Debug, Default)]
pub enum ShaderRef {
    #[default]
    Default,
    Module(Arc<Shader>),
}
assert_impl_all!(ShaderRef: Send, Sync);

impl ShaderRef {
    pub fn as_module(&self) -> Arc<Shader> {
        match self {
            Self::Default => todo!(),
            Self::Module(module) => Arc::clone(module),
        }
    }
}
