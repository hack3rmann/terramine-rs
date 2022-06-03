/*!
 * Defines some swizzele traits for vectors.
 */

pub trait NewVec4<T> {
	fn new(x: T, y: T, z: T, w: T) -> Self;
}

pub trait NewVec3<T> {
	fn new(x: T, y: T, z: T) -> Self;
}

pub trait NewVec2<T> {
	fn new(x: T, y: T) -> Self;
}

pub trait Swizzle4Dto1<T>: Copy {
	fn x(self) -> T;
	fn y(self) -> T;
	fn z(self) -> T;
	fn w(self) -> T;
}

pub trait Swizzle3Dto1<T>: Copy {
	fn x(self) -> T;
	fn y(self) -> T;
	fn z(self) -> T;
}

pub trait Swizzle2Dto1<T>: Copy {
	fn x(self) -> T;
	fn y(self) -> T;
}

pub trait Set4Dto1<T> {
	fn set_x(&mut self, other: T);
	fn set_y(&mut self, other: T);
	fn set_z(&mut self, other: T);
	fn set_w(&mut self, other: T);
}

pub trait Set3Dto1<T> {
	fn set_x(&mut self, other: T);
	fn set_y(&mut self, other: T);
	fn set_z(&mut self, other: T);
}

pub trait Set2Dto1<T> {
	fn set_x(&mut self, other: T);
	fn set_y(&mut self, other: T);
}

pub trait Vec4Swizzles4<T>: NewVec4<T> + Swizzle4Dto1<T> + Copy {
	#[inline]
	fn xyzw(self) -> Self { self }

