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
pub mod const_default;



use { crate::prelude::*, winit::dpi::{Pixel, PhysicalSize} };



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



pub trait Volume<T> {
    fn volume(&self) -> T;
}
assert_obj_safe!(Volume<vec3>);

macro impl_volume($VecType:ty, $ElemType:ty) {
    impl Volume<$ElemType> for $VecType {
        fn volume(&self) -> $ElemType {
            self.x * self.y * self.z
        }
    }
}

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



pub macro module_constructor($($content:tt)*) {
    #[ctor::ctor]
    fn __module_constructor_function() {
        $($content)*
    }
}

pub macro module_destructor($($content:tt)*) {
    #[ctor::dtor]
    fn __module_destructor_function() {
        $($content)*
    }
}