/**
 * Camera handler.
 */

pub mod frustum;

use crate::app::utils::user_io::{InputManager, KeyCode};
use crate::app::utils::math::prelude::*;
use frustum::Frustum;

/// Camera handler.
pub struct Camera {
	/* Screen needs */
	pub fov: Angle,
	pub aspect_ratio: f32,
	pub near: f32,
	pub far: f32,

	/* Additional control */
	pub speed_factor: f64,
	pub grabbes_cursor: bool,

	/* Position */
	pub pos: Float4,
	pub speed: Float4,
	pub speed_falloff: f32,

	/* Rotation */
	pub rotation: Matrix4,
	pub roll:	f64,
	pub pitch:	f64,
	pub yaw:	f64,
	pub up:		Float4,
	pub front:	Float4,
	pub right:	Float4,

	/* Frustum */
	frustum: Option<Frustum>,
}

#[allow(dead_code)]
impl Camera {
	/// Creates camera.
	pub fn new() -> Self { Default::default() }

	/// Gives camera positioned to given coordinates
	pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
		self.set_position(x, y, z);
		return self;
	}

	/// Sets position.
	pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
		self.pos = Float4::xyz1(x, y, z);
	}

	/// Gives camera rotated to given angles
	pub fn with_rotation(mut self, roll: f64, pitch: f64, yaw: f64) -> Self {
		self.set_rotation(roll, pitch, yaw);
		return self;
	}

	/// Stores rotation.
	pub fn set_rotation(&mut self, roll: f64, pitch: f64, yaw: f64) {
		self.roll = roll;
		self.pitch = pitch;
		self.yaw = yaw;

		self.rotation = Matrix4::rotation_rpy(roll as f32, pitch as f32, yaw as f32);

		self.update_vectors();
	}

	/// Sets rotation to [0.0, 0.0, 0.0].
	pub fn reset_rotation(&mut self) {
		self.set_rotation(0.0, 0.0, 0.0);
	}

	/// Moves camera towards its vectors.
	pub fn move_relative(&mut self, front: f64, up: f64, right: f64) {
		/* Front */
		self.pos += Float4::xyz1(self.front.x(), 0.0, self.front.z()).normalyze() * front as f32;

		/* Up */
		self.pos += Float4::xyz1(0.0, up as f32, 0.0);

		/* Right */
		self.pos += self.right * right as f32;
	}

	/// Moves camera towards coordinates.
	pub fn move_absolute(&mut self, ds: Float4) {
		self.pos += ds
	}

	/// Rotates camera.
	pub fn rotate(&mut self, roll: f64, pitch: f64, yaw: f64) {
		self.roll += roll;
		self.pitch += pitch;
		self.yaw += yaw;

		/* Vertical camera look boundaries */
		let eps = 0.001;
		if self.pitch > std::f64::consts::FRAC_PI_2 {
			self.pitch = std::f64::consts::FRAC_PI_2 - eps;
		} else if self.pitch < -std::f64::consts::FRAC_PI_2 {
			self.pitch = -std::f64::consts::FRAC_PI_2 + eps;
		}

		self.set_rotation(self.roll, self.pitch, self.yaw);
	}

	/// This function updates camera vectors from rotatiion matrix.
	pub fn update_vectors(&mut self) {
		/* Transform basic vectors with rotation matrix */
		self.up =    self.rotation.clone() * Float4::xyz1(0.0,  1.0,  0.0);
		self.front = self.rotation.clone() * Float4::xyz1(0.0,  0.0, -1.0);
		self.right = self.rotation.clone() * Float4::xyz1(1.0,  0.0,  0.0);

		/* Frustum update */
		self.frustum = Some(Frustum::new(self));
	}

	/// Updates camera (key press checking, etc).
	pub fn update(&mut self, input: &mut InputManager, dt: f64) {
		/* Camera move vector */
		let mut new_speed = Float4::all(0.0);

		/* Movement controls */
		if input.keyboard.is_pressed(KeyCode::W)		{ new_speed += Float4::xyz0(self.front.x(), 0.0, self.front.z()).normalyze() }
		if input.keyboard.is_pressed(KeyCode::S)		{ new_speed -= Float4::xyz0(self.front.x(), 0.0, self.front.z()).normalyze() }
		if input.keyboard.is_pressed(KeyCode::A)		{ new_speed += self.right.normalyze() }
		if input.keyboard.is_pressed(KeyCode::D)		{ new_speed -= self.right.normalyze() }
		if input.keyboard.is_pressed(KeyCode::Space)	{ new_speed += Float4::xyz0(0.0, 1.0, 0.0) }
		if input.keyboard.is_pressed(KeyCode::LShift)	{ new_speed -= Float4::xyz0(0.0, 1.0, 0.0) }

		/* Calculate new speed */
		new_speed = new_speed.normalyze() * self.speed_factor as f32;

		/* Normalyzing direction vector */
		self.speed = if new_speed != Float4::all(0.0) {
			self.speed / 2.0 + new_speed / 2.0
		} else {
			if self.speed.abs() > 0.1 {
				self.speed * self.speed_falloff
			} else {
				Float4::all(0.0)
			}
		};

		/* Move camera with move vector */
		self.move_absolute(self.speed * dt as f32);

		/* Reset */
		if input.keyboard.just_pressed(KeyCode::P) {
			self.set_position(0.0, 0.0, 2.0);
			self.reset_rotation();
		}

		/* Cursor borrow */
		if self.grabbes_cursor {
			self.rotate(
				 0.0,
				-input.mouse.dy * dt * 0.2,
				 input.mouse.dx * dt * 0.2,
			);
		}
	}

	/// Returns view matrix.
	pub fn get_view(&self) -> [[f32; 4]; 4] {
		Matrix4::look_at_lh(self.pos, self.pos + self.front, self.up).as_2d_array()
	}

	/// Returns projection matrix with `aspect_ratio = height / width`
	pub fn get_proj(&self) -> [[f32; 4]; 4] {
		Matrix4::perspective_fov_lh(self.fov.get_radians(), self.aspect_ratio, self.near, self.far).as_2d_array()
	}

	/// Checks if position is in camera frustum
	pub fn is_pos_in_view(&self, pos: Float4) -> bool {
		self.get_frustum().is_in_frustum(pos)
	}

	/// Checks if AABB is in camera frustum
	pub fn is_aabb_in_view(&self, aabb: AABB) -> bool {
		aabb.is_in_frustum(&self.get_frustum())
	}

	/// Gives frustum from camera
	pub fn get_frustum(&self) -> Frustum {
		if self.frustum.is_none() {
			Frustum::new(self)
		} else {
			self.frustum.as_ref().unwrap().clone()
		}
	}

	/// Returns X component of pos vector.
	pub fn get_x(&self) -> f32 { self.pos.x() }

	/// Returns Y component of pos vector.
	pub fn get_y(&self) -> f32 { self.pos.y() }

	/// Returns Z component of pos vector.
	pub fn get_z(&self) -> f32 { self.pos.z() }
}

impl Default for Camera {
	fn default() -> Self {
		let mut cam = Camera {
			roll: 0.0,
			pitch: 0.0,
			yaw: 0.0,
			fov: Angle::from_degrees(60.0),
			near: 0.5,
			far: 1000.0,
			grabbes_cursor: false,
			speed_factor: 10.0,
			speed_falloff: 0.88,
			aspect_ratio: 768.0 / 1024.0,
			pos: Float4::xyz1(0.0, 0.0, -3.0),
			speed: Float4::all(0.0),
			up: Float4::xyz1(0.0, 1.0, 0.0),
			front: Float4::xyz1(0.0, 0.0, -1.0),
			right: Float4::xyz1(1.0, 0.0, 0.0),
			rotation: Default::default(),
			frustum: None,
		};
		cam.update_vectors();

		return cam;
	}
}