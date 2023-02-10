use {
    crate::app::utils::{
        graphics::camera::Camera,
        //math::prelude::*,
    },
    math_linear::{prelude::*, math::ray::space_3d::Line},
};

/// Represents the camera frustum
#[derive(Clone)]
pub struct Frustum {
    pub near: Plane,
    pub far: Plane,
    pub left: Plane,
    pub right: Plane,
    pub top: Plane,
    pub bottom: Plane,

    pub courner_rays: [Line; 4]
}

impl Frustum {
    /// Creates frustum struct from camera data
    pub fn new(cam: &Camera) -> Frustum {
        /* Far rectangle half size */
        let half_vertical_side = (cam.fov.get_radians() / 2.0).tan() * cam.far_plane_dist;
        let half_horizontal_side = half_vertical_side / cam.aspect_ratio;
        
        let front_far = cam.front * cam.far_plane_dist;

        /* Planes */
        let near	= Plane::from_origin_and_normal(cam.pos + cam.front * cam.near_plane_dist, cam.front);
        let far		= Plane::from_origin_and_normal(cam.pos + front_far, -cam.front);
        let right	= Plane::from_origin_and_normal(cam.pos, cam.up.cross(front_far + cam.right * half_horizontal_side));
        let left	= Plane::from_origin_and_normal(cam.pos, (front_far - cam.right * half_horizontal_side).cross(cam.up));
        let top		= Plane::from_origin_and_normal(cam.pos, cam.right.cross(front_far - cam.up * half_vertical_side));
        let bottom	= Plane::from_origin_and_normal(cam.pos, (front_far + cam.up * half_vertical_side).cross(cam.right));

        /* Lines */
        let courner_rays = [
            Line::from_2_points(cam.pos, cam.pos + (front_far + cam.right * half_horizontal_side + cam.up * half_vertical_side)),
            Line::from_2_points(cam.pos, cam.pos + (front_far - cam.right * half_horizontal_side + cam.up * half_vertical_side)),
            Line::from_2_points(cam.pos, cam.pos + (front_far + cam.right * half_horizontal_side - cam.up * half_vertical_side)),
            Line::from_2_points(cam.pos, cam.pos + (front_far - cam.right * half_horizontal_side - cam.up * half_vertical_side)),
        ];

        Frustum { near, far, left, right, top, bottom, courner_rays }
    }

	/// Frustum check
	pub fn is_aabb_in_frustum(&self, aabb: AABB) -> bool {
		/* Frirst pass
		 * 1) Checks if camera position is in AABB
		 * 2) Checks if center of chunk is in frustum
		 * Very cheap operation */
		if aabb.is_containing(self.courner_rays[0].origin) { return true; }
		if self.is_in_frustum(aabb.center()) { return true; }

		/* Second pass
		 * Checks every vertex of AABB is behind the frustum
		 ? 8 times more expensive than previous */

		let vertex_set = aabb.as_vertex_array();

		let mut result = false;
		for vertex in vertex_set {
			if self.near.is_in_positive_side(vertex) {
				result = true;
				break;
			}
		}
		if !result { return result }

		/* Third pass
		 * Checks every vertex of AABB is in frustum
		 ? 6 times more expensive than previous */

		for vertex in vertex_set {
			if self.is_in_frustum(vertex) {
				return true;
			}
		}

		/* Fourth pass
		 * Checks if someone of 4 frustum corner rays intersects AABB
		 ? Kinda cheap operation */
		
		for ray in self.courner_rays {
			if aabb.is_intersected_by_ray(ray) {
				return true;
			}
		}

		/* All passed */
		return false;
	}

    /// Checks if given vector is in frustum
    pub fn is_in_frustum(&self, vec: vec3) -> bool {
        self.near	.is_in_positive_side(vec) &&
        self.far	.is_in_positive_side(vec) &&
        self.left	.is_in_positive_side(vec) &&
        self.right	.is_in_positive_side(vec) &&
        self.top	.is_in_positive_side(vec) &&
        self.bottom	.is_in_positive_side(vec)
    }

    /// Gives signed distance sum
    #[allow(dead_code)]
    pub fn signed_distance_sum(&self, vec: vec3) -> f32 {
        let mut sum = 0.0;
        sum += self.near	.signed_distance(vec);
        sum += self.far		.signed_distance(vec);
        sum += self.left	.signed_distance(vec);
        sum += self.right	.signed_distance(vec);
        sum += self.top		.signed_distance(vec);
        sum += self.bottom	.signed_distance(vec);

        return sum;
    }
}