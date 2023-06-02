use {
    crate::prelude::*,
    graphics::{
        Device, Queue, FromFile, Extent3d, Texture, TextureView,
        Sampler, TextureFormat, BindGroupLayout, BindGroup, AsBindGroup,
    },
    image::{ImageBuffer, Rgba, ImageError},
    tokio::{fs, io},
};



pub type RgbaImageBuffer = ImageBuffer<Rgba<u8>, Vec<u8>>;



#[derive(Clone, Debug, PartialEq, Deref, Default, From, Into)]
pub struct Image {
    pub inner: RgbaImageBuffer,
}
assert_impl_all!(Image: Send, Sync);

impl Image {
    pub fn dimensions(&self) -> UInt2 {
        self.inner.dimensions().into()
    }

    pub fn extent_size(&self) -> Extent3d {
        let (width, height) = self.dimensions().as_tuple();
        Extent3d { width, height, depth_or_array_layers: 1 }
    }
}

#[async_trait]
impl FromFile for Image {
    type Error = LoadImageError;
    async fn from_file(file_name: impl AsRef<Path> + Send) -> Result<Self, Self::Error> {
        let dir = Path::new(cfg::texture::DIRECTORY);

        let bytes = fs::read(dir.join(file_name)).await?;
        let image = image::load_from_memory(&bytes)?.to_rgba8();

        Ok(Self::from(image))
    }
}



macros::sum_errors! {
    pub enum LoadImageError { Io => io::Error, Load => ImageError }
}



#[derive(Debug, Clone)]
pub struct GpuImageDescriptor<'s> {
    pub device: &'s Device,
    pub queue: &'s Queue,
    pub image: &'s Image,
    pub label: Option<StaticStr>,
}
assert_impl_all!(GpuImageDescriptor: Send, Sync);



#[derive(Debug, Clone)]
pub struct GpuImage {
    pub texture: Texture,
    pub sampler: Sampler,
    pub view: TextureView,
    pub format: TextureFormat,
    pub label: Option<StaticStr>,
}

impl GpuImage {
    pub fn new(desc: GpuImageDescriptor) -> Self {
        use crate::graphics::texture::*;

        let format = TextureFormat::Rgba8UnormSrgb;

        let texture = Texture::new(
            desc.device,
            desc.queue,
            desc.image,
            &TextureDescriptor {
                label: desc.label.as_deref(),
                size: desc.image.extent_size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
        );

        texture.write(desc.queue, desc.image);

        let view = texture.create_view(&default());

        let sampler = Sampler::new(
            desc.device,
            &SamplerDescriptor {
                label: desc.label.as_deref(),
                min_filter: FilterMode::Linear,
                ..default()
            },
        );

        Self { texture, sampler, view, format, label: desc.label }
    }
}

impl AsBindGroup for GpuImage {
    fn bind_group_layout(device: &Device) -> BindGroupLayout
    where
        Self: Sized,
    {
        use crate::graphics::*;

        device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("gpu_image_binds"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            },
        ).into()
    }

    fn as_bind_group(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
        use crate::graphics::{BindGroupDescriptor, BindGroupEntry, BindingResource};

        device.create_bind_group(
            &BindGroupDescriptor {
                label: self.label.as_deref(),
                layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&self.view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                ],
            },
        ).into()
    }
}
