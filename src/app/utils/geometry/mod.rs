pub mod frustum;
pub mod line;
pub mod plane;
pub mod aabb;

pub use line::Line3d;
pub use plane::Plane;
pub use aabb::Aabb;
pub use frustum::Frustum;
use static_assertions::assert_obj_safe;
use glam::*;



pub trait Intersects<T> {
    fn intersects(&self, with: &T) -> bool;
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

impl Intersects<Line3d> for Aabb {
    fn intersects(&self, with: &Line3d) -> bool {
        self.intersects_ray(with)
    }
}

impl Intersects<Aabb> for Line3d {
    fn intersects(&self, with: &Aabb) -> bool {
        with.intersects(self)
    }
}



pub trait Contains<T> {
    fn contains(&self, obj: &T) -> bool;
}
assert_obj_safe!(Contains<Vec3>);

impl Contains<Vec3> for Frustum {
    /// # Note
    /// 
    /// It's `const fn`.
    fn contains(&self, obj: &Vec3) -> bool {
        self.contains_point(*obj)
    }
}

impl Contains<Vec3> for Aabb {
    fn contains(&self, obj: &Vec3) -> bool {
        self.contains_point(*obj)
    }
}