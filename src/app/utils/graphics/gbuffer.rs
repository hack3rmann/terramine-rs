use crate::{prelude::*, graphics::*};



#[derive(Debug, Clone)]
pub struct GBuffer {
    depth: GpuImage,
    albedo: GpuImage,
    position: GpuImage,
    normal: GpuImage,
}

impl From<&GBuffer> for RenderNodeOutput {
    fn from(value: &GBuffer) -> Self {
        Self {
            color_targets: vec![
                value.albedo.view.clone(),
                value.position.view.clone(),
                value.normal.view.clone(),
            ],
            depth_target: Some(value.depth.view.clone()),
        }
    }
}

impl AsBindGroup for GBuffer {
    type Data = Self;

    fn label() -> Option<&'static str> {
        Some(Self::cache_key())
    }

    fn cache_key() -> &'static str {
        "gbuffer"
    }

    fn update(
        &self, _: &Device, _: &Queue,
        _: &mut PreparedBindGroup<Self::Data>,
    ) -> bool {
        // TODO: implement <GBuffer as AsBindGroup>::update
        logger::error!(from = "gbuffer", "GBuffer::update not yet implemented");
        false
    }

    fn unprepared_bind_group(
        &self, _: &Device, _: &BindGroupLayout,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        use OwnedBindingResource::*;
        
        Ok(UnpreparedBindGroup {
            data: self.clone(),
            bindings: vec![
                (0, TextureView(self.depth.view.clone())),
                (1, Sampler(self.depth.sampler.clone())),
                (2, TextureView(self.albedo.view.clone())),
                (3, Sampler(self.albedo.sampler.clone())),
                (4, TextureView(self.position.view.clone())),
                (5, Sampler(self.position.sampler.clone())),
                (6, TextureView(self.normal.view.clone())),
                (7, Sampler(self.normal.sampler.clone())),
            ],
        })
    }

    fn bind_group_layout_entries(_: &Device) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Depth,
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 5,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 6,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 7,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                count: None,
            },
        ]
    }
}