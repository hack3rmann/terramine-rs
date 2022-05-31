/*!
 * Defines some swizzele traits for vectors.
 */

pub trait NewVec4<T> {
	fn new(x: T, y: T, z: T, w: T) -> Self;
}

pub trait Swizzle4Dto1<T>: Copy {
	fn x(self) -> T;
	fn y(self) -> T;
	fn z(self) -> T;
	fn w(self) -> T;
}

pub trait Set4Dto1<T> {
	fn set_x(&mut self, other: T);
	fn set_y(&mut self, other: T);
	fn set_z(&mut self, other: T);
	fn set_w(&mut self, other: T);
}

pub trait Swizzele4DCordsShuffle<T>: Copy + NewVec4<T> + Swizzle4Dto1<T> {
	fn xyzw(self) -> Self { Self::new(self.x(), self.y(), self.z(), self.w()) }
	fn xywz(self) -> Self { Self::new(self.x(), self.y(), self.w(), self.z()) }
	fn xzyw(self) -> Self { Self::new(self.x(), self.z(), self.y(), self.w()) }
	fn xzwy(self) -> Self { Self::new(self.x(), self.z(), self.w(), self.y()) }
	fn xwyz(self) -> Self { Self::new(self.x(), self.w(), self.y(), self.z()) }
	fn xwzy(self) -> Self { Self::new(self.x(), self.w(), self.z(), self.y()) }

	fn yxzw(self) -> Self { Self::new(self.y(), self.x(), self.z(), self.w()) }
	fn yxwz(self) -> Self { Self::new(self.y(), self.x(), self.w(), self.z()) }
	fn yzxw(self) -> Self { Self::new(self.y(), self.z(), self.x(), self.w()) }
	fn yzwx(self) -> Self { Self::new(self.y(), self.z(), self.w(), self.x()) }
	fn ywxz(self) -> Self { Self::new(self.y(), self.w(), self.x(), self.z()) }
	fn ywzx(self) -> Self { Self::new(self.y(), self.w(), self.z(), self.x()) }

	fn zxyw(self) -> Self { Self::new(self.z(), self.x(), self.y(), self.w()) }
	fn zxwy(self) -> Self { Self::new(self.z(), self.x(), self.w(), self.y()) }
	fn zyxw(self) -> Self { Self::new(self.z(), self.y(), self.x(), self.w()) }
	fn zywx(self) -> Self { Self::new(self.z(), self.y(), self.w(), self.x()) }
	fn zwxy(self) -> Self { Self::new(self.z(), self.w(), self.x(), self.y()) }
	fn zwyx(self) -> Self { Self::new(self.z(), self.w(), self.y(), self.x()) }

	fn wxyz(self) -> Self { Self::new(self.w(), self.x(), self.y(), self.z()) }
	fn wxzy(self) -> Self { Self::new(self.w(), self.x(), self.z(), self.y()) }
	fn wyxz(self) -> Self { Self::new(self.w(), self.y(), self.x(), self.z()) }
	fn wyzx(self) -> Self { Self::new(self.w(), self.y(), self.z(), self.x()) }
	fn wzxy(self) -> Self { Self::new(self.w(), self.z(), self.x(), self.y()) }
	fn wzyx(self) -> Self { Self::new(self.w(), self.z(), self.y(), self.x()) }
}