	fn xxxx(self) -> Self { Self::new(self.x(), self.x(), self.x(), self.x()) }
	fn xxxy(self) -> Self { Self::new(self.x(), self.x(), self.x(), self.y()) }
	fn xxxz(self) -> Self { Self::new(self.x(), self.x(), self.x(), self.z()) }
	fn xxxw(self) -> Self { Self::new(self.x(), self.x(), self.x(), self.w()) }
	fn xxyx(self) -> Self { Self::new(self.x(), self.x(), self.y(), self.x()) }
	fn xxyy(self) -> Self { Self::new(self.x(), self.x(), self.y(), self.y()) }
	fn xxyz(self) -> Self { Self::new(self.x(), self.x(), self.y(), self.z()) }
	fn xxyw(self) -> Self { Self::new(self.x(), self.x(), self.y(), self.w()) }
	fn xxzx(self) -> Self { Self::new(self.x(), self.x(), self.z(), self.x()) }
	fn xxzy(self) -> Self { Self::new(self.x(), self.x(), self.z(), self.y()) }
	fn xxzz(self) -> Self { Self::new(self.x(), self.x(), self.z(), self.z()) }
	fn xxzw(self) -> Self { Self::new(self.x(), self.x(), self.z(), self.w()) }
	fn xxwx(self) -> Self { Self::new(self.x(), self.x(), self.w(), self.x()) }
	fn xxwy(self) -> Self { Self::new(self.x(), self.x(), self.w(), self.y()) }
	fn xxwz(self) -> Self { Self::new(self.x(), self.x(), self.w(), self.z()) }
	fn xxww(self) -> Self { Self::new(self.x(), self.x(), self.w(), self.w()) }
	fn xyxx(self) -> Self { Self::new(self.x(), self.y(), self.x(), self.x()) }
	fn xyxy(self) -> Self { Self::new(self.x(), self.y(), self.x(), self.y()) }
	fn xyxz(self) -> Self { Self::new(self.x(), self.y(), self.x(), self.z()) }
	fn xyxw(self) -> Self { Self::new(self.x(), self.y(), self.x(), self.w()) }
	fn xyyx(self) -> Self { Self::new(self.x(), self.y(), self.y(), self.x()) }
	fn xyyy(self) -> Self { Self::new(self.x(), self.y(), self.y(), self.y()) }
	fn xyyz(self) -> Self { Self::new(self.x(), self.y(), self.y(), self.z()) }
	fn xyyw(self) -> Self { Self::new(self.x(), self.y(), self.y(), self.w()) }
	fn xyzx(self) -> Self { Self::new(self.x(), self.y(), self.z(), self.x()) }
	fn xyzy(self) -> Self { Self::new(self.x(), self.y(), self.z(), self.y()) }
	fn xyzz(self) -> Self { Self::new(self.x(), self.y(), self.z(), self.z()) }
	fn xywx(self) -> Self { Self::new(self.x(), self.y(), self.w(), self.x()) }
	fn xywy(self) -> Self { Self::new(self.x(), self.y(), self.w(), self.y()) }
	fn xywz(self) -> Self { Self::new(self.x(), self.y(), self.w(), self.z()) }
	fn xyww(self) -> Self { Self::new(self.x(), self.y(), self.w(), self.w()) }
	fn xzxx(self) -> Self { Self::new(self.x(), self.z(), self.x(), self.x()) }
	fn xzxy(self) -> Self { Self::new(self.x(), self.z(), self.x(), self.y()) }
	fn xzxz(self) -> Self { Self::new(self.x(), self.z(), self.x(), self.z()) }
	fn xzxw(self) -> Self { Self::new(self.x(), self.z(), self.x(), self.w()) }
	fn xzyx(self) -> Self { Self::new(self.x(), self.z(), self.y(), self.x()) }
	fn xzyy(self) -> Self { Self::new(self.x(), self.z(), self.y(), self.y()) }
	fn xzyz(self) -> Self { Self::new(self.x(), self.z(), self.y(), self.z()) }
	fn xzyw(self) -> Self { Self::new(self.x(), self.z(), self.y(), self.w()) }
	fn xzzx(self) -> Self { Self::new(self.x(), self.z(), self.z(), self.x()) }
	fn xzzy(self) -> Self { Self::new(self.x(), self.z(), self.z(), self.y()) }
	fn xzzz(self) -> Self { Self::new(self.x(), self.z(), self.z(), self.z()) }
	fn xzzw(self) -> Self { Self::new(self.x(), self.z(), self.z(), self.w()) }
	fn xzwx(self) -> Self { Self::new(self.x(), self.z(), self.w(), self.x()) }
	fn xzwy(self) -> Self { Self::new(self.x(), self.z(), self.w(), self.y()) }
	fn xzwz(self) -> Self { Self::new(self.x(), self.z(), self.w(), self.z()) }
	fn xzww(self) -> Self { Self::new(self.x(), self.z(), self.w(), self.w()) }
	fn xwxx(self) -> Self { Self::new(self.x(), self.w(), self.x(), self.x()) }
	fn xwxy(self) -> Self { Self::new(self.x(), self.w(), self.x(), self.y()) }
	fn xwxz(self) -> Self { Self::new(self.x(), self.w(), self.x(), self.z()) }
	fn xwxw(self) -> Self { Self::new(self.x(), self.w(), self.x(), self.w()) }
	fn xwyx(self) -> Self { Self::new(self.x(), self.w(), self.y(), self.x()) }
	fn xwyy(self) -> Self { Self::new(self.x(), self.w(), self.y(), self.y()) }
	fn xwyz(self) -> Self { Self::new(self.x(), self.w(), self.y(), self.z()) }
	fn xwyw(self) -> Self { Self::new(self.x(), self.w(), self.y(), self.w()) }
	fn xwzx(self) -> Self { Self::new(self.x(), self.w(), self.z(), self.x()) }
	fn xwzy(self) -> Self { Self::new(self.x(), self.w(), self.z(), self.y()) }
	fn xwzz(self) -> Self { Self::new(self.x(), self.w(), self.z(), self.z()) }
	fn xwzw(self) -> Self { Self::new(self.x(), self.w(), self.z(), self.w()) }
	fn xwwx(self) -> Self { Self::new(self.x(), self.w(), self.w(), self.x()) }
	fn xwwy(self) -> Self { Self::new(self.x(), self.w(), self.w(), self.y()) }
	fn xwwz(self) -> Self { Self::new(self.x(), self.w(), self.w(), self.z()) }
	fn xwww(self) -> Self { Self::new(self.x(), self.w(), self.w(), self.w()) }
	fn yxxx(self) -> Self { Self::new(self.y(), self.x(), self.x(), self.x()) }
	fn yxxy(self) -> Self { Self::new(self.y(), self.x(), self.x(), self.y()) }
	fn yxxz(self) -> Self { Self::new(self.y(), self.x(), self.x(), self.z()) }
	fn yxxw(self) -> Self { Self::new(self.y(), self.x(), self.x(), self.w()) }
	fn yxyx(self) -> Self { Self::new(self.y(), self.x(), self.y(), self.x()) }
	fn yxyy(self) -> Self { Self::new(self.y(), self.x(), self.y(), self.y()) }
	fn yxyz(self) -> Self { Self::new(self.y(), self.x(), self.y(), self.z()) }
	fn yxyw(self) -> Self { Self::new(self.y(), self.x(), self.y(), self.w()) }
	fn yxzx(self) -> Self { Self::new(self.y(), self.x(), self.z(), self.x()) }
	fn yxzy(self) -> Self { Self::new(self.y(), self.x(), self.z(), self.y()) }
	fn yxzz(self) -> Self { Self::new(self.y(), self.x(), self.z(), self.z()) }
	fn yxzw(self) -> Self { Self::new(self.y(), self.x(), self.z(), self.w()) }
	fn yxwx(self) -> Self { Self::new(self.y(), self.x(), self.w(), self.x()) }
	fn yxwy(self) -> Self { Self::new(self.y(), self.x(), self.w(), self.y()) }
	fn yxwz(self) -> Self { Self::new(self.y(), self.x(), self.w(), self.z()) }
	fn yxww(self) -> Self { Self::new(self.y(), self.x(), self.w(), self.w()) }
	fn yyxx(self) -> Self { Self::new(self.y(), self.y(), self.x(), self.x()) }
	fn yyxy(self) -> Self { Self::new(self.y(), self.y(), self.x(), self.y()) }
	fn yyxz(self) -> Self { Self::new(self.y(), self.y(), self.x(), self.z()) }
	fn yyxw(self) -> Self { Self::new(self.y(), self.y(), self.x(), self.w()) }
	fn yyyx(self) -> Self { Self::new(self.y(), self.y(), self.y(), self.x()) }
	fn yyyy(self) -> Self { Self::new(self.y(), self.y(), self.y(), self.y()) }
	fn yyyz(self) -> Self { Self::new(self.y(), self.y(), self.y(), self.z()) }
	fn yyyw(self) -> Self { Self::new(self.y(), self.y(), self.y(), self.w()) }
	fn yyzx(self) -> Self { Self::new(self.y(), self.y(), self.z(), self.x()) }
	fn yyzy(self) -> Self { Self::new(self.y(), self.y(), self.z(), self.y()) }
	fn yyzz(self) -> Self { Self::new(self.y(), self.y(), self.z(), self.z()) }
	fn yyzw(self) -> Self { Self::new(self.y(), self.y(), self.z(), self.w()) }
	fn yywx(self) -> Self { Self::new(self.y(), self.y(), self.w(), self.x()) }
	fn yywy(self) -> Self { Self::new(self.y(), self.y(), self.w(), self.y()) }
	fn yywz(self) -> Self { Self::new(self.y(), self.y(), self.w(), self.z()) }
	fn yyww(self) -> Self { Self::new(self.y(), self.y(), self.w(), self.w()) }
	fn yzxx(self) -> Self { Self::new(self.y(), self.z(), self.x(), self.x()) }
	fn yzxy(self) -> Self { Self::new(self.y(), self.z(), self.x(), self.y()) }
	fn yzxz(self) -> Self { Self::new(self.y(), self.z(), self.x(), self.z()) }
	fn yzxw(self) -> Self { Self::new(self.y(), self.z(), self.x(), self.w()) }
	fn yzyx(self) -> Self { Self::new(self.y(), self.z(), self.y(), self.x()) }
	fn yzyy(self) -> Self { Self::new(self.y(), self.z(), self.y(), self.y()) }
	fn yzyz(self) -> Self { Self::new(self.y(), self.z(), self.y(), self.z()) }
	fn yzyw(self) -> Self { Self::new(self.y(), self.z(), self.y(), self.w()) }
	fn yzzx(self) -> Self { Self::new(self.y(), self.z(), self.z(), self.x()) }
	fn yzzy(self) -> Self { Self::new(self.y(), self.z(), self.z(), self.y()) }
	fn yzzz(self) -> Self { Self::new(self.y(), self.z(), self.z(), self.z()) }
	fn yzzw(self) -> Self { Self::new(self.y(), self.z(), self.z(), self.w()) }
	fn yzwx(self) -> Self { Self::new(self.y(), self.z(), self.w(), self.x()) }
	fn yzwy(self) -> Self { Self::new(self.y(), self.z(), self.w(), self.y()) }
	fn yzwz(self) -> Self { Self::new(self.y(), self.z(), self.w(), self.z()) }
	fn yzww(self) -> Self { Self::new(self.y(), self.z(), self.w(), self.w()) }
	fn ywxx(self) -> Self { Self::new(self.y(), self.w(), self.x(), self.x()) }
	fn ywxy(self) -> Self { Self::new(self.y(), self.w(), self.x(), self.y()) }
	fn ywxz(self) -> Self { Self::new(self.y(), self.w(), self.x(), self.z()) }
	fn ywxw(self) -> Self { Self::new(self.y(), self.w(), self.x(), self.w()) }
	fn ywyx(self) -> Self { Self::new(self.y(), self.w(), self.y(), self.x()) }
	fn ywyy(self) -> Self { Self::new(self.y(), self.w(), self.y(), self.y()) }
	fn ywyz(self) -> Self { Self::new(self.y(), self.w(), self.y(), self.z()) }
	fn ywyw(self) -> Self { Self::new(self.y(), self.w(), self.y(), self.w()) }
	fn ywzx(self) -> Self { Self::new(self.y(), self.w(), self.z(), self.x()) }
	fn ywzy(self) -> Self { Self::new(self.y(), self.w(), self.z(), self.y()) }
	fn ywzz(self) -> Self { Self::new(self.y(), self.w(), self.z(), self.z()) }
	fn ywzw(self) -> Self { Self::new(self.y(), self.w(), self.z(), self.w()) }
	fn ywwx(self) -> Self { Self::new(self.y(), self.w(), self.w(), self.x()) }
	fn ywwy(self) -> Self { Self::new(self.y(), self.w(), self.w(), self.y()) }
	fn ywwz(self) -> Self { Self::new(self.y(), self.w(), self.w(), self.z()) }
	fn ywww(self) -> Self { Self::new(self.y(), self.w(), self.w(), self.w()) }
	fn zxxx(self) -> Self { Self::new(self.z(), self.x(), self.x(), self.x()) }
	fn zxxy(self) -> Self { Self::new(self.z(), self.x(), self.x(), self.y()) }
	fn zxxz(self) -> Self { Self::new(self.z(), self.x(), self.x(), self.z()) }
	fn zxxw(self) -> Self { Self::new(self.z(), self.x(), self.x(), self.w()) }
	fn zxyx(self) -> Self { Self::new(self.z(), self.x(), self.y(), self.x()) }
	fn zxyy(self) -> Self { Self::new(self.z(), self.x(), self.y(), self.y()) }
	fn zxyz(self) -> Self { Self::new(self.z(), self.x(), self.y(), self.z()) }
	fn zxyw(self) -> Self { Self::new(self.z(), self.x(), self.y(), self.w()) }
	fn zxzx(self) -> Self { Self::new(self.z(), self.x(), self.z(), self.x()) }
	fn zxzy(self) -> Self { Self::new(self.z(), self.x(), self.z(), self.y()) }
	fn zxzz(self) -> Self { Self::new(self.z(), self.x(), self.z(), self.z()) }
	fn zxzw(self) -> Self { Self::new(self.z(), self.x(), self.z(), self.w()) }
	fn zxwx(self) -> Self { Self::new(self.z(), self.x(), self.w(), self.x()) }
	fn zxwy(self) -> Self { Self::new(self.z(), self.x(), self.w(), self.y()) }
	fn zxwz(self) -> Self { Self::new(self.z(), self.x(), self.w(), self.z()) }
	fn zxww(self) -> Self { Self::new(self.z(), self.x(), self.w(), self.w()) }
	fn zyxx(self) -> Self { Self::new(self.z(), self.y(), self.x(), self.x()) }
	fn zyxy(self) -> Self { Self::new(self.z(), self.y(), self.x(), self.y()) }
	fn zyxz(self) -> Self { Self::new(self.z(), self.y(), self.x(), self.z()) }
	fn zyxw(self) -> Self { Self::new(self.z(), self.y(), self.x(), self.w()) }
	fn zyyx(self) -> Self { Self::new(self.z(), self.y(), self.y(), self.x()) }
	fn zyyy(self) -> Self { Self::new(self.z(), self.y(), self.y(), self.y()) }
	fn zyyz(self) -> Self { Self::new(self.z(), self.y(), self.y(), self.z()) }
	fn zyyw(self) -> Self { Self::new(self.z(), self.y(), self.y(), self.w()) }
	fn zyzx(self) -> Self { Self::new(self.z(), self.y(), self.z(), self.x()) }
	fn zyzy(self) -> Self { Self::new(self.z(), self.y(), self.z(), self.y()) }
	fn zyzz(self) -> Self { Self::new(self.z(), self.y(), self.z(), self.z()) }
	fn zyzw(self) -> Self { Self::new(self.z(), self.y(), self.z(), self.w()) }
	fn zywx(self) -> Self { Self::new(self.z(), self.y(), self.w(), self.x()) }
	fn zywy(self) -> Self { Self::new(self.z(), self.y(), self.w(), self.y()) }
	fn zywz(self) -> Self { Self::new(self.z(), self.y(), self.w(), self.z()) }
	fn zyww(self) -> Self { Self::new(self.z(), self.y(), self.w(), self.w()) }
	fn zzxx(self) -> Self { Self::new(self.z(), self.z(), self.x(), self.x()) }
	fn zzxy(self) -> Self { Self::new(self.z(), self.z(), self.x(), self.y()) }
	fn zzxz(self) -> Self { Self::new(self.z(), self.z(), self.x(), self.z()) }
	fn zzxw(self) -> Self { Self::new(self.z(), self.z(), self.x(), self.w()) }
	fn zzyx(self) -> Self { Self::new(self.z(), self.z(), self.y(), self.x()) }
	fn zzyy(self) -> Self { Self::new(self.z(), self.z(), self.y(), self.y()) }
	fn zzyz(self) -> Self { Self::new(self.z(), self.z(), self.y(), self.z()) }
	fn zzyw(self) -> Self { Self::new(self.z(), self.z(), self.y(), self.w()) }
	fn zzzx(self) -> Self { Self::new(self.z(), self.z(), self.z(), self.x()) }
	fn zzzy(self) -> Self { Self::new(self.z(), self.z(), self.z(), self.y()) }
	fn zzzz(self) -> Self { Self::new(self.z(), self.z(), self.z(), self.z()) }
	fn zzzw(self) -> Self { Self::new(self.z(), self.z(), self.z(), self.w()) }
	fn zzwx(self) -> Self { Self::new(self.z(), self.z(), self.w(), self.x()) }
	fn zzwy(self) -> Self { Self::new(self.z(), self.z(), self.w(), self.y()) }
	fn zzwz(self) -> Self { Self::new(self.z(), self.z(), self.w(), self.z()) }
	fn zzww(self) -> Self { Self::new(self.z(), self.z(), self.w(), self.w()) }
	fn zwxx(self) -> Self { Self::new(self.z(), self.w(), self.x(), self.x()) }
	fn zwxy(self) -> Self { Self::new(self.z(), self.w(), self.x(), self.y()) }
	fn zwxz(self) -> Self { Self::new(self.z(), self.w(), self.x(), self.z()) }
	fn zwxw(self) -> Self { Self::new(self.z(), self.w(), self.x(), self.w()) }
	fn zwyx(self) -> Self { Self::new(self.z(), self.w(), self.y(), self.x()) }
	fn zwyy(self) -> Self { Self::new(self.z(), self.w(), self.y(), self.y()) }
	fn zwyz(self) -> Self { Self::new(self.z(), self.w(), self.y(), self.z()) }
	fn zwyw(self) -> Self { Self::new(self.z(), self.w(), self.y(), self.w()) }
	fn zwzx(self) -> Self { Self::new(self.z(), self.w(), self.z(), self.x()) }
	fn zwzy(self) -> Self { Self::new(self.z(), self.w(), self.z(), self.y()) }
	fn zwzz(self) -> Self { Self::new(self.z(), self.w(), self.z(), self.z()) }
	fn zwzw(self) -> Self { Self::new(self.z(), self.w(), self.z(), self.w()) }
	fn zwwx(self) -> Self { Self::new(self.z(), self.w(), self.w(), self.x()) }
	fn zwwy(self) -> Self { Self::new(self.z(), self.w(), self.w(), self.y()) }
	fn zwwz(self) -> Self { Self::new(self.z(), self.w(), self.w(), self.z()) }
	fn zwww(self) -> Self { Self::new(self.z(), self.w(), self.w(), self.w()) }
	fn wxxx(self) -> Self { Self::new(self.w(), self.x(), self.x(), self.x()) }
	fn wxxy(self) -> Self { Self::new(self.w(), self.x(), self.x(), self.y()) }
	fn wxxz(self) -> Self { Self::new(self.w(), self.x(), self.x(), self.z()) }
	fn wxxw(self) -> Self { Self::new(self.w(), self.x(), self.x(), self.w()) }
	fn wxyx(self) -> Self { Self::new(self.w(), self.x(), self.y(), self.x()) }
	fn wxyy(self) -> Self { Self::new(self.w(), self.x(), self.y(), self.y()) }
	fn wxyz(self) -> Self { Self::new(self.w(), self.x(), self.y(), self.z()) }
	fn wxyw(self) -> Self { Self::new(self.w(), self.x(), self.y(), self.w()) }
	fn wxzx(self) -> Self { Self::new(self.w(), self.x(), self.z(), self.x()) }
	fn wxzy(self) -> Self { Self::new(self.w(), self.x(), self.z(), self.y()) }
	fn wxzz(self) -> Self { Self::new(self.w(), self.x(), self.z(), self.z()) }
	fn wxzw(self) -> Self { Self::new(self.w(), self.x(), self.z(), self.w()) }
	fn wxwx(self) -> Self { Self::new(self.w(), self.x(), self.w(), self.x()) }
	fn wxwy(self) -> Self { Self::new(self.w(), self.x(), self.w(), self.y()) }
	fn wxwz(self) -> Self { Self::new(self.w(), self.x(), self.w(), self.z()) }
	fn wxww(self) -> Self { Self::new(self.w(), self.x(), self.w(), self.w()) }
	fn wyxx(self) -> Self { Self::new(self.w(), self.y(), self.x(), self.x()) }
	fn wyxy(self) -> Self { Self::new(self.w(), self.y(), self.x(), self.y()) }
	fn wyxz(self) -> Self { Self::new(self.w(), self.y(), self.x(), self.z()) }
	fn wyxw(self) -> Self { Self::new(self.w(), self.y(), self.x(), self.w()) }
	fn wyyx(self) -> Self { Self::new(self.w(), self.y(), self.y(), self.x()) }
	fn wyyy(self) -> Self { Self::new(self.w(), self.y(), self.y(), self.y()) }
	fn wyyz(self) -> Self { Self::new(self.w(), self.y(), self.y(), self.z()) }
	fn wyyw(self) -> Self { Self::new(self.w(), self.y(), self.y(), self.w()) }
	fn wyzx(self) -> Self { Self::new(self.w(), self.y(), self.z(), self.x()) }
	fn wyzy(self) -> Self { Self::new(self.w(), self.y(), self.z(), self.y()) }
	fn wyzz(self) -> Self { Self::new(self.w(), self.y(), self.z(), self.z()) }
	fn wyzw(self) -> Self { Self::new(self.w(), self.y(), self.z(), self.w()) }
	fn wywx(self) -> Self { Self::new(self.w(), self.y(), self.w(), self.x()) }
	fn wywy(self) -> Self { Self::new(self.w(), self.y(), self.w(), self.y()) }
	fn wywz(self) -> Self { Self::new(self.w(), self.y(), self.w(), self.z()) }
	fn wyww(self) -> Self { Self::new(self.w(), self.y(), self.w(), self.w()) }
	fn wzxx(self) -> Self { Self::new(self.w(), self.z(), self.x(), self.x()) }
	fn wzxy(self) -> Self { Self::new(self.w(), self.z(), self.x(), self.y()) }
	fn wzxz(self) -> Self { Self::new(self.w(), self.z(), self.x(), self.z()) }
	fn wzxw(self) -> Self { Self::new(self.w(), self.z(), self.x(), self.w()) }
	fn wzyx(self) -> Self { Self::new(self.w(), self.z(), self.y(), self.x()) }
	fn wzyy(self) -> Self { Self::new(self.w(), self.z(), self.y(), self.y()) }
	fn wzyz(self) -> Self { Self::new(self.w(), self.z(), self.y(), self.z()) }
	fn wzyw(self) -> Self { Self::new(self.w(), self.z(), self.y(), self.w()) }
	fn wzzx(self) -> Self { Self::new(self.w(), self.z(), self.z(), self.x()) }
	fn wzzy(self) -> Self { Self::new(self.w(), self.z(), self.z(), self.y()) }
	fn wzzz(self) -> Self { Self::new(self.w(), self.z(), self.z(), self.z()) }
	fn wzzw(self) -> Self { Self::new(self.w(), self.z(), self.z(), self.w()) }
	fn wzwx(self) -> Self { Self::new(self.w(), self.z(), self.w(), self.x()) }
	fn wzwy(self) -> Self { Self::new(self.w(), self.z(), self.w(), self.y()) }
	fn wzwz(self) -> Self { Self::new(self.w(), self.z(), self.w(), self.z()) }
	fn wzww(self) -> Self { Self::new(self.w(), self.z(), self.w(), self.w()) }
	fn wwxx(self) -> Self { Self::new(self.w(), self.w(), self.x(), self.x()) }
	fn wwxy(self) -> Self { Self::new(self.w(), self.w(), self.x(), self.y()) }
	fn wwxz(self) -> Self { Self::new(self.w(), self.w(), self.x(), self.z()) }
	fn wwxw(self) -> Self { Self::new(self.w(), self.w(), self.x(), self.w()) }
	fn wwyx(self) -> Self { Self::new(self.w(), self.w(), self.y(), self.x()) }
	fn wwyy(self) -> Self { Self::new(self.w(), self.w(), self.y(), self.y()) }
	fn wwyz(self) -> Self { Self::new(self.w(), self.w(), self.y(), self.z()) }
	fn wwyw(self) -> Self { Self::new(self.w(), self.w(), self.y(), self.w()) }
	fn wwzx(self) -> Self { Self::new(self.w(), self.w(), self.z(), self.x()) }
	fn wwzy(self) -> Self { Self::new(self.w(), self.w(), self.z(), self.y()) }
	fn wwzz(self) -> Self { Self::new(self.w(), self.w(), self.z(), self.z()) }
	fn wwzw(self) -> Self { Self::new(self.w(), self.w(), self.z(), self.w()) }
	fn wwwx(self) -> Self { Self::new(self.w(), self.w(), self.w(), self.x()) }
	fn wwwy(self) -> Self { Self::new(self.w(), self.w(), self.w(), self.y()) }
	fn wwwz(self) -> Self { Self::new(self.w(), self.w(), self.w(), self.z()) }
	fn wwww(self) -> Self { Self::new(self.w(), self.w(), self.w(), self.w()) }
}

