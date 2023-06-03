pub mod window;
pub mod graphics;
pub mod user_io;
pub mod terrain;
pub mod time;
pub mod profiler;
pub mod reinterpreter;
pub mod saves;
pub mod concurrency;
pub mod runtime;
pub mod werror;
pub mod cfg;
pub mod logger;
pub mod assets;
pub mod transform;
pub mod camera;
pub mod str_view;
pub mod wrapper;
pub mod macros;
pub mod failure;



use { crate::prelude::*, winit::dpi::{Pixel, PhysicalSize} };

pub use crate::{module_constructor, module_destructor};



/// Returns the "default value" for a type.
/// Default values are often some kind of initial value,
/// identity value, or anything else that may make sense as a default
pub fn default<T: Default>() -> T {
    T::default()
}

/// Returns the "default value" for a type, but in const context.
/// Default values are often some kind of initial value,
/// identity value, or anything else that may make sense as a default
pub const fn const_default<T: ConstDefault>() -> T {
    T::DEFAULT
}



pub trait ToPhisicalSize<P: Pixel> {
    fn to_phisical_size(&self) -> PhysicalSize<P>;
}
assert_obj_safe!(ToPhisicalSize<u32>);

impl ToPhisicalSize<u32> for UInt2 {
    fn to_phisical_size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.x, self.y)
    }
}



pub trait ToVec2 {
    fn to_vec2(&self) -> UInt2;
}

impl ToVec2 for PhysicalSize<u32> {
    fn to_vec2(&self) -> UInt2 {
        UInt2::new(self.width, self.height)
    }
}



macro impl_volume($VecType:ty, $ElemType:ty) {
    impl Volume<$ElemType> for $VecType {
        fn volume(&self) -> $ElemType {
            self.x * self.y * self.z
        }
    }
}

pub trait Volume<T> {
    fn volume(&self) -> T;
}
assert_obj_safe!(Volume<vec3>);

impl_volume!(Byte3, i8);
impl_volume!(UByte3, u8);
impl_volume!(Short3, i16);
impl_volume!(UShort3, u16);
impl_volume!(Int3, i32);
impl_volume!(UInt3, u32);
impl_volume!(Long3, i64);
impl_volume!(ULong3, u64);
impl_volume!(Large3, i128);
impl_volume!(ULarge3, u128);
impl_volume!(ISize3, isize);
impl_volume!(USize3, usize);
impl_volume!(vec3, f32);
impl_volume!(Double3, f64);



macro impl_nums_const_default($($Int:ty),* $(,)?) {
    $(
        impl ConstDefault for $Int {
            const DEFAULT: Self = 0 as Self;
        }
    )*
}



pub trait ConstDefault {
    const DEFAULT: Self;
}

impl_nums_const_default! { i8, u8, i16, u16, i32, u32, f32, i64, u64, isize, usize }

impl ConstDefault for bool {
    const DEFAULT: Self = false;
}

impl ConstDefault for String {
    const DEFAULT: Self = Self::new();
}

impl<T> ConstDefault for Option<T> {
    const DEFAULT: Self = Self::None;
}

impl<T> ConstDefault for Vec<T> {
    const DEFAULT: Self = vec![];
}

impl<T, const N: usize> ConstDefault for SmallVec<[T; N]> {
    const DEFAULT: Self = Self::new_const();
}



#[macro_export]
macro_rules! module_constructor {
    ($($content:tt)*) => {
        #[ctor::ctor]
        fn __module_constructor_function() {
            $($content)*
        }
    };
}

#[macro_export]
macro_rules! module_destructor {
    ($($content:tt)*) => {
        #[ctor::dtor]
        fn __module_destructor_function() {
            $($content)*
        }
    };
}