use super::line::Line3d;
use glam::*;



/// Represents axis aligned bounding box
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    lo: Vec3,
    hi: Vec3,
}

impl Aabb {
    /// Constructs AABB from Float4 vectors.
    pub const fn new(lo: Vec3, hi: Vec3) -> Self {
        Self { lo, hi }
    }

    /// Represents AABB as corner vertex array.
    pub const fn as_vertex_array(self) -> [Vec3; 8] {
        [
            Vec3::new(self.lo.x, self.lo.y, self.lo.z),
            Vec3::new(self.lo.x, self.lo.y, self.hi.z),
            Vec3::new(self.lo.x, self.hi.y, self.lo.z),
            Vec3::new(self.lo.x, self.hi.y, self.hi.z),
            Vec3::new(self.hi.x, self.lo.y, self.lo.z),
            Vec3::new(self.hi.x, self.lo.y, self.hi.z),
            Vec3::new(self.hi.x, self.hi.y, self.lo.z),
            Vec3::new(self.hi.x, self.hi.y, self.hi.z),
        ]
    }

    /// Gives center point in AABB.
    pub fn center(self) -> Vec3 {
        Vec3::new(
            0.5 * (self.lo.x + self.hi.x),
            0.5 * (self.lo.y + self.hi.y),
            0.5 * (self.lo.z + self.hi.z),
        )
    }

    /// Tests ray intersection.
    /// 
    /// Source: <https://tavianator.com/2011/ray_box.html>
    pub fn intersects_ray(self, ray: &Line3d) -> bool {
        let mut t_max = f32::INFINITY;
        let mut t_min = f32::NEG_INFINITY;

        if ray.direction.x != 0.0 {
            let t1: f32 = (self.lo.x - ray.origin.x) / ray.direction.x;
            let t2: f32 = (self.hi.x - ray.origin.x) / ray.direction.x;

            t_min = f32::max(t_min, f32::min(t1, t2));
            t_max = f32::min(t_max, f32::max(t1, t2));
        }

        if ray.direction.y != 0.0 {
            let t1: f32 = (self.lo.y - ray.origin.y) / ray.direction.y;
            let t2: f32 = (self.hi.y - ray.origin.y) / ray.direction.y;

            t_min = f32::max(t_min, f32::min(t1, t2));
            t_max = f32::min(t_max, f32::max(t1, t2));
        }

        if ray.direction.z != 0.0 {
            let t1: f32 = (self.lo.z - ray.origin.z) / ray.direction.z;
            let t2: f32 = (self.hi.z - ray.origin.z) / ray.direction.z;

            t_min = f32::max(t_min, f32::min(t1, t2));
            t_max = f32::min(t_max, f32::max(t1, t2));
        }

        t_max >= t_min
    }

    /// Checks if AABB contains given vertex or the vertex is on boundary.
    pub fn contains_point(self, p: Vec3) -> bool {
        const EPS: f32 = f32::EPSILON;
        self.lo.x - EPS < p.x && p.x < self.hi.x + EPS
            && self.lo.y - EPS < p.y && p.y < self.hi.y + EPS
            && self.lo.z - EPS < p.z && p.z < self.hi.z + EPS
    }
}