use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, SmartDefault)]
pub struct Cube {
    #[default(vec3::ZERO)]
    pub centre_pos: vec3,

    #[default(vec3::ONE)]
    pub sizes: vec3,
}
static_assertions::assert_impl_all!(Cube: Send, Sync);
