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
