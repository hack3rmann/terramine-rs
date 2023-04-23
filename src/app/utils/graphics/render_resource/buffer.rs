use {
    crate::prelude::*,
    std::ops::{Bound, RangeBounds},
};

crate::define_atomic_id!(BufferId);

#[derive(Clone, Debug, Deref)]
pub struct Buffer {
    pub id: BufferId,
    #[deref]
    pub inner: Arc<wgpu::Buffer>,
}
assert_impl_all!(Buffer: Send, Sync);

impl Buffer {
    pub fn slice(&self, bounds: impl RangeBounds<wgpu::BufferAddress>) -> BufferSlice {
        BufferSlice {
            id: self.id,
            // need to compute and store this manually because wgpu doesn't export offset on wgpu::BufferSlice
            offset: match bounds.start_bound() {
                Bound::Included(&bound) => bound,
                Bound::Excluded(&bound) => bound + 1,
                Bound::Unbounded => 0,
            },
            inner: self.inner.slice(bounds),
        }
    }
}

impl From<wgpu::Buffer> for Buffer {
    fn from(value: wgpu::Buffer) -> Self {
        Self {
            id: BufferId::new(),
            inner: Arc::new(value),
        }
    }
}

#[derive(Clone, Debug, Deref)]
pub struct BufferSlice<'s> {
    pub id: BufferId,
    pub offset: wgpu::BufferAddress,
    #[deref]
    pub inner: wgpu::BufferSlice<'s>,
}
assert_impl_all!(BufferSlice: Send, Sync);
