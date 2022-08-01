pub mod swizzle;
pub use swizzle::*;

use {
	directx_math::*,
	crate::app::utils::reinterpreter::*
};

/// Represents 4D 32-bit float vector.
#[derive(Clone, Copy, Debug)]
pub struct Float4 {
	pub i_vec: XMVECTOR
}

/// Represents 3D 32-bit int vector.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Int3 {
	x: i32,
	y: i32,
	z: i32
}

#[allow(dead_code)]
impl Float4 {
	/// Constructs vector from one number.
	pub fn all(xyzw: f32) -> Self {
		Self::new(xyzw, xyzw, xyzw, xyzw)
	}

	/// Constructs unit vector.
	pub fn one() -> Self {
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
	
	/// Normalyzes the vector.
	pub fn normalyze(self) -> Self {
		Float4 {
			i_vec: XMVector3Normalize(self.i_vec)
		}
	}

	/// Gives dot product of two vectors
	pub fn dot(self, other: Float4) -> f32 {
		XMVectorGetX(XMVector3Dot(self.i_vec, other.i_vec))
	}

	/// Gives cross product of two vectors
	pub fn cross(self, other: Float4) -> Self {
		Float4 {
			i_vec: XMVector3Cross(self.i_vec, other.i_vec)
		}
	}

	/// Gives absolute value of vector
	pub fn abs(self) -> f32 {
		XMVectorGetX(XMVector3Length(self.i_vec))
	}
}

#[allow(dead_code)]
impl Int3 {
	/// Constructs vector from one number.
	pub fn all(xyz: i32) -> Self {
		Self::new(xyz, xyz, xyz)
	}

