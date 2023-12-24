pub use crate::{define_atomic_id, sum_errors};



#[macro_export]
macro_rules! define_atomic_id {
    ($AtomicIdType:ident) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
        pub struct $AtomicIdType(core::num::NonZeroU64);
    
        // We use new instead of default to indicate that each ID created will be unique.
        #[allow(clippy::new_without_default)]
        impl $AtomicIdType {
            pub fn new() -> Self {
                use std::sync::atomic::{AtomicU64, Ordering};
    
                static COUNTER: AtomicU64 = AtomicU64::new(1);
    
                let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
                Self(core::num::NonZeroU64::new(counter).unwrap_or_else(|| {
                    panic!(
                        "The system ran out of unique `{}`s.",
                        stringify!($AtomicIdType)
                    );
                }))
            }
        }
    
        impl std::ops::Deref for $AtomicIdType {
            type Target = core::num::NonZeroU64;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    
        impl std::ops::DerefMut for $AtomicIdType {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}



#[macro_export]
macro_rules! sum_errors {
    ($vis:vis enum $ErrName:ident { $($VariantName:ident => $ErrType:ty),+ $(,)? }) => {
        #[derive(Debug, thiserror::Error)]
        $vis enum $ErrName {
            $(
                #[error(transparent)]
                $VariantName(#[from] $ErrType),
            )+
        }
    };
}



/// Makes atomic static's declarations look like usual static's declarations.
/// 
/// # Example
/// 
/// ```
/// # use crate::macros::atomic_static;
/// 
/// #[derive(Clone, Copy)]
/// enum Weather {
///     Sunny,
///     Rainy,
///     Cloudy,
/// }
/// 
/// atomic_static! {
///     pub static WEATHER: Weather = Weather::Sunny;
///     pub static IS_WEATHER_GOOD: bool = true;
/// }
/// ```
/// 
/// is equivalent to
/// 
/// ```
/// # use std::sync::atomic::AtomicBool;
/// # use atomic::Atomic;
/// pub static IS_WEATHER_GOOD: AtomicBool = AtomicBool::new(true);
/// pub static WEATHER: Atomic<Weather> = Atomic::new(Weather::Sunny);
/// ```
pub macro atomic_static($($vis:vis static $NAME:ident: $Type:ty = $init:expr;)*) {
    $(
        $vis static $NAME: crate::macros::Atomic!($Type) = <crate::macros::Atomic!($Type)>::new($init);
    )*
}

/// Makes an atomic type out of any [`Copy`] type.
/// 
/// # Example
/// 
/// Common types are already atomics:
/// 
/// ```
/// # use crate::macros::Atomic;
/// type AtomicUsize = Atomic!(usize);
/// ```
/// 
/// And any small type too:
/// 
/// ```
/// # use crate::macros::Atomic;
/// #[derive(Clone, Copy)]
/// enum Animal {
///     Dog,
///     Cat,
/// }
/// 
/// type AtomicAnimal = Atomic!(Animal);
/// ```
pub macro Atomic {
    (i8) => { ::std::sync::atomic::AtomicI8 },
    (u8) => { ::std::sync::atomic::AtomicU8 },
    (i16) => { ::std::sync::atomic::AtomicI16 },
    (u16) => { ::std::sync::atomic::AtomicU16 },
    (i32) => { ::std::sync::atomic::AtomicI32 },
    (u32) => { ::std::sync::atomic::AtomicU32 },
    (i64) => { ::std::sync::atomic::AtomicI64 },
    (i128) => { ::portable_atomic::AtomicI128 },
    (u128) => { ::portable_atomic::AtomicU128 },
    (isize) => { ::std::sync::atomic::AtomicIsize },
    (usize) => { ::std::sync::atomic::AtomicUsize },
    (f32) => { ::portable_atomic::AtomicF32 },
    (f64) => { ::portable_atomic::AtomicF64 },
    (bool) => { ::std::sync::atomic::AtomicBool },
    ($Type:ty) => { ::atomic::Atomic<$Type> },
}

pub macro load($ordering:ident: $($name:ident),*) {
    ($(
        $name.load(::std::sync::atomic::Ordering::$ordering),
    )*)
}

pub macro store($ordering:ident: $($name:ident = $value:expr),*) {
    ($(
        $name.store($value, ::std::sync::atomic::Ordering::$ordering),
    )*)
}

/// Makes a 2-D vector out of type.
pub macro Vec2 {
    (i8) => { ::math_linear::prelude::Byte2 },
    (u8) => { ::math_linear::prelude::UByte2 },
    (i16) => { ::math_linear::prelude::Short2 },
    (u16) => { ::math_linear::prelude::UShort2 },
    (i32) => { ::math_linear::prelude::Int2 },
    (u32) => { ::math_linear::prelude::UInt2 },
    (i64) => { ::math_linear::prelude::Long2 },
    (u64) => { ::math_linear::prelude::ULong2 },
    (i128) => { ::math_linear::prelude::Large2 },
    (u128) => { ::math_linear::prelude::ULarge2 },
    (f32) => { ::math_linear::prelude::Float2 },
    (f64) => { ::math_linear::prelude::Double2 },
    (isize) => { ::math_linear::prelude::ISize2 },
    (usize) => { ::math_linear::prelude::USize2 },
    ($other:ty) => { compile_error!("only primitive types are supported") },
}

/// Makes a 3-D vector out of type.
pub macro Vec3 {
    (i8) => { ::math_linear::prelude::Byte3 },
    (u8) => { ::math_linear::prelude::UByte3 },
    (i16) => { ::math_linear::prelude::Short3 },
    (u16) => { ::math_linear::prelude::UShort3 },
    (i32) => { ::math_linear::prelude::Int3 },
    (u32) => { ::math_linear::prelude::UInt3 },
    (i64) => { ::math_linear::prelude::Long3 },
    (u64) => { ::math_linear::prelude::ULong3 },
    (i128) => { ::math_linear::prelude::Large3 },
    (u128) => { ::math_linear::prelude::ULarge3 },
    (f32) => { ::math_linear::prelude::Float3 },
    (f64) => { ::math_linear::prelude::Double3 },
    (isize) => { ::math_linear::prelude::ISize3 },
    (usize) => { ::math_linear::prelude::USize3 },
    ($other:ty) => { compile_error("only primitive types are supported") },
}

/// BUilds a query on world
pub macro query($world:expr, $(qvalue:ty),+ $(,)?) {
    world.query::<($(qvalue,)+)>();
}



pub macro formula($formula:expr, where $($variable:ident = $value:expr),* $(,)?) {
    {
        $(
            let $variable = $value;
        )+

        $formula
    }
}