use std::fmt::Debug;

pub trait Pipeline: Debug + Send + Sync + 'static {
    fn name(&self) -> &str;
    fn shader_source(&self) -> &str;
    fn primitive_state(&self) -> wgpu::PrimitiveState;
    fn color_target_states(&self) -> Vec<wgpu::ColorTargetState>;
    fn depth_stencil_state(&self) -> Option<wgpu::DepthStencilState>;
    fn vertex_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'_>>;
}
static_assertions::assert_obj_safe!(Pipeline);

#[derive(Debug)]
pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub layout: wgpu::PipelineLayout,
}

impl RenderPipeline {
    
}