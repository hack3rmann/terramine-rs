// use {
//     crate::{
//         prelude::*,
//         graphics::render_resource::{BindGroupLayout, Shader},
//     },
//     wgpu::{
//         BufferAddress, ColorTargetState, DepthStencilState, MultisampleState, PrimitiveState,
//         PushConstantRange, VertexAttribute, VertexFormat, VertexStepMode,
//     },
// };

// crate::define_atomic_id!(RenderPipelineId);

// /// A [`RenderPipeline`] represents a graphics pipeline and its stages (shaders), bindings and vertex buffers.
// ///
// /// May be converted from and dereferences to a wgpu [`RenderPipeline`](wgpu::RenderPipeline).
// /// Can be created via [`RenderDevice::create_render_pipeline`](crate::renderer::RenderDevice::create_render_pipeline).
// #[derive(Clone, Debug, Deref)]
// pub struct RenderPipeline {
//     pub id: RenderPipelineId,
//     #[deref]
//     pub value: Arc<wgpu::RenderPipeline>,
// }
// assert_impl_all!(RenderPipeline: Send, Sync);

// impl From<wgpu::RenderPipeline> for RenderPipeline {
//     fn from(value: wgpu::RenderPipeline) -> Self {
//         Self {
//             id: RenderPipelineId::new(),
//             value: Arc::new(value),
//         }
//     }
// }

// crate::define_atomic_id!(ComputePipelineId);

// /// A [`ComputePipeline`] represents a compute pipeline and its single shader stage.
// ///
// /// May be converted from and dereferences to a wgpu [`ComputePipeline`](wgpu::ComputePipeline).
// /// Can be created via [`RenderDevice::create_compute_pipeline`](crate::renderer::RenderDevice::create_compute_pipeline).
// #[derive(Clone, Debug, Deref)]
// pub struct ComputePipeline {
//     pub id: ComputePipelineId,
//     #[deref]
//     pub value: Arc<wgpu::ComputePipeline>,
// }
// assert_impl_all!(ComputePipeline: Send, Sync);

// impl From<wgpu::ComputePipeline> for ComputePipeline {
//     fn from(value: wgpu::ComputePipeline) -> Self {
//         Self {
//             id: ComputePipelineId::new(),
//             value: Arc::new(value),
//         }
//     }
// }

// /// Describes a render (graphics) pipeline.
// #[derive(Clone, Debug, PartialEq)]
// pub struct RenderPipelineDescriptor {
//     /// Debug label of the pipeline. This will show up in graphics debuggers for easy identification.
//     pub label: Option<Cow<'static, str>>,
//     /// The layout of bind groups for this pipeline.
//     pub layout: Vec<BindGroupLayout>,
//     /// The push constant ranges for this pipeline.
//     /// Supply an empty vector if the pipeline doesn't use push constants.
//     pub push_constant_ranges: Vec<PushConstantRange>,
//     /// The compiled vertex stage, its entry point, and the input buffers layout.
//     pub vertex: VertexState,
//     /// The properties of the pipeline at the primitive assembly and rasterization level.
//     pub primitive: PrimitiveState,
//     /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
//     pub depth_stencil: Option<DepthStencilState>,
//     /// The multi-sampling properties of the pipeline.
//     pub multisample: MultisampleState,
//     /// The compiled fragment stage, its entry point, and the color targets.
//     pub fragment: Option<FragmentState>,
// }
// assert_impl_all!(RenderPipelineDescriptor: Send, Sync);

// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct VertexState {
//     /// The compiled shader module for this stage.
//     pub shader: Arc<Shader>,
//     pub shader_defs: Vec<ShaderDefVal>,
//     /// The name of the entry point in the compiled shader. There must be a
//     /// function with this name in the shader.
//     pub entry_point: Cow<'static, str>,
//     /// The format of any vertex buffers used with this pipeline.
//     pub buffers: Vec<VertexBufferLayout>,
// }
// assert_impl_all!(VertexState: Send, Sync);

// /// Describes how the vertex buffer is interpreted.
// #[derive(Default, Clone, Debug, Hash, Eq, PartialEq)]
// pub struct VertexBufferLayout {
//     /// The stride, in bytes, between elements of this buffer.
//     pub array_stride: BufferAddress,
//     /// How often this vertex buffer is "stepped" forward.
//     pub step_mode: VertexStepMode,
//     /// The list of attributes which comprise a single vertex.
//     pub attributes: Vec<VertexAttribute>,
// }
// assert_impl_all!(VertexBufferLayout: Send, Sync);

// impl VertexBufferLayout {
//     /// Creates a new densely packed [`VertexBufferLayout`] from an iterator of vertex formats.
//     /// Iteration order determines the `shader_location` and `offset` of the [`VertexAttributes`](VertexAttribute).
//     /// The first iterated item will have a `shader_location` and `offset` of zero.
//     /// The `array_stride` is the sum of the size of the iterated [`VertexFormats`](VertexFormat) (in bytes).
//     pub fn from_vertex_formats<T: IntoIterator<Item = VertexFormat>>(
//         step_mode: VertexStepMode,
//         vertex_formats: T,
//     ) -> Self {
//         let mut offset = 0;
//         let mut attributes = Vec::new();
//         for (shader_location, format) in vertex_formats.into_iter().enumerate() {
//             attributes.push(VertexAttribute {
//                 format,
//                 offset,
//                 shader_location: shader_location as u32,
//             });
//             offset += format.size();
//         }

//         VertexBufferLayout {
//             array_stride: offset,
//             step_mode,
//             attributes,
//         }
//     }
// }

// /// Describes the fragment process in a render pipeline.
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct FragmentState {
//     /// The compiled shader module for this stage.
//     pub shader: Arc<Shader>,
//     pub shader_defs: Vec<ShaderDefVal>,
//     /// The name of the entry point in the compiled shader. There must be a
//     /// function with this name in the shader.
//     pub entry_point: Cow<'static, str>,
//     /// The color state of the render targets.
//     pub targets: Vec<Option<ColorTargetState>>,
// }
// assert_impl_all!(FragmentState: Send, Sync);

// /// Describes a compute pipeline.
// #[derive(Clone, Debug)]
// pub struct ComputePipelineDescriptor {
//     pub label: Option<Cow<'static, str>>,
//     pub layout: Vec<BindGroupLayout>,
//     pub push_constant_ranges: Vec<PushConstantRange>,
//     /// The compiled shader module for this stage.
//     pub shader: Arc<Shader>,
//     pub shader_defs: Vec<ShaderDefVal>,
//     /// The name of the entry point in the compiled shader. There must be a
//     /// function with this name in the shader.
//     pub entry_point: Cow<'static, str>,
// }
// assert_impl_all!(ComputePipelineDescriptor: Send, Sync);