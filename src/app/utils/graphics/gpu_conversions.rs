use crate::prelude::*;

pub trait ToGpu {
    type Descriptor;
    type GpuType;
    type Error;

    fn to_gpu(&self, desc: Self::Descriptor) -> Result<Self::GpuType, Self::Error>;
}
assert_obj_safe!(ToGpu<Descriptor = (), GpuType = (), Error = ()>);