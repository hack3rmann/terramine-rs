use {
    crate::{
        prelude::*,
        graphics::camera_resource::Camera,
    },
    math_linear::{prelude::*, math::ray::space_3d::Line},
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
    pub fn new(cam: &Camera) -> Self {
        /* Far rectangle half size */
        let half_vertical_side = f32::tan(cam.fov.get_radians() / 2.0) * cam.far_plane_dist;
        let half_horizontal_side = half_vertical_side / cam.aspect_ratio;
        
        let front_far = cam.front * cam.far_plane_dist;

        /* Planes */
        let near	= Plane::new(cam.pos + cam.front * cam.near_plane_dist, cam.front);
        let far		= Plane::new(cam.pos + front_far, -cam.front);
        let right	= Plane::new(cam.pos, cam.up.cross(front_far + cam.right * half_horizontal_side));
        let left	= Plane::new(cam.pos, (front_far - cam.right * half_horizontal_side).cross(cam.up));
        let top		= Plane::new(cam.pos, cam.right.cross(front_far - cam.up * half_vertical_side));
        let bottom	= Plane::new(cam.pos, (front_far + cam.up * half_vertical_side).cross(cam.right));

        /* Lines */
        let courner_rays = [
            Line::from_2_points(cam.pos, cam.pos + (front_far + cam.right * half_horizontal_side + cam.up * half_vertical_side)),
            Line::from_2_points(cam.pos, cam.pos + (front_far - cam.right * half_horizontal_side + cam.up * half_vertical_side)),
            Line::from_2_points(cam.pos, cam.pos + (front_far + cam.right * half_horizontal_side - cam.up * half_vertical_side)),
            Line::from_2_points(cam.pos, cam.pos + (front_far - cam.right * half_horizontal_side - cam.up * half_vertical_side)),
        ];

        Self { near, far, left, right, top, bottom, courner_rays }
    }

    /// [AABB][Aabb]-frustum intersection check.
    pub fn is_aabb_in_frustum(&self, aabb: Aabb) -> bool {
        // If camera in AABB then intersection found.
        if aabb.contains_point(self.courner_rays[0].origin) {
            return true;
        }

        // If AABB centre is in frustum then intersection found.
        if self.contains(aabb.center()) {
            return true;
        }

        let aabb_vertices = aabb.as_vertex_array();

        let is_all_vertices_behind = aabb_vertices.iter().copied()
            .all(|vertex| self.near.signed_distance(vertex) < -f32::EPSILON);

        // If all vertices are behind the frustum there's no intersection.
        if is_all_vertices_behind {
            return true;
        }

        // If any AABB vertex is in frustum then intersection found.
        if aabb_vertices.iter().any(|&vertex| self.contains(vertex)) {
            return true;
        }

        // If any corner ray intersects AABB then intersection found.
        if self.courner_rays.iter().any(|ray| aabb.intersects_ray(ray)) {
            return true;
        }

        // All intersection tests failed.
        false
    }

    /// Checks if given vector is in frustum
    pub fn contains(&self, vec: vec3) -> bool {
        self.planes()
            .into_iter()
            .all(|plane| plane.is_in_positive_side(vec))
    }

    pub fn planes(&self) -> [Plane; 6] {
        [self.near, self.far, self.left, self.right, self.top, self.bottom]
    }

    /// Gives signed distance sum.
    #[allow(dead_code)]
    pub fn signed_distance_sum(&self, vec: vec3) -> f32 {
        self.planes()
            .into_iter()
            .map(|plane| plane.signed_distance(vec))
            .sum()
    }
}