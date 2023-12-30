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
    ) -> AnyResult<()> {
        for &node in self.nodes.iter() {
            node.run(world, encoder, targets, depth)?;
        }

        Ok(())
    }

    pub fn add(&mut self, node: impl Into<RenderNode>) -> &mut Self {
        self.nodes.push(node.into());
        self
    }
}



pub type RunRenderNode = fn(
    &World, &mut CommandEncoder, &[TextureView], Option<&TextureView>,
) -> AnyResult<()>;



#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Copy, Debug, Hash, Eq)]
pub struct RenderNode {
    pub run: RunRenderNode,
}
assert_impl_all!(RenderNode: Send, Sync);

impl RenderNode {
    pub fn run(
        self, world: &World, encoder: &mut CommandEncoder,
        targets: &[TextureView], depth: Option<&TextureView>,
    ) -> AnyResult<()> {
        (self.run)(world, encoder, targets, depth)
    }
}

impl From<RunRenderNode> for RenderNode {
    fn from(value: RunRenderNode) -> Self {
        Self { run: value }
    }
}

impl PartialEq for RenderNode {
    fn eq(&self, other: &Self) -> bool {
        self.run as usize == other.run as usize
    }
}