use {
    crate::{
        prelude::*,
        graphics::{TextureView, CommandEncoder},
    },
    std::fmt::Debug,
};



#[derive(Debug, Deref)]
pub struct ClearPass<'s>(pub wgpu::RenderPass<'s>);
assert_impl_all!(ClearPass: Send, Sync);

impl<'s> ClearPass<'s> {
    pub fn new(
        encoder: &'s mut wgpu::CommandEncoder,
        target_views: impl IntoIterator<Item = &'s TextureView>,
        target_depth: Option<&'s TextureView>,
        clear_color: wgpu::Color,
        clear_depth: Option<f32>,
    ) -> Self {
        use wgpu::{RenderPassColorAttachment, Operations, LoadOp, RenderPassDescriptor};

        let color_attachments: Vec<_> = target_views.into_iter()
            .map(|view| Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            }))
            .collect();

        let render_pass = encoder.begin_render_pass(
            &RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: target_depth.map(|view|
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_depth.unwrap_or(1.0)),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                ),
                // TODO: configurate this
                timestamp_writes: None,
                // TODO: configure this
                occlusion_query_set: None,
            },
        );

        Self(render_pass)
    }

    /// Makes new [`ClearPass`] then drops it.
    pub fn clear(
        encoder: &'s mut CommandEncoder,
        target_views: impl IntoIterator<Item = &'s TextureView>,
        target_depth: Option<&'s TextureView>,
        clear_color: wgpu::Color,
        clear_depth: Option<f32>,
    ) {
        let _pass = Self::new(
            encoder, target_views, target_depth, clear_color, clear_depth,
        );
    }
}



#[derive(Debug, Deref, From, Into)]
pub struct RenderPass<'s>(pub wgpu::RenderPass<'s>);
assert_impl_all!(ClearPass: Send, Sync);

impl<'s> RenderPass<'s> {
    pub fn new(
        label: &str,
        encoder: &'s mut CommandEncoder,
        target_views: impl IntoIterator<Item = &'s TextureView>,
        depth_view: Option<&'s TextureView>,
    ) -> Self {
        use wgpu::{RenderPassColorAttachment, Operations, LoadOp, StoreOp, RenderPassDescriptor};

        let color_attachments: Vec<_> = target_views.into_iter()
            .map(|view| Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            }))
            .collect();

        let render_pass = encoder.begin_render_pass(
            &RenderPassDescriptor {
                label: Some(label),
                color_attachments: &color_attachments,
                depth_stencil_attachment: depth_view.map(|view|
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                ),
                // TODO: configure this
                timestamp_writes: None,
                // TODO: configure this
                occlusion_query_set: None,
            },
        );

        Self::from(render_pass)
    }
}