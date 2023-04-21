use {
    crate::{
        prelude::*,
        graphics::shader::Shader,
    },
    wgpu::{*, util::DeviceExt},
};

pub trait Bufferizable {
    const ATTRS: &'static [VertexAttribute];
    const BUFFER_LAYOUT: VertexBufferLayout<'static>;
}

pub trait Renderable {
    type Error: std::error::Error;
    fn render<'rp, 's: 'rp>(&'s self, render_pass: &mut RenderPass<'rp>) -> Result<(), Self::Error>;
}

/// Generic mesh. Contains vertex buffer, shader and pipeline
#[derive(Debug)]
pub struct Mesh<V> {
    pub vertices: Buffer,
    pub n_vertices: usize,
    
    pub shared: MeshSharedResources,

    _vertex_marker: PhantomData<V>,
}

#[derive(Clone, Debug)]
pub struct MeshSharedResources {
    pub shader: Arc<Shader>,
    pub pipeline: Arc<RenderPipeline>,
    pub pipeline_layout: Arc<PipelineLayout>,
    pub fragment_targets: Arc<[Option<ColorTargetState>]>,
    pub bind_group_layouts: Arc<[Arc<BindGroupLayout>]>,
    pub label: Arc<String>,
    pub device: Arc<Device>,
    pub polygon_mode: PolygonMode,
    pub primitive_topology: PrimitiveTopology,
}

impl MeshSharedResources {
    pub fn new<V>(desc: MeshDescriptor) -> Self
    where
        V: Pod + Zeroable + Bufferizable,
    {
        let device = desc.device;
        let bind_group_layouts: Vec<_> = desc.bind_group_layouts.iter()
            .map(Arc::as_ref)
            .collect();

        let pipeline_layout = device.create_pipeline_layout(
            &PipelineLayoutDescriptor {
                label: Some(&desc.label),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            },
        );

        let pipeline = Mesh::<V>::create_pipeline(
            &device,
            &desc.shader,
            &desc.fragment_targets,
            desc.primitive_topology,
            desc.polygon_mode,
            &*desc.label,
            &pipeline_layout,
        );

        Self {
            shader: desc.shader,
            pipeline: Arc::new(pipeline),
            pipeline_layout: Arc::new(pipeline_layout),
            fragment_targets: desc.fragment_targets,
            bind_group_layouts: desc.bind_group_layouts,
            label: desc.label,
            device,
            polygon_mode: desc.polygon_mode,
            primitive_topology: desc.primitive_topology,
        }
    }
}

static_assertions::assert_impl_all!(MeshSharedResources: Send, Sync);

impl<V: Bufferizable + Send + Sync> Mesh<V> {
    const __MESH_ASSERT_IMPL_SEND_SYNC: fn() = || {
        fn assert_impl_all<T: Send + Sync>() { }
        assert_impl_all::<Mesh<V>>();
    };
}

#[derive(Debug)]
pub struct MeshDescriptor {
    pub primitive_topology: PrimitiveTopology,
    pub polygon_mode: PolygonMode,
    pub device: Arc<Device>,
    pub shader: Arc<Shader>,
    pub label: Arc<String>,
    pub fragment_targets: Arc<[Option<ColorTargetState>]>,
    pub bind_group_layouts: Arc<[Arc<BindGroupLayout>]>,
}

impl<V> Mesh<V> {
    pub fn from_shared_src(shared: MeshSharedResources, vertices: &[V]) -> Self
    where
        V: Pod + Zeroable + Bufferizable,
    {
        let MeshSharedResources {
            shader, fragment_targets, bind_group_layouts,
            label, device, polygon_mode, primitive_topology, ..
        } = shared;

        Mesh::new(MeshDescriptor {
            primitive_topology,
            polygon_mode,
            device,
            shader,
            label,
            fragment_targets,
            bind_group_layouts,
        }, vertices)
    }

    pub fn new(desc: MeshDescriptor, vertices: &[V]) -> Self
    where
        V: Pod + Zeroable + Bufferizable,
    {
        let vbuffer = desc.device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some(&desc.label),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            },
        );

        Self {
            shared: MeshSharedResources::new::<V>(desc),
            vertices: vbuffer,
            n_vertices: vertices.len(),
            _vertex_marker: PhantomData
        }
    }

    // TODO: optimize by reusing previous capacity.
    pub fn replace_vertices(&mut self, vertices: &[V])
    where
        V: Pod + Zeroable,
    {
        self.vertices = self.shared.device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some(&self.shared.label),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            },
        );
        self.n_vertices = vertices.len();
    }

    pub fn reload_shader(&mut self, shader: Arc<Shader>)
    where
        V: Bufferizable,
    {
        self.shared.shader = shader;
        self.shared.pipeline = Arc::new(Self::create_pipeline(
            &self.shared.device,
            &self.shared.shader,
            &self.shared.fragment_targets,
            self.shared.primitive_topology,
            self.shared.polygon_mode,
            &*self.shared.label,
            &self.shared.pipeline_layout,
        ));
    }

    pub fn is_empty(&self) -> bool {
        self.n_vertices == 0
    }

    fn create_pipeline(
        device: &Device, shader: &ShaderModule, fragment_targets: &[Option<ColorTargetState>],
        primitive_topology: PrimitiveTopology, polygon_mode: PolygonMode, label: impl AsRef<str>,
        pipeline_layout: &PipelineLayout,
    ) -> RenderPipeline
    where
        V: Bufferizable
    {
        device.create_render_pipeline(
            &RenderPipelineDescriptor {
                label: Some(label.as_ref()),

                layout: Some(pipeline_layout),

                vertex: VertexState {
                    module: shader,
                    entry_point: "vs_main",
                    buffers: &[V::BUFFER_LAYOUT],
                },

                fragment: Some(FragmentState {
                    module: shader,
                    entry_point: "fs_main",
                    targets: fragment_targets,
                }),

                primitive: PrimitiveState {
                    topology: primitive_topology,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    polygon_mode,
                    unclipped_depth: false,
                    conservative: false,
                },

                depth_stencil: None,

                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },

                multiview: None,
            },
        )
    }
}

impl<V: Bufferizable> Renderable for Mesh<V> {
    type Error = !;
    fn render<'rp, 's: 'rp>(&'s self, render_pass: &mut RenderPass<'rp>) -> Result<(), !> {
        if self.is_empty() { return Ok(()) }

        render_pass.set_pipeline(&self.shared.pipeline);
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.draw(0..self.n_vertices as u32, 0..1);

        Ok(())
    }
}