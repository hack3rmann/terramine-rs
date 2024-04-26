use crate::{graphics::*, prelude::*};



#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct RenderGraph {
    nodes: Vec<RenderNode>,
}
assert_impl_all!(RenderGraph: Send, Sync, Component);

impl RenderGraph {
    pub const fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn run(
        &self, world: &World, encoder: &mut CommandEncoder,
        targets: &[TextureView], depth: Option<&TextureView>,
    ) {
        for node in self.nodes.iter() {
            node.run(world, encoder, targets, depth)
                .log_error(
                    "render-graph",
                    format!("failed to run \"{}\" graph node", node.name)
                );
        }
    }

    pub fn add(&mut self, node: RenderNode) -> &mut Self {
        self.nodes.push(node);
        self
    }
}



pub type RunRenderNode = fn(
    &World, &mut CommandEncoder, &[TextureView], Option<&TextureView>,
) -> AnyResult<()>;



#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Debug, Hash, Eq)]
pub struct RenderNode {
    pub run: RunRenderNode,
    pub name: StaticStr,
}
assert_impl_all!(RenderNode: Send, Sync);

impl RenderNode {
    pub fn new(run: RunRenderNode, name: impl Into<StaticStr>) -> Self {
        Self { run, name: name.into() }
    }

    pub fn run(
        &self, world: &World, encoder: &mut CommandEncoder,
        targets: &[TextureView], depth: Option<&TextureView>,
    ) -> AnyResult<()> {
        (self.run)(world, encoder, targets, depth)
    }
}

impl PartialEq for RenderNode {
    fn eq(&self, other: &Self) -> bool {
        self.run as usize == other.run as usize
    }
}



pub trait AsRenderNodeInput {
    fn collect_bind_groups(&self, cache: &BindsCache) -> Vec<BindGroup>;
    fn as_render_node_input(&self, cache: &BindsCache) -> RenderNodeInput {
        RenderNodeInput { binds: self.collect_bind_groups(cache) }
    }
}
assert_obj_safe!(AsRenderNodeInput);



#[derive(Clone, Debug)]
pub struct RenderNodeInput {
    /// [`BindGroup`]s in binding order
    binds: Vec<BindGroup>,
}
assert_impl_all!(RenderNodeInput: Send, Sync);

impl RenderNodeInput {
    pub fn new(cache: &BindsCache, src: &impl AsRenderNodeInput) -> Self {
        src.as_render_node_input(cache)
    }

    pub fn bind<'s>(&'s self, pass: &mut RenderPass<'s>) {
        for (i, bind) in self.binds.iter().enumerate() {
            bind.bind(pass, i as u32);
        }
    }
}



#[derive(Clone, Debug)]
pub struct RenderNodeOutput {
    pub color_targets: Vec<TextureView>,
    pub depth_target: Option<TextureView>,
}

impl RenderNodeOutput {
    pub fn color_targets(&self) -> &[TextureView] {
        &self.color_targets
    }

    pub fn depth_target(&self) -> Option<&TextureView> {
        self.depth_target.as_ref()
    }
}



#[cfg(test)]
mod tests {
    use { super::*, crate::terrain::chunk::array::render::* };

    struct ChunkRenderNode {
        pub label: StaticStr,
        pub textures: ChunkArrayTextures,
        pub run: fn(&World, &mut CommandEncoder, RenderNodeInput, RenderNodeOutput),
        pub output: RenderNodeOutput,
    }

    impl AsRenderNodeInput for ChunkRenderNode {
        fn collect_bind_groups(&self, cache: &BindsCache) -> Vec<BindGroup> {
            vec![
                cache.get::<CommonUniform>().unwrap().bind_group.clone(),
                cache.get::<CameraUniform>().unwrap().bind_group.clone(),
                cache.get::<ChunkArrayTextures>().unwrap().bind_group.clone(),
            ]
        }
    }

    impl ChunkRenderNode {
        pub fn run(
            &self, world: &World, encoder: &mut CommandEncoder,
            input: RenderNodeInput,
        ) -> AnyResult<()> {
            let mut query = world.query::<(&GpuMesh, &ChunkArrayPipeline)>();

            let pipeline_cache = world.resource::<&PipelineCache>()?;

            let mut pass = RenderPass::new(
                "chunk_array_render_pass",
                encoder,
                self.output.color_targets(),
                self.output.depth_target(),
            );

            for (_entity, (mesh, pipeline_bound)) in query.iter() {
                let pipeline = pipeline_cache.get(pipeline_bound)
                    .context("failed to get find pipeline in cache")?;

                input.bind(&mut pass);

                mesh.render(pipeline, &mut pass);
            }

            Ok(())
        }
    }
}