use {
    crate::{
        prelude::*,
        graphics::shader::Shader,
    },
    wgpu::{Buffer, RenderPipeline, Device, util::DeviceExt},
};

pub trait Bufferizable {
    const ATTRS: &'static [wgpu::VertexAttribute];
    const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static>;
}

/// Generic mesh. Contains vertex buffer, shader and pipeline
#[derive(Debug)]
pub struct Mesh<V> {
    pub vertices: Buffer,
    pub n_vertices: usize,

    pub shader: Arc<Shader>,
    pub pipeline: Arc<RenderPipeline>,
    pub device: Arc<Device>,
    pub label: String,

    _vertex_marker: PhantomData<V>,
}

impl<V: Bufferizable + Send + Sync> Mesh<V> {
    const __MESH_ASSERT_IMPL_SEND_SYNC: fn() = || {
        fn assert_impl_all<T: Send + Sync>() { }
        assert_impl_all::<Mesh<V>>();
    };
}

impl<V: Bufferizable> Mesh<V> {
    pub fn new(
        device: Arc<Device>, vertices: &[V], shader: Arc<Shader>,
        label: impl Into<String>, fragment_targets: &[Option<wgpu::ColorTargetState>],
        primitive_topology: wgpu::PrimitiveTopology,
        polygon_mode: wgpu::PolygonMode,
    ) -> Self
    where
        V: Pod + Zeroable,
    {
        let label = label.into();

        let vbuffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&label),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some(&label),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            },
        );

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(&label),

                layout: Some(&pipeline_layout),

                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[V::BUFFER_LAYOUT],
                },

                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: fragment_targets,
                }),

                primitive: wgpu::PrimitiveState {
                    topology: primitive_topology,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode,
                    unclipped_depth: false,
                    conservative: false,
                },

                depth_stencil: None,

                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },

                multiview: None,
            },
        );

        Self {
            shader,
            device,
            label,
            vertices: vbuffer,
            n_vertices: vertices.len(),
            pipeline: Arc::new(pipeline),
            _vertex_marker: PhantomData
        }
    }

    pub fn replace_vertices(&mut self, vertices: &[V])
    where
        V: Pod + Zeroable,
    {
        self.vertices = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&self.label),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        self.n_vertices = vertices.len();
    }

    pub fn render<'rp, 's: 'rp>(&'s self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.draw(0..self.n_vertices as u32, 0..1);
    }
}