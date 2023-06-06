use crate::prelude::*;



pub trait ConstDefault {
    const DEFAULT: Self;
}

macro impl_nums_const_default($($Int:ty),* $(,)?) {
    $(
        impl ConstDefault for $Int {
            const DEFAULT: Self = 0 as Self;
        }
    )*
}

impl_nums_const_default! { i8, u8, i16, u16, i32, u32, f32, f64, i64, u64, isize, usize }

impl ConstDefault for bool {
    const DEFAULT: Self = false;
}

impl ConstDefault for char {
    const DEFAULT: Self = '\0';
}

impl ConstDefault for &str {
    const DEFAULT: Self = "";
}

impl ConstDefault for String {
    const DEFAULT: Self = Self::new();
}

impl<T> ConstDefault for Option<T> {
    const DEFAULT: Self = Self::None;
}

impl<V: ConstDefault, E> ConstDefault for Result<V, E> {
    const DEFAULT: Self = Ok(V::DEFAULT);
}

impl<T> ConstDefault for Vec<T> {
    const DEFAULT: Self = vec![];
}

impl<T: ConstDefault, const N: usize> ConstDefault for [T; N] {
    const DEFAULT: Self = [T::DEFAULT; N];
}

impl<T, const N: usize> ConstDefault for SmallVec<[T; N]> {
    const DEFAULT: Self = Self::new_const();
}

impl<T: ConstDefault> ConstDefault for Cell<T> {
    const DEFAULT: Self = Self::new(T::DEFAULT);
}

impl<T: ConstDefault> ConstDefault for RefCell<T> {
    const DEFAULT: Self = Self::new(T::DEFAULT);
}

impl<T: ConstDefault> ConstDefault for std::sync::Mutex<T> {
    const DEFAULT: Self = Self::new(T::DEFAULT);
}

impl<T: ConstDefault> ConstDefault for std::sync::RwLock<T> {
    const DEFAULT: Self = Self::new(T::DEFAULT);
}

impl<T: ConstDefault> ConstDefault for parking_lot::Mutex<T> {
    const DEFAULT: Self = Self::new(T::DEFAULT);
}

impl<T: ConstDefault> ConstDefault for parking_lot::RwLock<T> {
    const DEFAULT: Self = Self::new(T::DEFAULT);
}

impl<T> ConstDefault for VecDeque<T> {
    const DEFAULT: Self = Self::new();
}

macro impl_atomics($($AtomicName:ident($TypeName:ident)),* $(,)?) {
    $(
        impl ConstDefault for $AtomicName {
            const DEFAULT: Self = Self::new(<$TypeName>::DEFAULT);
        }
    )*
}

impl_atomics! {
    AtomicI8(i8), AtomicU8(u8), AtomicI16(i16), AtomicU16(u16), AtomicI32(i32),
    AtomicU32(u32), AtomicI64(i64), AtomicU64(u64), AtomicIsize(isize), AtomicUsize(usize),
}

impl<T> ConstDefault for AtomicPtr<T> {
    const DEFAULT: Self = Self::new(std::ptr::null_mut());
}

impl ConstDefault for Duration {
    const DEFAULT: Self = Self::new(0, 0);
}

impl ConstDefault for NotNan<f32> {
    const DEFAULT: Self = unsafe { Self::new_unchecked(f32::DEFAULT) };
}

impl ConstDefault for NotNan<f64> {
    const DEFAULT: Self = unsafe { Self::new_unchecked(f64::DEFAULT) };
}