pub trait Vec4Swizzles3<T>: NewVec4<T> + Swizzle4Dto1<T> + Copy {
	type Vec3: NewVec3<T>;

	fn xxx(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.x(), self.x()) }
	fn xxy(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.x(), self.y()) }
	fn xxz(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.x(), self.z()) }
	fn xxw(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.x(), self.w()) }
	fn xyx(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.y(), self.x()) }
	fn xyy(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.y(), self.y()) }
	fn xyz(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.y(), self.z()) }
	fn xyw(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.y(), self.w()) }
	fn xzx(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.z(), self.x()) }
	fn xzy(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.z(), self.y()) }
	fn xzz(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.z(), self.z()) }
	fn xzw(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.z(), self.w()) }
	fn xwx(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.w(), self.x()) }
	fn xwy(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.w(), self.y()) }
	fn xwz(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.w(), self.z()) }
	fn xww(self) -> Self::Vec3 { Self::Vec3::new(self.x(), self.w(), self.w()) }
	fn yxx(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.x(), self.x()) }
	fn yxy(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.x(), self.y()) }
	fn yxz(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.x(), self.z()) }
	fn yxw(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.x(), self.w()) }
	fn yyx(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.y(), self.x()) }
	fn yyy(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.y(), self.y()) }
	fn yyz(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.y(), self.z()) }
	fn yyw(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.y(), self.w()) }
	fn yzx(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.z(), self.x()) }
	fn yzy(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.z(), self.y()) }
	fn yzz(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.z(), self.z()) }
	fn yzw(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.z(), self.w()) }
	fn ywx(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.w(), self.x()) }
	fn ywy(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.w(), self.y()) }
	fn ywz(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.w(), self.z()) }
	fn yww(self) -> Self::Vec3 { Self::Vec3::new(self.y(), self.w(), self.w()) }
	fn zxx(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.x(), self.x()) }
	fn zxy(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.x(), self.y()) }
	fn zxz(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.x(), self.z()) }
	fn zxw(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.x(), self.w()) }
	fn zyx(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.y(), self.x()) }
	fn zyy(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.y(), self.y()) }
	fn zyz(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.y(), self.z()) }
	fn zyw(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.y(), self.w()) }
	fn zzx(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.z(), self.x()) }
	fn zzy(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.z(), self.y()) }
	fn zzz(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.z(), self.z()) }
	fn zzw(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.z(), self.w()) }
	fn zwx(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.w(), self.x()) }
	fn zwy(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.w(), self.y()) }
	fn zwz(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.w(), self.z()) }
	fn zww(self) -> Self::Vec3 { Self::Vec3::new(self.z(), self.w(), self.w()) }
	fn wxx(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.x(), self.x()) }
	fn wxy(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.x(), self.y()) }
	fn wxz(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.x(), self.z()) }
	fn wxw(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.x(), self.w()) }
	fn wyx(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.y(), self.x()) }
	fn wyy(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.y(), self.y()) }
	fn wyz(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.y(), self.z()) }
	fn wyw(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.y(), self.w()) }
	fn wzx(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.z(), self.x()) }
	fn wzy(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.z(), self.y()) }
	fn wzz(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.z(), self.z()) }
	fn wzw(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.z(), self.w()) }
	fn wwx(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.w(), self.x()) }
	fn wwy(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.w(), self.y()) }
	fn wwz(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.w(), self.z()) }
	fn www(self) -> Self::Vec3 { Self::Vec3::new(self.w(), self.w(), self.w()) }
}

