pub mod frustum;

use {
    crate::prelude::*,
    frustum::Frustum,
    math_linear::math::{ray::space_3d::Line, bounding_volumes::aabb::Aabb},
};



pub trait Intersects<With> {
    fn intersects(&self, with: &With) -> bool;
}
assert_obj_safe!(Intersects<i32>);

impl Intersects<Frustum> for Aabb {
    fn intersects(&self, with: &Frustum) -> bool {
        with.intersects_aabb(self)
    }
}

impl Intersects<Aabb> for Frustum {
    fn intersects(&self, with: &Aabb) -> bool {
        with.intersects(self)
    }
}

impl Intersects<Line> for Aabb {
    fn intersects(&self, with: &Line) -> bool {
        self.intersects_ray(with)
    }
}

impl Intersects<Aabb> for Line {
    fn intersects(&self, with: &Aabb) -> bool {
        with.intersects(self)
    }
}



pub trait Contains<T> {
    fn contains(&self, obj: &T) -> bool;
}
assert_obj_safe!(Contains<vec3>);

impl Contains<vec3> for Frustum {
    /// # Note
    /// 
    /// It's `const fn`.
    fn contains(&self, obj: &vec3) -> bool {
        self.contains_point(*obj)
    }
}

impl Contains<vec3> for Aabb {
    /// # Note
    /// 
    /// It's `const fn`.
    fn contains(&self, obj: &vec3) -> bool {
        self.contains_point(*obj)
    }
}