use crate::app::utils::math::plane::Plane;
use crate::app::utils::graphics::camera::Camera;
use crate::app::utils::math::vector::Float4;

/// Represents the camera frustum
pub struct Frustum {
	pub near: Plane,
	pub far: Plane,
	pub left: Plane,
	pub right: Plane,
	pub top: Plane,
	pub bottom: Plane,
}

impl Frustum {
	/// Creates frustum struct from camera data
	pub fn new(cam: &Camera) -> Frustum {
		let half_vertical_side = (cam.fov.get_radians() / 2.0).tan() * cam.far;
		let half_horizontal_side = half_vertical_side / cam.aspect_ratio;
		
		let front_far = cam.front * cam.far;

		let near	= Plane::from_origin_and_normal(cam.pos + cam.front * cam.near, cam.front);
		let far		= Plane::from_origin_and_normal(cam.pos + front_far, -cam.front);
		let right	= Plane::from_origin_and_normal(cam.pos, cam.up.cross(front_far + cam.right * half_horizontal_side));
		let left	= Plane::from_origin_and_normal(cam.pos, (front_far - cam.right * half_horizontal_side).cross(cam.up));
		let top		= Plane::from_origin_and_normal(cam.pos, cam.right.cross(front_far - cam.up * half_vertical_side));
		let bottom	= Plane::from_origin_and_normal(cam.pos, (front_far + cam.up * half_vertical_side).cross(cam.right));

		Frustum { near, far, left, right, top, bottom }
	}

	/// Checks if given vector is in frustum
	pub fn is_in_frustum(&self, vec: Float4) -> bool {
	//	self.near	.is_in_positive_side(vec) &&
	//	self.far	.is_in_positive_side(vec) &&
		self.left	.is_in_positive_side(vec) &&
		self.right	.is_in_positive_side(vec) &&
		self.top	.is_in_positive_side(vec) &&
		self.bottom	.is_in_positive_side(vec)
	}
}