pub trait Vec4Swizzles2<T>: NewVec4<T> + Swizzle4Dto1<T> + Copy {
	type Vec2: NewVec2<T>;

	fn xx(self) -> Self::Vec2 { Self::Vec2::new(self.x(), self.x()) }
	fn xy(self) -> Self::Vec2 { Self::Vec2::new(self.x(), self.y()) }
	fn xz(self) -> Self::Vec2 { Self::Vec2::new(self.x(), self.z()) }
	fn xw(self) -> Self::Vec2 { Self::Vec2::new(self.x(), self.w()) }
	fn yx(self) -> Self::Vec2 { Self::Vec2::new(self.y(), self.x()) }
	fn yy(self) -> Self::Vec2 { Self::Vec2::new(self.y(), self.y()) }
	fn yz(self) -> Self::Vec2 { Self::Vec2::new(self.y(), self.z()) }
	fn yw(self) -> Self::Vec2 { Self::Vec2::new(self.y(), self.w()) }
	fn zx(self) -> Self::Vec2 { Self::Vec2::new(self.z(), self.x()) }
	fn zy(self) -> Self::Vec2 { Self::Vec2::new(self.z(), self.y()) }
	fn zz(self) -> Self::Vec2 { Self::Vec2::new(self.z(), self.z()) }
	fn zw(self) -> Self::Vec2 { Self::Vec2::new(self.z(), self.w()) }
	fn wx(self) -> Self::Vec2 { Self::Vec2::new(self.w(), self.x()) }
	fn wy(self) -> Self::Vec2 { Self::Vec2::new(self.w(), self.y()) }
	fn wz(self) -> Self::Vec2 { Self::Vec2::new(self.w(), self.z()) }
	fn ww(self) -> Self::Vec2 { Self::Vec2::new(self.w(), self.w()) }
}

