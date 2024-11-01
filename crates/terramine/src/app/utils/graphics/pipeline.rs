use crate::{
    prelude::*,
    graphics::{Device, PrimitiveState, ColorTargetState, Shader}, define_atomic_id,
};



pub use wgpu::PipelineLayoutDescriptor;



#[derive(Debug)]
pub struct RenderPipelineDescriptor<'s> {
    pub shader: &'s Shader,
    pub color_states: &'s [Option<ColorTargetState>],
    pub primitive_state: PrimitiveState,
    pub label: Option<StaticStr>,
    pub layout: &'s wgpu::PipelineLayout,
}



define_atomic_id!(PipelineId);

#[derive(Debug, Clone)]
pub struct RenderPipeline {
    pub inner: Arc<wgpu::RenderPipeline>,
    pub id: PipelineId,
    pub label: Option<StaticStr>,
}
assert_impl_all!(RenderPipeline: Send, Sync);

impl RenderPipeline {
    pub fn new(device: &Device, desc: RenderPipelineDescriptor<'_>) -> Self {
        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: desc.label.as_deref(),
                layout: Some(desc.layout),

                vertex: wgpu::VertexState {
                    module: &desc.shader.module,
                    entry_point: cfg::shader::WGSL_VERTEX_ENTRY_NAME,
                    buffers: &desc.shader.vertex_layout,
                },

                fragment: Some(wgpu::FragmentState {
                    module: &desc.shader.module,
                    entry_point: cfg::shader::WGSL_FRAGMENT_ENTRY_NAME,
                    targets: desc.color_states,
                }),

                primitive: desc.primitive_state,

                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: default(),
                    bias: default(),
                }),

                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },

                multiview: None,
            },
        );

        Self {
            inner: Arc::new(pipeline),
            label: desc.label,
            id: PipelineId::new(),
        }
    }

    pub fn id(&self) -> PipelineId {
        self.id
    }
}

impl From<wgpu::RenderPipeline> for RenderPipeline {
    fn from(value: wgpu::RenderPipeline) -> Self {
        Self { inner: Arc::new(value), label: None, id: PipelineId::new() }
    }
}

impl Deref for RenderPipeline {
    type Target = wgpu::RenderPipeline;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
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



/// Cache for all of the pipelines. See [`PipelineBound`]
#[derive(Debug, Default)]
pub struct PipelineCache {
    pub(crate) pipelines: HashMap<PipelineId, RenderPipeline>,
}
assert_impl_all!(PipelineCache: Send, Sync, Component);

impl PipelineCache {
    pub fn new() -> Self {
        Self { pipelines: default() }
    }

    pub fn insert(&mut self, pipeline: RenderPipeline) -> Option<RenderPipeline> {
        self.pipelines.insert(pipeline.id, pipeline)
    }

    pub fn get(&self, bound: &impl PipelineBound) -> Option<&RenderPipeline> {
        self.pipelines.get(&bound.id())
    }
}



/// A trait to mark specific component as a pipeline bound
pub trait PipelineBound: Sized {
    fn id(&self) -> PipelineId;
    fn from_pipeline(pipeline: &RenderPipeline) -> Self;
}