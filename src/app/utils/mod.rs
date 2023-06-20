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
pub mod physics;
pub mod geometry;
pub mod iterator;



use { crate::prelude::*, winit::dpi::{Pixel, PhysicalSize, PhysicalPosition} };



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



#[const_trait]
pub trait ToPhisicalSize<P: Pixel> {
    fn to_physical_size(&self) -> PhysicalSize<P>;
}
assert_obj_safe!(ToPhisicalSize<u32>);

impl const ToPhisicalSize<u32> for UInt2 {
    fn to_physical_size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.x, self.y)
    }
}



#[const_trait]
pub trait ToPhisicalPosition<P: Pixel> {
    fn to_physical_position(&self) -> PhysicalPosition<P>;
}
assert_obj_safe!(ToPhisicalPosition<u32>);

impl const ToPhisicalPosition<u32> for UInt2 {
    fn to_physical_position(&self) -> PhysicalPosition<u32> {
        PhysicalPosition::new(self.x, self.y)
    }
}



#[const_trait]
pub trait ToVec2 {
    fn to_vec2(&self) -> UInt2;
}

impl const ToVec2 for PhysicalSize<u32> {
    fn to_vec2(&self) -> UInt2 {
        UInt2::new(self.width, self.height)
    }
}



#[const_trait]
pub trait Volume<T> {
    fn volume(&self) -> T;
}
assert_obj_safe!(Volume<vec3>);

macro impl_volume($($VecType:ty : $ElemType:ty),* $(,)?) {
    $(
        impl const Volume<$ElemType> for $VecType {
            fn volume(&self) -> $ElemType {
                self.x * self.y * self.z
            }
        }
    )*
}

impl_volume! {
    Byte3: i8, UByte3: u8, Short3: i16, UShort3: u16, Int3: i32, UInt3: u32, Long3: i64,
    ULong3: u64, Large3: i128, ULarge3: u128, ISize3: isize, USize3: usize, vec3: f32, Double3: f64,
}



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



pub const fn min(lhs: f32, rhs: f32) -> f32 {
    if lhs <= rhs { lhs } else { rhs }
}

pub const fn max(lhs: f32, rhs: f32) -> f32 {
    if lhs >= rhs { lhs } else { rhs }
}