pub trait Vec3Swizzles4<T>: NewVec3<T> + Swizzle3Dto1<T> + Copy {
	type Vec4: NewVec4<T>;

	fn xxxx(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.x(), self.x(), self.x()) }
	fn xxxy(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.x(), self.x(), self.y()) }
	fn xxxz(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.x(), self.x(), self.z()) }
	fn xxyx(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.x(), self.y(), self.x()) }
	fn xxyy(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.x(), self.y(), self.y()) }
	fn xxyz(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.x(), self.y(), self.z()) }
	fn xxzx(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.x(), self.z(), self.x()) }
	fn xxzy(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.x(), self.z(), self.y()) }
	fn xxzz(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.x(), self.z(), self.z()) }
	fn xyxx(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.y(), self.x(), self.x()) }
	fn xyxy(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.y(), self.x(), self.y()) }
	fn xyxz(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.y(), self.x(), self.z()) }
	fn xyyx(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.y(), self.y(), self.x()) }
	fn xyyy(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.y(), self.y(), self.y()) }
	fn xyyz(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.y(), self.y(), self.z()) }
	fn xyzx(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.y(), self.z(), self.x()) }
	fn xyzy(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.y(), self.z(), self.y()) }
	fn xyzz(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.y(), self.z(), self.z()) }
	fn xzxx(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.z(), self.x(), self.x()) }
	fn xzxy(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.z(), self.x(), self.y()) }
	fn xzxz(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.z(), self.x(), self.z()) }
	fn xzyx(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.z(), self.y(), self.x()) }
	fn xzyy(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.z(), self.y(), self.y()) }
	fn xzyz(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.z(), self.y(), self.z()) }
	fn xzzx(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.z(), self.z(), self.x()) }
	fn xzzy(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.z(), self.z(), self.y()) }
	fn xzzz(self) -> Self::Vec4 { Self::Vec4::new(self.x(), self.z(), self.z(), self.z()) }
	fn yxxx(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.x(), self.x(), self.x()) }
	fn yxxy(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.x(), self.x(), self.y()) }
	fn yxxz(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.x(), self.x(), self.z()) }
	fn yxyx(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.x(), self.y(), self.x()) }
	fn yxyy(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.x(), self.y(), self.y()) }
	fn yxyz(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.x(), self.y(), self.z()) }
	fn yxzx(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.x(), self.z(), self.x()) }
	fn yxzy(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.x(), self.z(), self.y()) }
	fn yxzz(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.x(), self.z(), self.z()) }
	fn yyxx(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.y(), self.x(), self.x()) }
	fn yyxy(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.y(), self.x(), self.y()) }
	fn yyxz(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.y(), self.x(), self.z()) }
	fn yyyx(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.y(), self.y(), self.x()) }
	fn yyyy(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.y(), self.y(), self.y()) }
	fn yyyz(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.y(), self.y(), self.z()) }
	fn yyzx(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.y(), self.z(), self.x()) }
	fn yyzy(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.y(), self.z(), self.y()) }
	fn yyzz(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.y(), self.z(), self.z()) }
	fn yzxx(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.z(), self.x(), self.x()) }
	fn yzxy(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.z(), self.x(), self.y()) }
	fn yzxz(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.z(), self.x(), self.z()) }
	fn yzyx(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.z(), self.y(), self.x()) }
	fn yzyy(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.z(), self.y(), self.y()) }
	fn yzyz(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.z(), self.y(), self.z()) }
	fn yzzx(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.z(), self.z(), self.x()) }
	fn yzzy(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.z(), self.z(), self.y()) }
	fn yzzz(self) -> Self::Vec4 { Self::Vec4::new(self.y(), self.z(), self.z(), self.z()) }
	fn zxxx(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.x(), self.x(), self.x()) }
	fn zxxy(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.x(), self.x(), self.y()) }
	fn zxxz(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.x(), self.x(), self.z()) }
	fn zxyx(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.x(), self.y(), self.x()) }
	fn zxyy(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.x(), self.y(), self.y()) }
	fn zxyz(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.x(), self.y(), self.z()) }
	fn zxzx(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.x(), self.z(), self.x()) }
	fn zxzy(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.x(), self.z(), self.y()) }
	fn zxzz(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.x(), self.z(), self.z()) }
	fn zyxx(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.y(), self.x(), self.x()) }
	fn zyxy(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.y(), self.x(), self.y()) }
	fn zyxz(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.y(), self.x(), self.z()) }
	fn zyyx(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.y(), self.y(), self.x()) }
	fn zyyy(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.y(), self.y(), self.y()) }
	fn zyyz(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.y(), self.y(), self.z()) }
	fn zyzx(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.y(), self.z(), self.x()) }
	fn zyzy(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.y(), self.z(), self.y()) }
	fn zyzz(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.y(), self.z(), self.z()) }
	fn zzxx(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.z(), self.x(), self.x()) }
	fn zzxy(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.z(), self.x(), self.y()) }
	fn zzxz(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.z(), self.x(), self.z()) }
	fn zzyx(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.z(), self.y(), self.x()) }
	fn zzyy(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.z(), self.y(), self.y()) }
	fn zzyz(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.z(), self.y(), self.z()) }
	fn zzzx(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.z(), self.z(), self.x()) }
	fn zzzy(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.z(), self.z(), self.y()) }
	fn zzzz(self) -> Self::Vec4 { Self::Vec4::new(self.z(), self.z(), self.z(), self.z()) }
}

