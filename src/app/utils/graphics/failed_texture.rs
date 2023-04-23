#![allow(dead_code)]

use {
    crate::prelude::*,
    wgpu::{*, Texture as WgpuTexture},
    std::path::Path,
    tokio::{fs, io},
};

#[derive(Debug)]
pub struct Texture {
    pub size: Extent3d,
    pub inner: WgpuTexture,
    pub bind_group: BindGroup,
    pub bind_group_layout: Arc<BindGroupLayout>,

    pub label: String,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}
assert_impl_all!(Texture: Send, Sync);

impl Texture {
    pub fn from_image_bytes(
        device: Arc<Device>, queue: Arc<Queue>,
        image_bytes: &[u8], label: impl Into<String>,
        texture_binding: u32, sampler_binding: u32,
    ) -> Self {
        let label = label.into();

        let image = image::load_from_memory(image_bytes)
            .expect("failed to load test image")
            .to_rgba8();

        let (width, height) = image.dimensions();

        let size = Extent3d { width, height, depth_or_array_layers: 1 };

        let texture = device.create_texture(
            &TextureDescriptor {
                label: Some(&label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
        );

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &image,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(width * mem::size_of::<f32>() as u32),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            size,
        );

        let view = texture.create_view(&default());

        let sampler = device.create_sampler(
            &SamplerDescriptor {
                label: Some(&format!("{label}_sampler")),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Nearest,
                ..default()
            },
        );

        let layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some(&format!("{label}_bind_group_layout_descriptor")),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: texture_binding,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: sampler_binding,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            },
        );

        let bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some(&format!("{label}_bind_group")),
                layout: &layout,
                entries: &[
                    BindGroupEntry {
                        binding: texture_binding,
                        resource: BindingResource::TextureView(&view),
                    },
                    BindGroupEntry {
                        binding: sampler_binding,
                        resource: BindingResource::Sampler(&sampler),
                    },
                ],
            },
        );

        Self { size, inner: texture, bind_group, label, device, queue, bind_group_layout: Arc::new(layout) }
    }

    pub async fn load_from_file(
        device: Arc<Device>, queue: Arc<Queue>,
        file_name: impl AsRef<Path>, label: impl Into<String>,
        texture_binding: u32, sampler_binding: u32,
    ) -> io::Result<Self> {
        let file_path = Path::new(cfg::texture::DIRECTORY).join(file_name);

        let _work_guard = logger::work!(from = "texture-loader", "loading from {file_path:?}");

        let image_bytes = fs::read(file_path).await?;

        Ok(Self::from_image_bytes(device, queue, &image_bytes, label, texture_binding, sampler_binding))
    }
}