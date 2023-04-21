#![allow(dead_code)]

use {
    crate::prelude::*,
    std::path::Path,
    wgpu::{ShaderModule, Device},
    tokio::{fs, io},
};

/// Wrapper around [`wgpu`]'s [`ShaderModule`].
#[derive(Debug, Deref)]
pub struct Shader {
    #[deref]
    inner: ShaderModule,

    device: Arc<Device>,
    label: String,
}

impl Shader {
    pub fn from_source(device: Arc<Device>, source_code: String, label: impl Into<String>) -> Self {
        let label = label.into();

        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some(&label),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(source_code)),
            },
        );

        Self { label, device, inner: shader }
    }

    pub async fn load_from_file(
        device: Arc<Device>, label: impl Into<String>, file_name: impl AsRef<Path>,
    ) -> io::Result<Self> {
        use cfg::shader::DIRECTORY;
        let file_name = file_name.as_ref();

        let _work_guard = logger::work!(from = "shader-loader", "loading from {file_name:?}");

        let source = fs::read_to_string(Path::new(DIRECTORY).join(file_name)).await?;

        Ok(Self::from_source(device, source, label))
    }
}