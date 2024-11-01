use {
    crate::{prelude::*, graphics::{Device, BufferInitDescriptor}},
    std::ops::{Bound, RangeBounds},
};



pub use wgpu::BufferUsages;



macros::define_atomic_id!(BufferId);

#[derive(Clone, Debug)]
pub struct Buffer {
    pub inner: Arc<wgpu::Buffer>,
    pub id: BufferId,
}
assert_impl_all!(Buffer: Send, Sync);

impl Buffer {
    pub fn new(device: &Device, desc: &BufferInitDescriptor) -> Self {
        use crate::graphics::DeviceExt;

        let buffer = device.create_buffer_init(desc);
        
        Self::from(buffer)
    }

    pub fn slice(&self, bounds: impl RangeBounds<wgpu::BufferAddress>) -> BufferSlice {
        BufferSlice {
            id: self.id,
            // Need to compute and store this manually because wgpu doesn't export offset on wgpu::BufferSlice
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

impl Deref for Buffer {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}



#[derive(Clone, Debug, Deref)]
pub struct BufferSlice<'s> {
    #[deref]
    pub inner: wgpu::BufferSlice<'s>,
    pub offset: wgpu::BufferAddress,
    pub id: BufferId,
}
assert_impl_all!(BufferSlice: Send, Sync);
