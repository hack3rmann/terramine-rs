use crate::prelude::*;

crate::define_atomic_id!(BindGroupLayoutId);

#[derive(Clone, Debug, Deref)]
pub struct BindGroupLayout {
    pub id: BindGroupLayoutId,
    #[deref]
    pub inner: Arc<wgpu::BindGroupLayout>,
}
assert_impl_all!(BindGroupLayout: Send, Sync);

impl PartialEq for BindGroupLayout {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<wgpu::BindGroupLayout> for BindGroupLayout {
    fn from(value: wgpu::BindGroupLayout) -> Self {
        BindGroupLayout {
            id: BindGroupLayoutId::new(),
            inner: Arc::new(value),
        }
    }
}