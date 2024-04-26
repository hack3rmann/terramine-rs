use {
    crate::prelude::*,
    graphics::{
        Device, Queue, FromFile, Extent3d, Texture, TextureView,
        Sampler,
    },
    image::{ImageBuffer, Rgba, ImageError},
    tokio::{fs, io},
};



pub use wgpu::TextureFormat;



pub type RgbaImageBuffer = ImageBuffer<Rgba<u8>, Vec<u8>>;



#[derive(Clone, Debug, PartialEq, Deref, Default, From, Into)]
pub struct Image {
    pub inner: RgbaImageBuffer,
}
assert_impl_all!(Image: Send, Sync);

impl Image {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LoadImageError> {
        let image = image::load_from_memory(bytes)?.to_rgba8();
        Ok(Self::from(image))
    }

    pub fn dimensions(&self) -> UInt2 {
        self.inner.dimensions().into()
    }

    pub fn extent_size(&self) -> Extent3d {
        let (width, height) = self.dimensions().as_tuple();
        Extent3d { width, height, depth_or_array_layers: 1 }
    }
}

impl FromFile for Image {
    type Error = LoadImageError;

    async fn from_file(file_name: impl AsRef<Path> + Send) -> Result<Self, Self::Error> {
        let dir = Path::new(cfg::texture::DIRECTORY);

        let bytes = fs::read(dir.join(file_name)).await?;

        Self::from_bytes(&bytes)
    }
}



macros::sum_errors! {
    pub enum LoadImageError { Io => io::Error, Load => ImageError }
}



#[derive(Debug, Clone)]
pub struct GpuImageDescriptor<'s, S> {
    pub image: &'s Image,
    pub label: Option<S>,
}
assert_impl_all!(GpuImageDescriptor<&'static str>: Send, Sync);



#[derive(Debug, Clone)]
pub struct GpuImage {
    pub texture: Texture,
    pub sampler: Sampler,
    pub view: TextureView,
    pub label: Option<StaticStr>,
}

impl GpuImage {
    pub fn new<S: Into<StaticStr>>(
        device: &Device, queue: &Queue, desc: GpuImageDescriptor<'_, S>,
    ) -> Self {
        use crate::graphics::texture::*;

        let format = TextureFormat::Rgba8UnormSrgb;

        let label = desc.label.map(Into::into);

        let texture = Texture::new(
            device,
            queue,
            desc.image,
            &TextureDescriptor {
                label: label.as_deref(),
                size: desc.image.extent_size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
        );

        texture.write(queue, desc.image);

        let view = texture.create_view(&default());

        // FIXME: move sampler settings out of here
        let sampler = Sampler::new(
            device,
            &SamplerDescriptor {
                label: label.as_deref(),
                min_filter: FilterMode::Linear,
                ..default()
            },
        );

        Self { texture, sampler, view, label }
    }
    
    pub fn format(&self) -> TextureFormat {
        self.texture.format()
    }
}