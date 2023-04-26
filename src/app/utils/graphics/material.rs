use {
    crate::prelude::*,
    std::fmt::Debug,
};

pub trait Material: Debug + Send + Sync + 'static {
    fn get_shader(&self) -> &wgpu::ShaderModule;
    fn get_polygon_mode(&self) -> wgpu::PolygonMode;

    // TODO: remind this ->
    fn get_pipeline(&self) -> &wgpu::RenderPipeline;
}
assert_obj_safe!(Material);



#[derive(Debug)]
pub struct DefaultMaterial {
    pub shader: wgpu::ShaderModule,
    pub polygon_mode: wgpu::PolygonMode,
    pub pipeline: wgpu::RenderPipeline,
}

impl Material for DefaultMaterial {
    fn get_polygon_mode(&self) -> wgpu::PolygonMode {
        self.polygon_mode
    }

    fn get_shader(&self) -> &wgpu::ShaderModule {
        &self.shader
    }

    fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
