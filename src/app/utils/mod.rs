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



use {
    crate::prelude::*,
    std::{
        hash::{Hasher, BuildHasher, Hash},
        collections::hash_map::RandomState,
    },
};



/// A pre-hashed value of a specific type. Pre-hashing enables memoization of hashes that are expensive to compute.
/// It also enables faster [`PartialEq`] comparisons by short circuiting on hash equality.
/// See [`PassHash`] and [`PassHasher`] for a "pass through" [`BuildHasher`] and [`Hasher`] implementation
/// designed to work with [`Hashed`]
/// See [`PreHashMap`] for a hashmap pre-configured to use [`Hashed`] keys.
#[derive(Deref, Debug, Clone, PartialEq, Eq)]
pub struct Hashed<V, H = RandomState> {
    #[deref]
    value: V,

    hash: u64,
    _state_marker: PhantomData<H>,
}
assert_impl_all!(Hashed<i32>: Send, Sync);

impl<V: Hash, H: BuildHasher + Default> Hashed<V, H> {
    /// Pre-hashes the given value using the [`BuildHasher`] configured in the [`Hashed`] type.
    pub fn new(value: V) -> Self {
        let builder = H::default();
        let mut hasher = builder.build_hasher();
        value.hash(&mut hasher);
        
        Self {
            hash: hasher.finish(),
            value,
            _state_marker: PhantomData,
        }
    }

    /// The pre-computed hash.
    pub fn hash(&self) -> u64 {
        self.hash
    }
}

impl<V, H> Hash for Hashed<V, H> {
    fn hash<R: Hasher>(&self, state: &mut R) {
        state.write_u64(self.hash);
    }
}



/// Returns the "default value" for a type.
/// Default values are often some kind of initial value,
/// identity value, or anything else that may make sense as a default
pub fn default<T: Default>() -> T {
    T::default()
}



pub trait ToPhisicalSize<P: winit::dpi::Pixel> {
    fn to_phisical_size(&self) -> winit::dpi::PhysicalSize<P>;
}
assert_obj_safe!(ToPhisicalSize<u32>);

impl ToPhisicalSize<u32> for UInt2 {
    fn to_phisical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(self.x, self.y)
    }
}



pub trait ToVec2 {
    fn to_vec2(&self) -> UInt2;
}

impl ToVec2 for winit::dpi::PhysicalSize<u32> {
    fn to_vec2(&self) -> UInt2 {
        UInt2::new(self.width, self.height)
    }
}



/// [`Nullable`] type like in `C#`.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, SmartDefault)]
pub struct Nullable<T> {
    #[default(None)]
    pub inner: Option<T>,
}
assert_impl_all!(Nullable<f32>: Send, Sync);

impl<T> Nullable<T> {
    /// A `null` value for [`Nullable`].
    pub const NULL: Self = Self::null();

    /// Constructs new non-`null` [`Nullable`].
    pub const fn new(value: T) -> Self {
        Self { inner: Some(value) }
    }

    /// Constructs `null` [`Nullable`].
    pub const fn null() -> Self {
        Self { inner: None }
    }

    /// Checks if [`Nullable`] is `null`.
    pub const fn is_null(&self) -> bool {
        self.inner.is_none()
    }

    /// Takes inner value and leaves [`Nullable`] with `null` value.
    pub fn take(&mut self) -> T {
        self.inner.take().expect("called take on null Nullable")
    }

    /// Unwraps [`Nullable`] value into a type.
    pub fn into_inner(self) -> T {
        self.inner.expect("called into_inner on null Nullable")
    }

    /// Gives shared reference to inner type without `null`-check.
    /// 
    /// # Safety
    /// 
    /// - [`Nullable`] value is not `null`.
    pub const unsafe fn get_unchecked(&self) -> &T {
        self.inner.as_ref().unwrap_unchecked()
    }

    /// Gives `mut` reference to inner type without `null`-check.
    /// 
    /// # Safety
    /// 
    /// - [`Nullable`] value is not `null`.
    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        self.inner.as_mut().unwrap_unchecked()
    }

    /// Unwraps [`Nullable`] value into a type without `null`-check.
    /// 
    /// # Safety
    /// 
    /// - [`Nullable`] value is not `null`.
    pub unsafe fn into_inner_unchecked(self) -> T {
        self.inner.unwrap_unchecked()
    }
}

impl<T> From<T> for Nullable<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(value: Option<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> From<Nullable<T>> for Option<T> {
    fn from(value: Nullable<T>) -> Self {
        value.inner
    }
}

impl<T> Deref for Nullable<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("called deref on null Nullable")
    }
}

impl<T> DerefMut for Nullable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect("called deref_mut on null Nullable")
    }
}



macro_rules! impl_volume {
    ($VecType:ty, $ElemType:ty) => {
        impl Volume<$ElemType> for $VecType {
            fn volume(&self) -> $ElemType {
                self.x * self.y * self.z
            }
        }
    };
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
