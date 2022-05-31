mod swizzle;
use directx_math::*;
use swizzle::{
	Swizzele4DCordsShuffle,
	Swizzle4Dto1,
	Set4Dto1,
	NewVec4,
};

/// Represents 4D 32-bit float vector.
#[derive(Clone, Copy)]
struct Float4 {
	pub i_vec: XMVECTOR
}

#[allow(dead_code)]
impl Float4 {
	/// Constructs vector from one number.
	pub fn all(xyzw: f32) -> Self {
		Self::new(xyzw, xyzw, xyzw, xyzw)
	}

	/// Constructs unit vector.
	pub fn unit() -> Self {
		Self::all(1.0)
	}

	/// Constructs vector from 3 floats and make W to be 1.0
	pub fn xyz1(x: f32, y: f32, z: f32) -> Self {
		Self::new(x, y, z, 1.0)
	}

	/// Constructs vector from 3 floats and make W to be 0.0
	pub fn xyz0(x: f32, y: f32, z: f32) -> Self {
		Self::new(x, y, z, 0.0)
	}
}

impl NewVec4<f32> for Float4 {
	/// Constructs vector from 4 floats.
	fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
		Float4 {
			i_vec: XMVectorSet(x, y, z, w)
		}
	}
}

impl Default for Float4 {
	fn default() -> Self {
		Self::new(0.0, 0.0, 0.0, 0.0)
	}
}

impl PartialEq for Float4 {
	fn eq(&self, other: &Self) -> bool {
		self.x() == other.x() &&
		self.y() == other.y() &&
		self.z() == other.z() &&
		self.w() == other.w()
	}
	fn ne(&self, other: &Self) -> bool {
		self.x() != other.x() ||
		self.y() != other.y() ||
		self.z() != other.z() ||
		self.w() != other.w()
	}
}

impl std::ops::Neg for Float4 {
	type Output = Self;
	fn neg(self) -> Self {
		Self::new(-self.x(), -self.y(), -self.z(), -self.w())
	}
}

impl std::ops::Sub for Float4 {
	type Output = Self;
	fn sub(self, other: Self) -> Self {
		Self::new(
			self.x() - other.x(),
			self.y() - other.y(),
			self.z() - other.z(),
			self.w() - other.w()
		)
	}
}

impl std::ops::SubAssign for Float4 {
	fn sub_assign(&mut self, other: Self) {
		self.set_x(self.x() - other.x());
		self.set_y(self.y() - other.y());
		self.set_z(self.z() - other.z());
		self.set_w(self.w() - other.w());
	}
}

impl std::ops::Add for Float4 {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		Self::new(
			self.x() + other.x(),
			self.y() + other.y(),
			self.z() + other.z(),
			self.w() + other.w()
		)
	}
}

impl std::ops::AddAssign for Float4 {
	fn add_assign(&mut self, other: Self) {
		self.set_x(self.x() + other.x());
		self.set_y(self.y() + other.y());
		self.set_z(self.z() + other.z());
		self.set_w(self.w() + other.w());
	}
}

impl std::ops::Mul<f32> for Float4 {
	type Output = Self;
	fn mul(self, k: f32) -> Self {
		Self::new(self.x() * k, self.y() * k, self.z() * k, self.w() * k)
	}
}

impl std::ops::MulAssign<f32> for Float4 {
	fn mul_assign(&mut self, k: f32) {
		self.set_x(self.x() * k);
		self.set_y(self.y() * k);
		self.set_z(self.z() * k);
		self.set_w(self.w() * k);
	}
}

impl std::ops::Div<f32> for Float4 {
	type Output = Self;
	fn div(self, k: f32) -> Self {
		assert!(k != 0.0, "Cannot divide by 0!");
		Self::new(self.x() / k, self.y() / k, self.z() / k, self.w() / k)
	}
}

impl std::ops::DivAssign<f32> for Float4 {
	fn div_assign(&mut self, k: f32) {
		assert!(k != 0.0, "Cannot divide by 0!");
		self.set_x(self.x() / k);
		self.set_y(self.y() / k);
		self.set_z(self.z() / k);
		self.set_w(self.w() / k);
	}
}

/**
 * Swizzle section
 */

impl Swizzle4Dto1<f32> for Float4 {
	fn x(self) -> f32 { XMVectorGetX(self.i_vec) }
	fn y(self) -> f32 { XMVectorGetY(self.i_vec) }
	fn z(self) -> f32 { XMVectorGetZ(self.i_vec) }
	fn w(self) -> f32 { XMVectorGetW(self.i_vec) }
}

impl Set4Dto1<f32> for Float4 {
	fn set_x(&mut self, other: f32) { self.i_vec = XMVectorSetX(self.i_vec, other); }
	fn set_y(&mut self, other: f32) { self.i_vec = XMVectorSetY(self.i_vec, other); }
	fn set_z(&mut self, other: f32) { self.i_vec = XMVectorSetZ(self.i_vec, other); }
	fn set_w(&mut self, other: f32) { self.i_vec = XMVectorSetW(self.i_vec, other); }
} 

impl Swizzele4DCordsShuffle<f32> for Float4 { }