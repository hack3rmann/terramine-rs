use {
    crate::{
        graphics::{
            Buffer, texture::Texture, Sampler,
            render_resource::{bind_group::BindGroup, bind_group_layout::BindGroupLayout},
        },
        prelude::*,
    },
    wgpu::util::DeviceExt,
};

#[derive(Debug)]
pub struct RenderDevice {
    pub device: Arc<wgpu::Device>,
}
assert_impl_all!(RenderDevice: Send, Sync);

impl From<wgpu::Device> for RenderDevice {
    fn from(device: wgpu::Device) -> Self {
        Self { device: Arc::new(device) }
    }
}

impl RenderDevice {
    /// List all [`Features`](wgpu::Features) that may be used with this device.
    ///
    /// Functions may panic if you use unsupported features.
    pub fn features(&self) -> wgpu::Features {
        self.device.features()
    }

    /// List all [`Limits`](wgpu::Limits) that were requested of this device.
    ///
    /// If any of these limits are exceeded, functions may panic.
    pub fn limits(&self) -> wgpu::Limits {
        self.device.limits()
    }

    /// Creates a [`ShaderModule`][wgpu::ShaderModule] from either SPIR-V or WGSL source code.
    pub fn create_shader_module(&self, desc: wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule {
        self.device.create_shader_module(desc)
    }

    /// Check for resource cleanups and mapping callbacks.
    ///
    /// Return `true` if the queue is empty, or `false` if there are more queue
    /// submissions still in flight. (Note that, unless access to the [`Queue`][wgpu::Queue] is
    /// coordinated somehow, this information could be out of date by the time
    /// the caller receives it. `Queue`s can be shared between threads, so
    /// other threads could submit new work at any time.)
    ///
    /// On the web, this is a no-op. `Device`s are automatically polled.
    pub fn poll(&self, maintain: wgpu::Maintain) {
        self.device.poll(maintain);
    }

    /// Creates an empty [`CommandEncoder`](wgpu::CommandEncoder).
    pub fn create_command_encoder(
        &self,
        desc: &wgpu::CommandEncoderDescriptor,
    ) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(desc)
    }

    /// Creates an empty [`RenderBundleEncoder`](wgpu::RenderBundleEncoder).
    pub fn create_render_bundle_encoder(
        &self,
        desc: &wgpu::RenderBundleEncoderDescriptor,
    ) -> wgpu::RenderBundleEncoder {
        self.device.create_render_bundle_encoder(desc)
    }

    /// Creates a new [`BindGroup`](wgpu::BindGroup).
    pub fn create_bind_group(&self, desc: &wgpu::BindGroupDescriptor) -> BindGroup {
        let wgpu_bind_group = self.device.create_bind_group(desc);
        BindGroup::from(wgpu_bind_group)
    }

    /// Creates a [`BindGroupLayout`](wgpu::BindGroupLayout).
    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor,
    ) -> BindGroupLayout {
        BindGroupLayout::from(self.device.create_bind_group_layout(desc))
    }

    /// Creates a [`PipelineLayout`](wgpu::PipelineLayout).
    pub fn create_pipeline_layout(
        &self,
        desc: &wgpu::PipelineLayoutDescriptor,
    ) -> wgpu::PipelineLayout {
        self.device.create_pipeline_layout(desc)
    }

    // /// Creates a [`RenderPipeline`].
    // pub fn create_render_pipeline(&self, desc: &RawRenderPipelineDescriptor) -> RenderPipeline {
    //     let wgpu_render_pipeline = self.device.create_render_pipeline(desc);
    //     RenderPipeline::from(wgpu_render_pipeline)
    // }

    // /// Creates a [`ComputePipeline`].
    // pub fn create_compute_pipeline(
    //     &self,
    //     desc: &wgpu::ComputePipelineDescriptor,
    // ) -> ComputePipeline {
    //     let wgpu_compute_pipeline = self.device.create_compute_pipeline(desc);
    //     ComputePipeline::from(wgpu_compute_pipeline)
    // }

    /// Creates a [`Buffer`].
    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> Buffer {
        let wgpu_buffer = self.device.create_buffer(desc);
        Buffer::from(wgpu_buffer)
    }

    /// Creates a [`Buffer`] and initializes it with the specified data.
    pub fn create_buffer_with_data(&self, desc: &wgpu::util::BufferInitDescriptor) -> Buffer {
        let wgpu_buffer = self.device.create_buffer_init(desc);
        Buffer::from(wgpu_buffer)
    }

    /// Creates a new [`Texture`] and initializes it with the specified data.
    ///
    /// `desc` specifies the general format of the texture.
    /// `data` is the raw data.
    pub fn create_texture_with_data(
        &self,
        render_queue: &RenderQueue,
        desc: &wgpu::TextureDescriptor,
        data: &[u8],
    ) -> Texture {
        self.device.create_texture_with_data(render_queue.as_ref(), desc, data).into()
    }

    /// Creates a new [`Texture`].
    ///
    /// `desc` specifies the general format of the texture.
    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor) -> Texture {
        self.device.create_texture(desc).into()
    }

    /// Creates a new [`Sampler`].
    ///
    /// `desc` specifies the behavior of the sampler.
    pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> Sampler {
        let wgpu_sampler = self.device.create_sampler(desc);
        Sampler::from(wgpu_sampler)
    }

    /// Initializes [`Surface`](wgpu::Surface) for presentation.
    ///
    /// # Panics
    ///
    /// - A old [`SurfaceTexture`](wgpu::SurfaceTexture) is still alive referencing an old surface.
    /// - Texture format requested is unsupported on the surface.
    pub fn configure_surface(&self, surface: &wgpu::Surface, config: &wgpu::SurfaceConfiguration) {
        surface.configure(&self.device, config);
    }

    pub fn map_buffer(
        &self,
        buffer: &wgpu::BufferSlice,
        map_mode: wgpu::MapMode,
        callback: impl FnOnce(Result<(), wgpu::BufferAsyncError>) + Send + 'static,
    ) {
        buffer.map_async(map_mode, callback);
    }

    pub fn align_copy_bytes_per_row(row_bytes: usize) -> usize {
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - row_bytes % align) % align;
        row_bytes + padded_bytes_per_row_padding
    }

    pub fn get_supported_read_only_binding_type(
        &self,
        buffers_per_shader_stage: u32,
    ) -> wgpu::BufferBindingType {
        if self.limits().max_storage_buffers_per_shader_stage >= buffers_per_shader_stage {
            wgpu::BufferBindingType::Storage { read_only: true }
        } else {
            wgpu::BufferBindingType::Uniform
        }
    }
}



#[derive(Debug, Deref, Clone)]
pub struct RenderQueue(pub Arc<wgpu::Queue>);
assert_impl_all!(RenderQueue: Send, Sync);

#[derive(Debug, Clone, Deref)]
pub struct RenderAdapter(pub Arc<wgpu::Adapter>);
assert_impl_all!(RenderAdapter: Send, Sync);

#[derive(Debug, Deref)]
pub struct RenderInstance(pub wgpu::Instance);
assert_impl_all!(RenderInstance: Send, Sync);

#[derive(Clone, Debug, Eq, PartialEq, Deref)]
pub struct RenderAdapterInfo(pub wgpu::AdapterInfo);
assert_impl_all!(RenderAdapterInfo: Send, Sync);