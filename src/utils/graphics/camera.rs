/**
 * Camera handler.
 */

use directx_math::*;
use crate::utils::user_io::{InputManager, KeyCode};

/// Camera handler.
pub struct Camera {
	pub roll: f64,
	pub pitch: f64,
	pub yaw: f64,
	pub fov: f32,
	pub speed: f64,
	pub grabbes_cursor: bool,
	pub aspect_ratio: f32,

	pub pos: XMVector,
	pub up: XMVector,
	pub front: XMVector,
	pub right: XMVector,

	pub rotation: XMMatrix,
}

#[allow(dead_code)]
impl Camera {
	/// Creates camera.
	pub fn new() -> Self { Default::default() }

	/// This function updates camera vectors from rotatiion matrix.
	pub fn update_vectors(&mut self) {
		/* Transform basic vectors with rotation matrix */
		self.up.0    = XMVector4Transform(XMVectorSet(0.0, 1.0,  0.0, 1.0), self.rotation.0);
		self.front.0 = XMVector4Transform(XMVectorSet(0.0, 0.0, -1.0, 1.0), self.rotation.0);
		self.right.0 = XMVector4Transform(XMVectorSet(1.0, 0.0,  0.0, 1.0), self.rotation.0);
	}

	/// Stores rotation.
	pub fn set_rotation(&mut self, roll: f64, pitch: f64, yaw: f64) {
		self.roll = roll;
		self.pitch = pitch;
		self.yaw = yaw;

		self.rotation.0 = XMMatrixRotationRollPitchYaw(roll as f32, pitch as f32, yaw as f32);

		self.update_vectors();
	}

	/// Sets position.
	pub fn set_position(&mut self, x: f64, y: f64, z: f64) {
		self.pos.0 = XMVectorSet(x as f32, y as f32, z as f32, 1.0);
	}

	/// Sets rotation to [0.0, 0.0, 0.0].
	pub fn reset_rotation(&mut self) {
		self.set_rotation(0.0, 0.0, 0.0);
	}

	/// Moves camera towards its vectors.
	pub fn move_pos(&mut self, front: f64, up: f64, right: f64) {
		/* Front */
		self.pos += XMVector(
			XMVector3Normalize(
				XMVectorSet(
					XMVectorGetX(self.front.0),
					0.0,
					XMVectorGetZ(self.front.0),
					1.0
				)
			)
		) * front as f32;

		/* Up */
		self.pos += XMVector(XMVectorSet(0.0, up as f32, 0.0, 1.0));

		/* Right */
		self.pos += self.right * right as f32;
	}

	/// Returns view matrix.
	pub fn get_view(&self) -> [[f32; 4]; 4] {
		XMMatrix(XMMatrixLookAtLH(self.pos.0, (self.pos + self.front).0, self.up.0)).into()
	}

	/// Returns projection matrix with `aspect_ratio = height / width`
	pub fn get_proj(&self) -> [[f32; 4]; 4] {
		XMMatrix(XMMatrixPerspectiveLH(1.0, self.aspect_ratio, 0.5, 100.0)).into()
	}

	/// Returns X component of pos vector.
	pub fn get_x(&self) -> f32 {
		XMVectorGetX(self.pos.0)
	}

	/// Returns Y component of pos vector.
	pub fn get_y(&self) -> f32 {
		XMVectorGetY(self.pos.0)
	}

	/// Returns Z component of pos vector.
	pub fn get_z(&self) -> f32 {
		XMVectorGetZ(self.pos.0)
	}

	/// Rotates camera.
	pub fn rotate(&mut self, roll: f64, pitch: f64, yaw: f64) {
		self.roll += roll;
		self.pitch += pitch;
		self.yaw += yaw;

		let eps = 0.001;
		if self.roll > std::f64::consts::FRAC_PI_2 {
			self.roll = std::f64::consts::FRAC_PI_2 - eps;
		} else if self.roll < -std::f64::consts::FRAC_PI_2 {
			self.roll = -std::f64::consts::FRAC_PI_2 + eps;
		}

		self.set_rotation(self.roll, self.pitch, self.yaw);
	}

	/// Updates camera (key press checking, etc).
	pub fn update(&mut self, input: &mut InputManager, dt: f64) {
		if input.keyboard.is_pressed(KeyCode::W)		{ self.move_pos( dt * self.speed,  0.0,    0.0); }
		if input.keyboard.is_pressed(KeyCode::S)		{ self.move_pos(-dt * self.speed,  0.0,    0.0); }
		if input.keyboard.is_pressed(KeyCode::D)		{ self.move_pos( 0.0,    0.0,   -dt * self.speed); }
		if input.keyboard.is_pressed(KeyCode::A)		{ self.move_pos( 0.0,    0.0,    dt * self.speed); }
		if input.keyboard.is_pressed(KeyCode::LShift)	{ self.move_pos( 0.0,   -dt * self.speed,  0.0); }
		if input.keyboard.is_pressed(KeyCode::Space)	{ self.move_pos( 0.0,    dt * self.speed,  0.0); }
		if input.keyboard.just_pressed(KeyCode::P) {
			self.set_position(0.0, 0.0, 2.0);
			self.reset_rotation();
		}
		if self.grabbes_cursor {
			self.rotate(
				-input.mouse.dy * dt * 0.2,
				input.mouse.dx * dt * 0.2,
				0.0
			);
		}
	}
}

impl Default for Camera {
	fn default() -> Self {
		let mut cam = Camera {
			roll: 0.0,
			pitch: 0.0,
			yaw: 0.0,
			fov: 60.0,
			grabbes_cursor: false,
			speed: 10.0,
			aspect_ratio: 768.0 / 1024.0,
			pos: XMVector(XMVectorSet(0.0, 0.0, -3.0, 1.0)),
			up: XMVector(XMVectorSet(0.0, 1.0, 0.0, 1.0)),
			front: XMVector(XMVectorSet(0.0, 0.0, -1.0, 1.0)),
			right: XMVector(XMVectorSet(1.0, 0.0, 0.0, 1.0)),
			rotation: XMMatrix(XMMatrixRotationRollPitchYaw(0.0, 0.0, 0.0)),
		};
		cam.update_vectors();

		return cam;
	}
}