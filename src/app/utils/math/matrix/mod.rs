mod traits;
use directx_math::*;
use super::vector::Float4;
use traits::{
	Inverse
};

#[allow(dead_code)]
pub type Matrix4 = Matrix4x4;

#[derive(Clone)]
pub struct Matrix4x4 {
	pub mat: XMMATRIX
}

#[allow(dead_code)]
impl Matrix4x4 {
	/// Constructs matrix from 16 floats.
	pub fn from_f32(
		a11: f32, a12: f32, a13: f32, a14: f32,
		a21: f32, a22: f32, a23: f32, a24: f32,
		a31: f32, a32: f32, a33: f32, a34: f32,
		a41: f32, a42: f32, a43: f32, a44: f32
	) -> Self {
		Matrix4x4 {
			mat: XMMatrixSet(
				a11, a12, a13, a14,
				a21, a22, a23, a24,
				a31, a32, a33, a34,
				a41, a42, a43, a44
			)
		}
	}

	/// Constructs matrix from 1d float array.
	pub fn from_1d_array(mat: [f32; 16]) -> Self {
		Matrix4x4 {
			mat: XMMatrixSet(
				mat[0],  mat[1],  mat[2],  mat[3],
				mat[4],  mat[5],  mat[6],  mat[7],
				mat[8],  mat[9],  mat[10], mat[11],
				mat[12], mat[13], mat[14], mat[15]
			)
		}
	}

	/// Constructs matrix from 2d float array.
	pub fn from_2d_array(mat: [[f32; 4]; 4]) -> Self {
		Matrix4x4 {
			mat: XMMatrixSet(
				mat[0][0], mat[0][1], mat[0][2], mat[0][3],
				mat[1][0], mat[1][1], mat[1][2], mat[1][3],
				mat[2][0], mat[2][1], mat[2][2], mat[2][3],
				mat[3][0], mat[3][1], mat[3][2], mat[3][3]
			)
		}
	}

	/// Gives 2D array representation of matrix.
	pub fn as_2d_array(self) -> [[f32; 4]; 4] {
		XMMatrix(self.mat).into()
	}

	/// Sets all elements to given float.
	pub fn all(all: f32) -> Self {
		Self::from_1d_array([all; 16])
	}

	/// Sets main diagonal to given float.
	pub fn set_diagonal(a: f32) -> Self {
		Self::from_f32(
			a,   0.0, 0.0, 0.0,
			0.0, a,   0.0, 0.0,
			0.0, 0.0, a,   0.0,
			0.0, 0.0, 0.0, a
		)
	}

	/// Sets main diagonal to given float except last is 1.0
	pub fn set_diagonal_last_unit(a: f32) -> Self {
		Self::from_f32(
			a,   0.0, 0.0, 0.0,
			0.0, a,   0.0, 0.0,
			0.0, 0.0, a,   0.0,
			0.0, 0.0, 0.0, 1.0
		)
	}

	/// Gives utit matrix.
	pub fn unit() -> Self {
		Self::set_diagonal(1.0)
	}

	/// Gives view matrix for left-handed coordinate system
	pub fn look_at_lh(eye_pos: Float4, focus_pos: Float4, up_dir: Float4) -> Self {
		Matrix4x4 {
			mat: XMMatrixLookAtLH(eye_pos.i_vec, focus_pos.i_vec, up_dir.i_vec)
		}
	}

	/// Gives view matrix for right-handed coordinate system
	pub fn look_at_rh(eye_pos: Float4, focus_pos: Float4, up_dir: Float4) -> Self {
		Matrix4x4 {
			mat: XMMatrixLookAtRH(eye_pos.i_vec, focus_pos.i_vec, up_dir.i_vec)
		}
	}

	/// Gives perspective matrix for left-handed coordinate system.
	pub fn perspective_lh(view_width: f32, view_height: f32, near: f32, far: f32) -> Self {
		Matrix4x4 {
			mat: XMMatrixPerspectiveLH(view_width, view_height, near, far)
		}
	}

	/// Gives perspective matrix for right-handed coordinate system.
	pub fn perspective_rh(view_width: f32, view_height: f32, near: f32, far: f32) -> Self {
		Matrix4x4 {
			mat: XMMatrixPerspectiveRH(view_width, view_height, near, far)
		}
	}

	/// Gives perspective matrix for left-handed coordinate system with fov setting.
	pub fn perspective_fov_lh(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
		Matrix4x4 {
			mat: XMMatrixPerspectiveLH(fov, aspect_ratio, near, far)
		}
	}

	/// Gives perspective matrix for right-handed coordinate system with fov setting.
	pub fn perspective_fov_rh(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
		Matrix4x4 {
			mat: XMMatrixPerspectiveRH(fov, aspect_ratio, near, far)
		}
	}

	/// Gives rotation matrix from roll, pitch and yaw.
	pub fn rotation_rpy(roll: f32, pitch: f32, yaw: f32) -> Self {
		Matrix4x4 {
			mat: XMMatrixRotationRollPitchYaw(pitch, yaw, roll)
		}
	}

	/// Matrices multiply.
	pub fn multiply(lhs: Self, rhs: Self) -> Self {
		Matrix4x4 {
			mat: XMMatrixMultiply(lhs.mat, &rhs.mat)
		}
	}

	/// Multiplies matrix by another.
	pub fn multiply_by(&mut self, another: &Self) {
		self.mat = XMMatrixMultiply(self.mat, &another.mat);
	}

	/// Transposes the matrix.
	pub fn transpose(&mut self) {
		self.mat = XMMatrixTranspose(self.mat)
	}
}

impl Default for Matrix4x4 {
	fn default() -> Self {
		Self::unit()
	}
}

impl Inverse for Matrix4x4 {
	fn inverse(&mut self) {
		self.mat = XMMatrixInverse(None, self.mat);
	}
}

impl std::ops::Not for Matrix4x4 {
	type Output = Self;
	fn not(mut self) -> Self {
		self.inverse();
		self
	}
}

impl std::ops::Mul<Self> for Matrix4x4 {
	type Output = Self;
	fn mul(self, another: Self) -> Self {
		Self {
			mat: XMMatrixMultiply(another.mat, &self.mat)
		}
	}
}

impl std::ops::MulAssign<Self> for Matrix4x4 {
	fn mul_assign(&mut self, another: Self) {
		self.mat = XMMatrixMultiply(self.mat, &another.mat)
	}
}

impl std::ops::Mul<Float4> for Matrix4x4 {
	type Output = Float4;
	fn mul(self, vec: Float4) -> Float4 {
		Float4 {
			i_vec: XMVector4Transform(vec.i_vec, self.mat)
		}
	}
}
