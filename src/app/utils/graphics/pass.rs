use {
    crate::prelude::*,
    std::fmt::Debug,
};



pub trait Renderable: type_uuid::TypeUuidDynamic + Debug {
    fn render<'s, 'rp>(
        &'s self, render_pass: &mut wgpu::RenderPass<'rp>,
    ) -> Result<(), Box<dyn std::error::Error>> where 's: 'rp;
}
assert_obj_safe!(Renderable);



#[derive(Debug, Deref)]
pub struct ClearPass<'s>(pub wgpu::RenderPass<'s>);
assert_impl_all!(ClearPass: Send, Sync);

impl<'s> ClearPass<'s> {
    pub fn new(encoder: &'s mut wgpu::CommandEncoder, target_views: impl IntoIterator<Item = &'s wgpu::TextureView>) -> Self {
        use wgpu::{RenderPassColorAttachment, Operations, LoadOp, RenderPassDescriptor};

        let color_attachments: Vec<_> = target_views.into_iter()
            .map(|view| Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(cfg::shader::CLEAR_COLOR),
                    store: true,
                },
            }))
            .collect();

        let render_pass = encoder.begin_render_pass(
            &RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
            },
        );

        Self(render_pass)
    }
}



#[derive(Debug, Deref)]
pub struct RenderPass<'s>(pub wgpu::RenderPass<'s>);
assert_impl_all!(ClearPass: Send, Sync);

impl<'s> RenderPass<'s> {
    pub fn new(
        encoder: &'s mut wgpu::CommandEncoder,
        label: &str,
        target_views: impl IntoIterator<Item = &'s wgpu::TextureView>
    ) -> Self {
        use wgpu::{RenderPassColorAttachment, Operations, LoadOp, RenderPassDescriptor};

        let color_attachments: Vec<_> = target_views.into_iter()
            .map(|view| Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            }))
            .collect();

        let render_pass = encoder.begin_render_pass(
            &RenderPassDescriptor {
                label: Some(label),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
            },
        );

        Self(render_pass)
    }
}