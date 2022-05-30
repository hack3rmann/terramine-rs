/**
 * Camera handler.
 */

use directx_math::*;

/// Camera handler.
pub struct Camera {
	pub roll: f64,
	pub pitch: f64,
	pub yaw: f64,
	pub fov: f32,

	/* Terrible code hightliting. It's type, not an object, stupid! */
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
		self.pos += self.front * front as f32;
		self.pos += self.up * up as f32;
		self.pos += self.right * right as f32;
	}

	/// Returns view matrix.
	pub fn get_view(&self) -> [[f32; 4]; 4] {
		XMMatrix(XMMatrixLookAtLH(self.pos.0, (self.pos + self.front).0, self.up.0)).into()
	}

	/// Returns projection matrix.
	pub fn get_proj(&self) -> [[f32; 4]; 4] {
		XMMatrix(XMMatrixPerspectiveLH(1.0, 768.0 / 1024.0, 0.5, 100.0)).into()
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

	pub fn rotate(&mut self, roll: f64, pitch: f64, yaw: f64) {
		self.roll += roll;
		self.pitch += pitch;
		self.yaw += yaw;
		self.set_rotation(self.roll, self.pitch, self.yaw);
	}
}

impl Default for Camera {
	fn default() -> Self {
		let mut cam = Camera {
			roll: 0.0,
			pitch: 0.0,
			yaw: 0.0,
			fov: 60.0,
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