use {
    crate::{
        prelude::*,
        graphics::{VertexBufferLayout, Device, FromFile},
    },
    tokio::{fs, io},
};



pub use wgpu::{ShaderModuleDescriptor, ShaderStages};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderSource {
    pub source: ShaderSourceCode,
}
assert_impl_all!(ShaderSource: Send, Sync);

impl<Source: Into<ShaderSourceCode>> From<Source> for ShaderSource {
    fn from(value: Source) -> Self {
        let source = value.into();
        Self { source }
    }
}

impl FromFile for ShaderSource {
    type Error = io::Error;
    
    async fn from_file(file_name: impl AsRef<Path> + Send) -> Result<Self, Self::Error> {
        let dir = Path::new(cfg::shader::DIRECTORY);
        let source = fs::read_to_string(dir.join(file_name)).await?;
        Ok(Self::from(source))
    }
}



#[derive(Debug)]
pub struct Shader {
    pub module: wgpu::ShaderModule,
    pub vertex_layout: Vec<VertexBufferLayout<'static>>,
}
assert_impl_all!(Shader: Send, Sync);

impl Shader {
    pub fn new(
        device: &Device,
        source: impl Into<ShaderSource>,
        vertex_layout: Vec<VertexBufferLayout<'static>>,
    ) -> Self {
        let source: ShaderSource = source.into();

        let module = device.create_shader_module(
            ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(source.source.into()),
            },
        );

        Self { module, vertex_layout }
    }
}



pub type ShaderSourceCode = StaticStr;