pub trait Vec3Swizzles3<T>: NewVec3<T> + Swizzle3Dto1<T> + Copy {
	#[inline]
	fn xyz(self) -> Self { self }
	
	fn xxx(self) -> Self { Self::new(self.x(), self.x(), self.x()) }
	fn xxy(self) -> Self { Self::new(self.x(), self.x(), self.y()) }
	fn xxz(self) -> Self { Self::new(self.x(), self.x(), self.z()) }
	fn xyx(self) -> Self { Self::new(self.x(), self.y(), self.x()) }
	fn xyy(self) -> Self { Self::new(self.x(), self.y(), self.y()) }
	fn xzx(self) -> Self { Self::new(self.x(), self.z(), self.x()) }
	fn xzy(self) -> Self { Self::new(self.x(), self.z(), self.y()) }
	fn xzz(self) -> Self { Self::new(self.x(), self.z(), self.z()) }
	fn yxx(self) -> Self { Self::new(self.y(), self.x(), self.x()) }
	fn yxy(self) -> Self { Self::new(self.y(), self.x(), self.y()) }
	fn yxz(self) -> Self { Self::new(self.y(), self.x(), self.z()) }
	fn yyx(self) -> Self { Self::new(self.y(), self.y(), self.x()) }
	fn yyy(self) -> Self { Self::new(self.y(), self.y(), self.y()) }
	fn yyz(self) -> Self { Self::new(self.y(), self.y(), self.z()) }
	fn yzx(self) -> Self { Self::new(self.y(), self.z(), self.x()) }
	fn yzy(self) -> Self { Self::new(self.y(), self.z(), self.y()) }
	fn yzz(self) -> Self { Self::new(self.y(), self.z(), self.z()) }
	fn zxx(self) -> Self { Self::new(self.z(), self.x(), self.x()) }
	fn zxy(self) -> Self { Self::new(self.z(), self.x(), self.y()) }
	fn zxz(self) -> Self { Self::new(self.z(), self.x(), self.z()) }
	fn zyx(self) -> Self { Self::new(self.z(), self.y(), self.x()) }
	fn zyy(self) -> Self { Self::new(self.z(), self.y(), self.y()) }
	fn zyz(self) -> Self { Self::new(self.z(), self.y(), self.z()) }
	fn zzx(self) -> Self { Self::new(self.z(), self.z(), self.x()) }
	fn zzy(self) -> Self { Self::new(self.z(), self.z(), self.y()) }
	fn zzz(self) -> Self { Self::new(self.z(), self.z(), self.z()) }
}

