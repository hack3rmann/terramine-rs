pub mod angle;
pub mod plane;
pub mod spheres;

use crate::prelude::*;

pub trait Intersect<Obj> {
    fn intersect(&self, obj: &Obj) -> Option<Vec<vec2>>;
}

// TODO: resolve template trait impl conflict.

pub enum Containment {
    Contains,
    OnBoundary,
    Outside,
}