	/// Constructs unit vector.
	pub fn unit() -> Self {
		Self::all(1)
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

impl NewVec3<i32> for Int3 {
	/// Constructs vector from 3 integers
	fn new(x: i32, y: i32, z: i32) -> Self {
		Int3 { x, y, z }
	}
}

impl Default for Float4 {
	fn default() -> Self {
		Self::all(0.0)
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

impl std::ops::Neg for Int3 {
	type Output = Self;
	fn neg(self) -> Self {
		Self::new(-self.x, -self.y, -self.z)
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

impl std::ops::Sub for Int3 {
	type Output = Self;
	fn sub(self, other: Self) -> Self {
		Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
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

impl std::ops::SubAssign for Int3 {
	fn sub_assign(&mut self, other: Self) {
		self.x -= other.x;
		self.y -= other.y;
		self.z -= other.z;
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

impl std::ops::Add for Int3 {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
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

impl std::ops::AddAssign for Int3 {
	fn add_assign(&mut self, other: Self) {
		self.x += other.x;
		self.y += other.y;
		self.z += other.z;
	}
}

impl std::ops::Mul<f32> for Float4 {
	type Output = Self;
	fn mul(self, k: f32) -> Self {
		Self::new(self.x() * k, self.y() * k, self.z() * k, self.w() * k)
	}
}

impl std::ops::Mul<i32> for Int3 {
	type Output = Self;
	fn mul(self, k: i32) -> Self {
		Self::new(self.x * k , self.y * k, self.z * k)
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

impl std::ops::MulAssign<i32> for Int3 {
	fn mul_assign(&mut self, k: i32) {
		self.x *= k;
		self.y *= k;
		self.z *= k;
	}
}

impl std::ops::Div<f32> for Float4 {
	type Output = Self;
	fn div(self, k: f32) -> Self {
		assert_ne!(k, 0.0, "Cannot divide by 0!");
		Self::new(self.x() / k, self.y() / k, self.z() / k, self.w() / k)
	}
}

impl std::ops::Div<i32> for Int3 {
	type Output = Self;
	fn div(self, k: i32) -> Self {
		assert_ne!(k, 0, "Cannot divide by 0!");
		Self::new(self.x / k, self.y / k, self.z / k)
	}
}

impl std::ops::DivAssign<f32> for Float4 {
	fn div_assign(&mut self, k: f32) {
		assert_ne!(k, 0.0, "Cannot divide by 0!");
		self.set_x(self.x() / k);
		self.set_y(self.y() / k);
		self.set_z(self.z() / k);
		self.set_w(self.w() / k);
	}
}

impl std::ops::DivAssign<i32> for Int3 {
	fn div_assign(&mut self, k: i32) {
		assert_ne!(k, 0, "Cannot divide by 0!");
		self.x /= k;
		self.y /= k;
		self.z /= k;
	}
}

impl std::ops::RemAssign for Int3 {
	fn rem_assign(&mut self, rhs: Self) {
		self.x %= rhs.x();
		self.y %= rhs.y();
		self.z %= rhs.z();
	}
}

impl std::ops::Rem for Int3 {
	type Output = Int3;
	fn rem(mut self, rhs: Self) -> Self::Output {
		self %= rhs;
		return self
	}
}

impl std::ops::RemAssign<i32> for Int3 {
	fn rem_assign(&mut self, rhs: i32) {
		self.x %= rhs;
		self.y %= rhs;
		self.z %= rhs;
	}
}

impl std::ops::Rem<i32> for Int3 {
	type Output = Int3;
	fn rem(mut self, rhs: i32) -> Self::Output {
		self %= rhs;
		return self
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

impl Swizzle3Dto1<i32> for Int3 {
	fn x(self) -> i32 { self.x }
	fn y(self) -> i32 { self.y }
	fn z(self) -> i32 { self.z }
}

impl Set4Dto1<f32> for Float4 {
	fn set_x(&mut self, other: f32) { self.i_vec = XMVectorSetX(self.i_vec, other); }
	fn set_y(&mut self, other: f32) { self.i_vec = XMVectorSetY(self.i_vec, other); }
	fn set_z(&mut self, other: f32) { self.i_vec = XMVectorSetZ(self.i_vec, other); }
	fn set_w(&mut self, other: f32) { self.i_vec = XMVectorSetW(self.i_vec, other); }
}

impl Set3Dto1<i32> for Int3 {
	fn set_x(&mut self, other: i32) { self.x = other; }
	fn set_y(&mut self, other: i32) { self.y = other; }
	fn set_z(&mut self, other: i32) { self.z = other; }
}

impl Vec4Swizzles4<f32> for Float4 { }
impl Vec3Swizzles3<i32> for Int3 { }

/**
 * Reinterpretor section
 */

unsafe impl Reinterpret for Int3 { }

unsafe impl ReinterpretAsBytes for Int3 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		let mut out = Vec::with_capacity(Self::static_size());

		out.append(&mut self.x().reinterpret_as_bytes());
		out.append(&mut self.y().reinterpret_as_bytes());
		out.append(&mut self.z().reinterpret_as_bytes());

		return out;
	}
}

unsafe impl ReinterpretFromBytes for Int3 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		let x = i32::reinterpret_from_bytes(&source[0..4]);
		let y = i32::reinterpret_from_bytes(&source[4..8]);
		let z = i32::reinterpret_from_bytes(&source[8..12]);

		Self::new(x, y, z)
	}
}

unsafe impl ReinterpretSize for Int3 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for Int3 {
	fn static_size() -> usize { 12 }
}



unsafe impl Reinterpret for Float4 { }

unsafe impl ReinterpretAsBytes for Float4 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		let mut out = Vec::with_capacity(Self::static_size());

		out.append(&mut self.x().reinterpret_as_bytes());
		out.append(&mut self.y().reinterpret_as_bytes());
		out.append(&mut self.z().reinterpret_as_bytes());
		out.append(&mut self.w().reinterpret_as_bytes());

		return out;
	}
}

unsafe impl ReinterpretFromBytes for Float4 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		let x = f32::reinterpret_from_bytes(&source[0..4]);
		let y = f32::reinterpret_from_bytes(&source[4..8]);
		let z = f32::reinterpret_from_bytes(&source[8..12]);
		let w = f32::reinterpret_from_bytes(&source[12..16]);

		Self::new(x, y, z, w)
	}
}

unsafe impl ReinterpretSize for Float4 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for Float4 {
	fn static_size() -> usize { 16 }
}



#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn reinterpret_int3() {
		let before = Int3::new(23, 441, 52);
		let after = Int3::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_float4() {
		let before = Float4::new(233.7, 123.5, 123123.5, 444.5);
		let after = Float4::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}
}