pub trait Vec3Swizzles2<T>: NewVec3<T> + Swizzle3Dto1<T> + Copy {
	type Vec2: NewVec2<T>;

	fn xx(self) -> Self::Vec2 { Self::Vec2::new(self.x(), self.x()) }
	fn xy(self) -> Self::Vec2 { Self::Vec2::new(self.x(), self.y()) }
	fn xz(self) -> Self::Vec2 { Self::Vec2::new(self.x(), self.z()) }
	fn yx(self) -> Self::Vec2 { Self::Vec2::new(self.y(), self.x()) }
	fn yy(self) -> Self::Vec2 { Self::Vec2::new(self.y(), self.y()) }
	fn yz(self) -> Self::Vec2 { Self::Vec2::new(self.y(), self.z()) }
	fn zx(self) -> Self::Vec2 { Self::Vec2::new(self.z(), self.x()) }
	fn zy(self) -> Self::Vec2 { Self::Vec2::new(self.z(), self.y()) }
	fn zz(self) -> Self::Vec2 { Self::Vec2::new(self.z(), self.z()) }
}

pub trait Vec2Swizzles4<T>: Swizzle2Dto1<T> + NewVec2<T> + Copy {
    type Vec4;

    fn xxxx(self) -> Self::Vec4;
    fn xxxy(self) -> Self::Vec4;
    fn xxyx(self) -> Self::Vec4;
    fn xxyy(self) -> Self::Vec4;
    fn xyxx(self) -> Self::Vec4;
    fn xyxy(self) -> Self::Vec4;
    fn xyyx(self) -> Self::Vec4;
    fn xyyy(self) -> Self::Vec4;
    fn yxxx(self) -> Self::Vec4;
    fn yxxy(self) -> Self::Vec4;
    fn yxyx(self) -> Self::Vec4;
    fn yxyy(self) -> Self::Vec4;
    fn yyxx(self) -> Self::Vec4;
    fn yyxy(self) -> Self::Vec4;
    fn yyyx(self) -> Self::Vec4;
    fn yyyy(self) -> Self::Vec4;
}

pub trait Vec2Swizzles3<T>: Swizzle2Dto1<T> + NewVec2<T> + Copy {
	type Vec3: NewVec3<T>;
    
    fn xxx(self) -> Self::Vec3;
    fn xxy(self) -> Self::Vec3;
    fn xyx(self) -> Self::Vec3;
    fn xyy(self) -> Self::Vec3;
    fn yxx(self) -> Self::Vec3;
    fn yxy(self) -> Self::Vec3;
    fn yyx(self) -> Self::Vec3;
    fn yyy(self) -> Self::Vec3;
}

pub trait Vec2Swizzles2<T>: Swizzle2Dto1<T> + NewVec2<T> + Copy {
    #[inline]
    fn xy(self) -> Self { self }

    fn xx(self) -> Self;
    fn yx(self) -> Self;
    fn yy(self) -> Self;
}