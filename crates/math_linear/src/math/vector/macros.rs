/// Derives basic API for 3-dimensional `$Name` vector.
/// Note: [`std::ops::Neg`] is not derived due to unsigned type (such as [`u32`]).
macro_rules! make_3_component_vector {
    () => {};

    (
        $(#[$macros:meta])*
        $vis:vis $Name:ident = ($x:ident, $y:ident, $z:ident): $Type:ty
    ) => {
        $(#[$macros])*
        #[repr(C)]
        #[doc = concat!(
            "Three dimencional mathematical vector ",
            stringify!($Name),
            " of type ",
            stringify!($Type)
        )]
        #[cfg_attr(feature = "byte_muck", derive(bytemuck::Pod, bytemuck::Zeroable))]
        #[derive(Clone, Copy, Default, PartialEq, Debug)]
        $vis struct $Name {
            $vis $x: $Type,
            $vis $y: $Type,
            $vis $z: $Type,
        }

        impl $Name {
            $vis const ZERO: Self = Self::zero();
            $vis const ONE:  Self = Self::ones();

            /// Constructs new vector
            $vis const fn new($x: $Type, $y: $Type, $z: $Type) -> Self {
                Self { $x, $y, $z }
            }

            $vis const fn $x(&self) -> $Type { self.$x }
            $vis const fn $y(&self) -> $Type { self.$y }
            $vis const fn $z(&self) -> $Type { self.$z }

            /// Constructs vector from one number.
            $vis const fn all(all: $Type) -> Self {
                Self::new(all, all, all)
            }

            /// Constructs zero vector.
            $vis const fn zero() -> Self {
                Self::all(0 as $Type)
            }

            /// Constructs unit vector.
            $vis const fn ones() -> Self {
                Self::all(1 as $Type)
            }

            /// Calculates dot product.
            $vis fn dot(self, other: Self) -> $Type {
                self.$x * other.$x + self.$y * other.$y + self.$z * other.$z
            }

            /// Calculates cross product.
            $vis fn cross(self, other: Self) -> Self {
                Self::new(
                    self.$y * other.$z - self.$z * other.$y,
                    self.$z * other.$x - self.$x * other.$z,
                    self.$x * other.$y - self.$y * other.$x,
                )
            }

            /// Normalizes the vector. Result is always [`Float3`] due to integer vectors can
            /// not be normalized.
            $vis fn normalized(self) -> Float3 {
                let len = self.len();
                if len == 0.0 {
                    Float3::zero()
                } else {
                    Float3::from(self) / len
                }
            }

            /// Computes squared length.
            $vis fn sqr(self) -> $Type {
                self.dot(self)
            }

            /// Computes vector length.
            $vis fn len(self) -> f32 {
                (self.sqr() as f32).sqrt()
            }

            /// Gives a vector with `self` direction and `new_len` length.
            $vis fn with_len(self, new_len: f32) -> Float3 {
                self.normalized() * new_len
            }

            /// Represents [`$Name`] as an array.
            $vis const fn as_array(self) -> [$Type; 3] {
                [self.$x(), self.$y(), self.$z()]
            }

            /// Represents [`$Name`] as a tuple.
            $vis const fn as_tuple(self) -> ($Type, $Type, $Type) {
                (self.$x(), self.$y(), self.$z())
            }

            /// Calculates reminder of the division `self` by `other`.
            $vis fn rem_euclid(self, other: Self) -> Self {
                (self % other + other) % other
            }

            /// Calculates result of the division `self` by `other`.
            $vis fn div_euclid(self, other: Self) -> Self {
                Self::new(
                    self.$x.div_euclid(other.$x),
                    self.$y.div_euclid(other.$y),
                    self.$z.div_euclid(other.$z),
                )
            }
        } // impl $Name

        impl std::fmt::Display for $Name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({x}, {y}, {z})", x = self.$x, y = self.$y, z = self.$z)
            }
        }

        impl From<[$Type; 3]> for $Name {
            fn from([$x, $y, $z]: [$Type; 3]) -> Self {
                Self::new($x, $y, $z)
            }
        }

        impl From<($Type, $Type, $Type)> for $Name {
            fn from(($x, $y, $z): ($Type, $Type, $Type)) -> Self {
                Self::new($x, $y, $z)
            }
        }

        impl std::ops::Sub for $Name {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self::new(self.$x - other.$x, self.$y - other.$y, self.$z - other.$z)
            }
        }

        impl std::ops::SubAssign for $Name {
            fn sub_assign(&mut self, other: Self) {
                self.$x -= other.$x;
                self.$y -= other.$y;
                self.$z -= other.$z;
            }
        }

        impl std::ops::Add for $Name {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self::new(self.$x + other.$x, self.$y + other.$y, self.$z + other.$z)
            }
        }

        impl std::ops::AddAssign for $Name {
            fn add_assign(&mut self, other: Self) {
                self.$x += other.$x;
                self.$y += other.$y;
                self.$z += other.$z;
            }
        }

        impl std::ops::Mul<$Type> for $Name {
            type Output = Self;
            fn mul(self, k: $Type) -> Self {
                Self::new(self.$x * k , self.$y * k, self.$z * k)
            }
        }

        impl std::ops::Mul<$Name> for $Type {
            type Output = $Name;
            fn mul(self, vec: $Name) -> Self::Output {
                vec * self
            }
        }

        impl std::ops::Mul for $Name {
            type Output = Self;
            fn mul(self, p: Self) -> Self {
                Self::new(self.$x * p.$x, self.$y * p.$y, self.$z * p.$z)
            }
        }

        impl std::ops::MulAssign<$Type> for $Name {
            fn mul_assign(&mut self, k: $Type) {
                self.$x *= k;
                self.$y *= k;
                self.$z *= k;
            }
        }

        impl std::ops::MulAssign for $Name {
            fn mul_assign(&mut self, p: Self) {
                self.$x *= p.$x;
                self.$y *= p.$y;
                self.$z *= p.$z;
            }
        }

        impl std::ops::Div<$Type> for $Name {
            type Output = Self;
            fn div(self, k: $Type) -> Self {
                Self::new(self.$x / k, self.$y / k, self.$z / k)
            }
        }

        impl std::ops::Div for $Name {
            type Output = Self;
            fn div(self, k: Self) -> Self {
                Self::new(self.$x / k.$x, self.$y / k.$x, self.$z / k.$x)
            }
        }

        impl std::ops::DivAssign<$Type> for $Name {
            fn div_assign(&mut self, k: $Type) {
                self.$x /= k;
                self.$y /= k;
                self.$z /= k;
            }
        }

        impl std::ops::DivAssign for $Name {
            fn div_assign(&mut self, k: Self) {
                self.$x /= k.$x;
                self.$y /= k.$y;
                self.$z /= k.$z;
            }
        }

        impl std::ops::RemAssign for $Name {
            fn rem_assign(&mut self, rhs: Self) {
                self.$x %= rhs.$x();
                self.$y %= rhs.$y();
                self.$z %= rhs.$z();
            }
        }

        impl std::ops::Rem for $Name {
            type Output = Self;
            fn rem(mut self, rhs: Self) -> Self::Output {
                self %= rhs;
                return self
            }
        }

        impl std::ops::RemAssign<$Type> for $Name {
            fn rem_assign(&mut self, rhs: $Type) {
                self.$x %= rhs;
                self.$y %= rhs;
                self.$z %= rhs;
            }
        }

        impl std::ops::Rem<$Type> for $Name {
            type Output = Self;
            fn rem(mut self, rhs: $Type) -> Self::Output {
                self %= rhs;
                return self
            }
        }

        impl std::ops::Rem<$Name> for $Type {
            type Output = $Name;
            fn rem(self, vec: $Name) -> Self::Output {
                $Name::all(self) % vec
            }
        }

        impl PartialOrd for $Name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                use std::cmp::Ordering;

                if self == other { return Some(Ordering::Equal) }

                if self.$x < other.$x && self.$y < other.$y && self.$z < other.$z {
                    return Some(Ordering::Less)
                }

                if self.$x > other.$x && self.$y > other.$y && self.$z > other.$z {
                    return Some(Ordering::Greater)
                }

                None
            }
        }
    };

    ($(
        $(#[$macros:meta])*
        $vis:vis $Name:ident = ($x:ident, $y:ident, $z:ident): $Type:ty
    );+;) => {
        $(make_3_component_vector! {
            $(#[$macros])*
            $vis $Name = ($x, $y, $z): $Type
        })+
    }
}

macro_rules! derive_froms_3_component {
    ($Type1:ty { $x1:ident, $y1:ident, $z1:ident : $ElemType1:ty } <->
     $Type2:ty { $x2:ident, $y2:ident, $z2:ident : $ElemType2:ty }) =>
    {
        impl From<$Type2> for $Type1 {
            fn from(rhs: $Type2) -> Self {
                Self::new(rhs.$x2() as $ElemType1, rhs.$y2() as $ElemType1, rhs.$z2() as $ElemType1)
            }
        }

        impl From<$Type1> for $Type2 {
            fn from(rhs: $Type1) -> Self {
                Self::new(rhs.$x1() as $ElemType2, rhs.$y1() as $ElemType2, rhs.$z1() as $ElemType2)
            }
        }
    };

    ($($Type1:ty { $x1:ident, $y1:ident, $z1:ident : $ElemType1:ty } <->
       $Type2:ty { $x2:ident, $y2:ident, $z2:ident : $ElemType2:ty });+;) =>
    {
        $(derive_froms_3_component! { $Type1 { $x1, $y1, $z1: $ElemType1 } <-> $Type2 { $x2, $y2, $z2: $ElemType2 } })+
    }
}

/// Derives basic API for 2-dimensional `$Name` vector.
/// Note: [`std::ops::Neg`] is not derived due to unsigned type (such as [`u32`]).
macro_rules! make_2_component_vector {
    () => { };

    (
        $(#[$macros:meta])*
        $vis:vis $Name:ident = ($x:ident, $y:ident): $Type:ty
    ) => {
        $(#[$macros])*
        #[repr(C)]
        #[cfg_attr(feature = "byte_muck", derive(bytemuck::Pod, bytemuck::Zeroable))]
        #[derive(Clone, Copy, Default, PartialEq, Debug)]
        $vis struct $Name {
            $vis $x: $Type,
            $vis $y: $Type,
        }

        impl $Name {
            $vis const ZERO: Self = Self::zero();
            $vis const ONE:  Self = Self::ones();

            /// Constructs new vector
            $vis const fn new($x: $Type, $y: $Type) -> Self {
                Self { $x, $y }
            }

            /// Creates new vector from `impl Into<$Type>` traits.
            $vis fn make($x: impl Into<$Type>, $y: impl Into<$Type>) -> Self {
                Self::new($x.into(), $y.into())
            }

            $vis const fn $x(&self) -> $Type { self.$x }
            $vis const fn $y(&self) -> $Type { self.$y }

            /// Constructs vector from one number.
            $vis const fn all(all: $Type) -> Self {
                Self::new(all, all)
            }

            /// Constructs zero vector.
            $vis const fn zero() -> Self {
                Self::all(0 as $Type)
            }

            /// Constructs unit vector.
            $vis const fn ones() -> Self {
                Self::all(1 as $Type)
            }

            /// Calculates dot product.
            $vis fn dot(self, other: Self) -> $Type {
                self.$x * other.$x + self.$y * other.$y
            }

            /// Calculates pseudoscalar product.
            $vis fn cross(self, other: Self) -> $Type {
                self.$x * other.$y - self.$y * other.$x
            }

            /// Computes squares length.
            $vis fn sqr(self) -> $Type {
                self.dot(self)
            }

            /// Computes vector length.
            $vis fn len(self) -> f32 {
                (self.sqr() as f32).sqrt()
            }

            /// Gives a vector with `self` direction and `new_len` length.
            $vis fn with_len(self, new_len: f32) -> Float2 {
                self.normalized() * new_len
            }

            /// Gives normalized vector. Result is always [`Float2`] due to
            /// integer vectors can not be normalized.
            $vis fn normalized(self) -> Float2 {
                let len = self.len();
                if len == 0.0 {
                    Float2::zero()
                } else {
                    Float2::from(self) / len
                }
            }

            /// Represents [`$Name`] as an array.
            $vis const fn as_array(self) -> [$Type; 2] {
                [self.$x(), self.$y()]
            }

            /// Represents [`$Name`] as a tuple.
            $vis const fn as_tuple(self) -> ($Type, $Type) {
                (self.$x(), self.$y())
            }

            /// Calculates reminder of the division `self` by `other`.
            $vis fn rem_euclid(self, other: Self) -> Self {
                (self % other + other) % other
            }

            /// Calculates result of the division `self` by `other`.
            $vis fn div_euclid(self, other: Self) -> Self {
                Self::new(
                    self.$x.div_euclid(other.$x),
                    self.$y.div_euclid(other.$y),
                )
            }
        } // impl $Name

        impl std::fmt::Display for $Name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({x}, {y})", x = self.$x, y = self.$y)
            }
        }

        impl From<[$Type; 2]> for $Name {
            fn from([$x, $y]: [$Type; 2]) -> Self {
                Self::new($x, $y)
            }
        }

        impl From<($Type, $Type)> for $Name {
            fn from(($x, $y): ($Type, $Type)) -> Self {
                Self::new($x, $y)
            }
        }

        impl std::ops::Sub for $Name {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self::new(self.$x - other.$x, self.$y - other.$y)
            }
        }

        impl std::ops::SubAssign for $Name {
            fn sub_assign(&mut self, other: Self) {
                self.$x -= other.$x;
                self.$y -= other.$y;
            }
        }

        impl std::ops::Add for $Name {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self::new(self.$x + other.$x, self.$y + other.$y)
            }
        }

        impl std::ops::AddAssign for $Name {
            fn add_assign(&mut self, other: Self) {
                self.$x += other.$x;
                self.$y += other.$y;
            }
        }

        impl std::ops::Mul<$Type> for $Name {
            type Output = Self;
            fn mul(self, k: $Type) -> Self {
                Self::new(self.$x * k , self.$y * k)
            }
        }

        impl std::ops::Mul<$Name> for $Type {
            type Output = $Name;
            fn mul(self, vec: $Name) -> $Name {
                $Name::new(vec.$x * self, vec.$y * self)
            }
        }

        impl std::ops::Mul for $Name {
            type Output = Self;
            fn mul(self, p: Self) -> Self {
                Self::new(self.$x * p.$x, self.$y * p.$y)
            }
        }

        impl std::ops::MulAssign<$Type> for $Name {
            fn mul_assign(&mut self, k: $Type) {
                self.$x *= k;
                self.$y *= k;
            }
        }

        impl std::ops::MulAssign for $Name {
            fn mul_assign(&mut self, p: Self) {
                self.$x *= p.$x;
                self.$y *= p.$y;
            }
        }

        impl std::ops::Div<$Type> for $Name {
            type Output = Self;
            fn div(self, k: $Type) -> Self {
                Self::new(self.$x / k, self.$y / k)
            }
        }

        impl std::ops::Div for $Name {
            type Output = Self;
            fn div(self, k: Self) -> Self {
                Self::new(self.$x / k.$x, self.$y / k.$x)
            }
        }

        impl std::ops::DivAssign<$Type> for $Name {
            fn div_assign(&mut self, k: $Type) {
                self.$x /= k;
                self.$y /= k;
            }
        }

        impl std::ops::DivAssign for $Name {
            fn div_assign(&mut self, k: Self) {
                self.$x /= k.$x;
                self.$y /= k.$y;
            }
        }

        impl std::ops::RemAssign for $Name {
            fn rem_assign(&mut self, rhs: Self) {
                self.$x %= rhs.$x();
                self.$y %= rhs.$y();
            }
        }

        impl std::ops::Rem for $Name {
            type Output = Self;
            fn rem(mut self, rhs: Self) -> Self::Output {
                self %= rhs;
                return self
            }
        }

        impl std::ops::RemAssign<$Type> for $Name {
            fn rem_assign(&mut self, rhs: $Type) {
                self.$x %= rhs;
                self.$y %= rhs;
            }
        }

        impl std::ops::Rem<$Type> for $Name {
            type Output = Self;
            fn rem(mut self, rhs: $Type) -> Self::Output {
                self %= rhs;
                return self
            }
        }

        impl PartialOrd for $Name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                use std::cmp::Ordering;

                if self == other { return Some(Ordering::Equal) }

                if self.$x < other.$x && self.$y < other.$y {
                    return Some(Ordering::Less)
                }

                if self.$x > other.$x && self.$y > other.$y {
                    return Some(Ordering::Greater)
                }

                None
            }
        }
    };

    ($(
        $(#[$macros:meta])*
        $vis:vis $Name:ident = ($x:ident, $y:ident): $Type:ty
    );+;) => {
        $(make_2_component_vector! {
            $(#[$macros])*
            $vis $Name = ($x, $y): $Type
        })+
    }
}

macro_rules! _impl_eq {
    () => { };

    ($($VecName:ident,)+) => {
        $(impl Eq for $VecName {})+
    };
}

macro_rules! impl_neg {
    () => { };

    ( $VecName:ident = ($($p:ident),+) ) => {
        impl std::ops::Neg for $VecName {
            type Output = Self;
            fn neg(self) -> Self::Output {
                Self::new($(-self.$p),+)
            }
        }

        impl $VecName {
            pub fn abs(self) -> Self {
                Self::new($(self.$p.abs()),+)
            }
        }
    }
}

macro_rules! derive_froms_2_component {
    ($Type1:ty { $x1:ident, $y1:ident : $ElemType1:ty } <->
     $Type2:ty { $x2:ident, $y2:ident : $ElemType2:ty }) =>
    {
        impl From<$Type2> for $Type1 {
            fn from(rhs: $Type2) -> Self {
                Self::new(rhs.$x2() as $ElemType1, rhs.$y2() as $ElemType1)
            }
        }

        impl From<$Type1> for $Type2 {
            fn from(rhs: $Type1) -> Self {
                Self::new(rhs.$x1() as $ElemType2, rhs.$y1() as $ElemType2)
            }
        }
    };

    ($($Type1:ty { $x1:ident, $y1:ident : $ElemType1:ty } <->
       $Type2:ty { $x2:ident, $y2:ident : $ElemType2:ty });+;) =>
    {
        $(derive_froms_2_component! { $Type1 { $x1, $y1: $ElemType1 } <-> $Type2 { $x2, $y2: $ElemType2 } })+
    }
}

macro_rules! generate_3d_swizzles {
    ($vis:vis $Name:ident -> $Lower:ident = ($x:ident, $y:ident, $z:ident): $Type:ty) => {
        generate_3d_swizzles_only_3d! { $vis $Name = ($x, $y, $z): $Type }
        generate_3d_swizzles_only_2d! { $vis $Name -> $Lower = ($x, $y, $z): $Type }
    };

    ($($vis:vis $Name:ident -> $Lower:ident = ($x:ident, $y:ident, $z:ident): $Type:ty);+;) => {
        $(generate_3d_swizzles! { $vis $Name -> $Lower = ($x, $y, $z): $Type })+
    };
}

macro_rules! generate_3d_swizzles_only_3d {
    ($vis:vis $Name:ident = ($x:ident, $y:ident, $z:ident): $Type:ty) => {
        impl $Name {
            $vis fn xxx(self) -> Self { Self::new(self.$x, self.$x, self.$x) }
            $vis fn xxy(self) -> Self { Self::new(self.$x, self.$x, self.$y) }
            $vis fn xxz(self) -> Self { Self::new(self.$x, self.$x, self.$z) }
            $vis fn xyx(self) -> Self { Self::new(self.$x, self.$y, self.$x) }
            $vis fn xyy(self) -> Self { Self::new(self.$x, self.$y, self.$y) }
            $vis fn xyz(self) -> Self { Self::new(self.$x, self.$y, self.$z) }
            $vis fn xzx(self) -> Self { Self::new(self.$x, self.$z, self.$x) }
            $vis fn xzy(self) -> Self { Self::new(self.$x, self.$z, self.$y) }
            $vis fn xzz(self) -> Self { Self::new(self.$x, self.$z, self.$z) }
            $vis fn yxx(self) -> Self { Self::new(self.$y, self.$x, self.$x) }
            $vis fn yxy(self) -> Self { Self::new(self.$y, self.$x, self.$y) }
            $vis fn yxz(self) -> Self { Self::new(self.$y, self.$x, self.$z) }
            $vis fn yyx(self) -> Self { Self::new(self.$y, self.$y, self.$x) }
            $vis fn yyy(self) -> Self { Self::new(self.$y, self.$y, self.$y) }
            $vis fn yyz(self) -> Self { Self::new(self.$y, self.$y, self.$z) }
            $vis fn yzx(self) -> Self { Self::new(self.$y, self.$z, self.$x) }
            $vis fn yzy(self) -> Self { Self::new(self.$y, self.$z, self.$y) }
            $vis fn yzz(self) -> Self { Self::new(self.$y, self.$z, self.$z) }
            $vis fn zxx(self) -> Self { Self::new(self.$z, self.$x, self.$x) }
            $vis fn zxy(self) -> Self { Self::new(self.$z, self.$x, self.$y) }
            $vis fn zxz(self) -> Self { Self::new(self.$z, self.$x, self.$z) }
            $vis fn zyx(self) -> Self { Self::new(self.$z, self.$y, self.$x) }
            $vis fn zyy(self) -> Self { Self::new(self.$z, self.$y, self.$y) }
            $vis fn zyz(self) -> Self { Self::new(self.$z, self.$y, self.$z) }
            $vis fn zzx(self) -> Self { Self::new(self.$z, self.$z, self.$x) }
            $vis fn zzy(self) -> Self { Self::new(self.$z, self.$z, self.$y) }
            $vis fn zzz(self) -> Self { Self::new(self.$z, self.$z, self.$z) }
        }
    };

    ($($vis:vis $Name:ident = ($x:ident, $y:ident, $z:ident): $Type:ty);+;) => {
        $(generate_3d_swizzles_only_3d! { $vis $Name = ($x, $y, $z): $Type })+
    };
}

macro_rules! generate_3d_swizzles_only_2d {
    ($vis:vis $Name:ident -> $Lower:ident = ($x:ident, $y:ident, $z:ident): $Type:ty) => {
        impl $Name {
            $vis fn xx(self) -> $Lower { $Lower::new(self.$x, self.$x) }
            $vis fn xy(self) -> $Lower { $Lower::new(self.$x, self.$y) }
            $vis fn xz(self) -> $Lower { $Lower::new(self.$x, self.$z) }
            $vis fn yx(self) -> $Lower { $Lower::new(self.$y, self.$x) }
            $vis fn yy(self) -> $Lower { $Lower::new(self.$y, self.$y) }
            $vis fn yz(self) -> $Lower { $Lower::new(self.$y, self.$z) }
            $vis fn zx(self) -> $Lower { $Lower::new(self.$z, self.$x) }
            $vis fn zy(self) -> $Lower { $Lower::new(self.$z, self.$y) }
            $vis fn zz(self) -> $Lower { $Lower::new(self.$z, self.$z) }
        }
    };

    ($($vis:vis $Name:ident -> $Lower:ident = ($x:ident, $y:ident, $z:ident): $Type:ty);+;) => {
        $(generate_3d_swizzles_only_2d! { $vis $Name -> $Lower = ($x, $y, $z): $Type })+
    };
}

#[macro_export]
macro_rules! vector_macro {
    ($name:ident : $Vec3:ty, $Vec2:ty : $Type:ty) => {
        ::paste::paste! {
            #[macro_export]
            macro_rules! [< _ $name >] {
                () => {()};

                ($x:expr) => {{
                    {$x} as $Type
                }};

                ($x:expr, $y:expr) => {{
                    use $crate::prelude::*;
                    <$Vec2>::new({$x} as $Type, {$y} as $Type)
                }};

                ($x:expr, $y:expr, $z:expr) => {{
                    use $crate::prelude::*;
                    <$Vec3>::new({$x} as $Type, {$y} as $Type, {$z} as $Type)
                }};
            }

            #[doc(hidden)]
            pub use [< _ $name >] as $name;
        }

        impl From<($Vec2, $Type)> for $Vec3 {
            fn from((lhs, rhs): ($Vec2, $Type)) -> Self {
                Self::new(lhs.x, lhs.y, rhs)
            }
        }

        impl From<($Type, $Vec2)> for $Vec3 {
            fn from((lhs, rhs): ($Type, $Vec2)) -> Self {
                Self::new(lhs, rhs.x, rhs.y)
            }
        }
    };
}

pub(crate) use vector_macro;

use super::{Float2, Float3, Int2, Int3, UInt2, UInt3, USize2, USize3};

vector_macro! { vecf: Float3, Float2: f32 }
vector_macro! { veci: Int3,   Int2:   i32 }
vector_macro! { vecu: UInt3,  UInt2:  u32 }
vector_macro! { vecs: USize3, USize2: usize }

#[test]
fn t() {
    let a = vecf!(1, 2);
    let b = 3.0;
    let c = Float3::from((a, b));

    assert_eq!(c, vecf!(1, 2, 3));
}
