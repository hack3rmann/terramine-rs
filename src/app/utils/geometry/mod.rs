pub mod frustum;

use {
    crate::prelude::*,
    frustum::Frustum,
    math_linear::math::{ray::space_3d::Line, bounding_volumes::aabb::Aabb},
};



#[const_trait]
pub trait Intersects<With> {
    fn intersects(&self, with: &With) -> bool;
}
assert_obj_safe!(Intersects<i32>);

/// [`Intersects`] is commutative.
impl<T: ~const Intersects<S>, S> const Intersects<T> for S {
    default fn intersects(&self, with: &T) -> bool {
        with.intersects(self)
    }
}

impl Intersects<Frustum> for Aabb {
    fn intersects(&self, with: &Frustum) -> bool {
        with.intersects_aabb(self)
    }
}

impl const Intersects<Line> for Aabb {
    fn intersects(&self, with: &Line) -> bool {
        self.intersects_ray(with)
    }
}



#[const_trait]
pub trait Contains<T> {
    fn contains(&self, obj: &T) -> bool;
}
assert_obj_safe!(Contains<vec3>);

impl const Contains<vec3> for Frustum {
    fn contains(&self, obj: &vec3) -> bool {
        self.contains_point(*obj)
    }
}

impl const Contains<vec3> for Aabb {
    fn contains(&self, obj: &vec3) -> bool {
        self.contains_point(*obj)
    }
}