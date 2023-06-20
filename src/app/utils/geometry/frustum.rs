use {
    crate::{
        prelude::*,
        camera::CameraComponent,
        transform::Transform,
        geometry::{Intersects, Contains},
    },
    math_linear::math::ray::space_3d::Line,
};



/// Represents the camera frustum
#[derive(Clone, Debug)]
pub struct Frustum {
    pub near: Plane,
    pub far: Plane,
    pub left: Plane,
    pub right: Plane,
    pub top: Plane,
    pub bottom: Plane,

    pub courner_rays: [Line; 4]
}
assert_impl_all!(Frustum: Send, Sync);

impl Frustum {
    /// Creates frustum struct from camera data
    pub fn new(cam: &CameraComponent, transform: &Transform) -> Self {
        let front_dir = transform.rotation.front();
        let up_dir = transform.rotation.up();
        let right_dir = transform.rotation.right();
        let pos = transform.translation.position;

        // Far rectangle half size.
        let half_vertical_side = f32::tan(cam.fov.get_radians() / 2.0) * cam.far_plane;
        let half_horizontal_side = half_vertical_side / cam.aspect_ratio;
        
        let front_far = front_dir * cam.far_plane;

        // Planes.
        let near_plane   = Plane::new(pos + front_dir * cam.near_plane, front_dir);
        let far_plane    = Plane::new(pos + front_far, -front_dir);
        let right_plane  = Plane::new(pos, up_dir.cross(front_far + right_dir * half_horizontal_side));
        let left_plane   = Plane::new(pos, (front_far - right_dir * half_horizontal_side).cross(up_dir));
        let top_plane    = Plane::new(pos, right_dir.cross(front_far - up_dir * half_vertical_side));
        let bottom_plane = Plane::new(pos, (front_far + up_dir * half_vertical_side).cross(right_dir));

        let courner_rays = [
            Line::from_2_points(pos, pos + (front_far + right_dir * half_horizontal_side + up_dir * half_vertical_side)),
            Line::from_2_points(pos, pos + (front_far - right_dir * half_horizontal_side + up_dir * half_vertical_side)),
            Line::from_2_points(pos, pos + (front_far + right_dir * half_horizontal_side - up_dir * half_vertical_side)),
            Line::from_2_points(pos, pos + (front_far - right_dir * half_horizontal_side - up_dir * half_vertical_side)),
        ];

        Self {
            near: near_plane,
            far: far_plane,
            left: left_plane,
            right: right_plane,
            top: top_plane,
            bottom: bottom_plane,
            courner_rays,
        }
    }

    /// [AABB][Aabb]-[frustum][Frustum] intersection check.
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        // If camera in AABB then intersection found.
        ensure_or!(!aabb.contains(&self.courner_rays[0].origin), return true);

        // If AABB centre is in frustum then intersection found.
        ensure_or!(!self.contains(&aabb.center()), return true);

        let aabb_vertices = aabb.as_vertex_array();

        let is_all_vertices_behind = aabb_vertices.iter().copied()
            .all(|vertex| self.near.signed_distance(vertex) <= -f32::EPSILON);

        // If all vertices are behind the frustum there's no intersection.
        ensure_or!(!is_all_vertices_behind, return false);

        // If any AABB vertex is in frustum then intersection found.
        ensure_or!(!aabb_vertices.iter().any(|vertex| self.contains(vertex)), return true);

        // If any corner ray intersects AABB then intersection found.
        ensure_or!(!self.courner_rays.iter().any(|ray| aabb.intersects(ray)), return true);

        // All intersection tests failed.
        false
    }

    /// Checks if given point is in frustum or on it's boundary.
    pub const fn contains_point(&self, vec: vec3) -> bool {
        self.near.signed_distance(vec) > -f32::EPSILON
            && self.far.signed_distance(vec) > -f32::EPSILON
            && self.left.signed_distance(vec) > -f32::EPSILON
            && self.right.signed_distance(vec) > -f32::EPSILON
            && self.top.signed_distance(vec) > -f32::EPSILON
            && self.bottom.signed_distance(vec) > -f32::EPSILON
    }

    pub const fn planes(&self) -> [Plane; 6] {
        [self.near, self.far, self.left, self.right, self.top, self.bottom]
    }
}