use crate::{
    prelude::*,
    graphics::{Device, PrimitiveState, Material},
};



pub use wgpu::PipelineLayoutDescriptor;



#[derive(Debug)]
pub struct RenderPipelineDescriptor<'s> {
    pub device: &'s Device,
    pub material: &'s dyn Material,
    pub primitive_state: PrimitiveState,
    pub label: Option<StaticStr>,
    pub layout: &'s wgpu::PipelineLayout,
}



#[derive(Debug, Deref)]
pub struct RenderPipeline {
    #[deref]
    pub inner: wgpu::RenderPipeline,
    pub label: Option<StaticStr>,
}
assert_impl_all!(RenderPipeline: Send, Sync);

impl RenderPipeline {
    pub fn new(desc: RenderPipelineDescriptor) -> Self {
        let shader = desc.material.get_shader().as_module();

        let pipeline = desc.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: desc.label.as_deref(),
                layout: Some(desc.layout),

                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: cfg::shader::WGSL_VERTEX_ENTRY,
                    buffers: &shader.vertex_layout,
                },

                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: cfg::shader::WGSL_FRAGMENT_ENTRY,
                    targets: desc.material.get_color_states(),
                }),

                primitive: desc.primitive_state,

                depth_stencil: None,

                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },

                multiview: None,
            },
        );

        Self { inner: pipeline, label: desc.label }
    }
}

impl From<wgpu::RenderPipeline> for RenderPipeline {
    fn from(value: wgpu::RenderPipeline) -> Self {
        Self { inner: value, label: None }
    }
}



#[derive(Debug, Deref)]
pub struct PipelineLayout {
    pub inner: wgpu::PipelineLayout,
}
assert_impl_all!(PipelineLayout: Send, Sync);

impl PipelineLayout {
    pub fn new(device: &Device, desc: &PipelineLayoutDescriptor) -> Self {
        let layout = device.create_pipeline_layout(desc);
        Self::from(layout)
    }
}

impl From<wgpu::PipelineLayout> for PipelineLayout {
    fn from(value: wgpu::PipelineLayout) -> Self {
        Self { inner: value }
    }
}
