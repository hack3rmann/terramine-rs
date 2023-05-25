use {
    crate::prelude::*,
    std::{
        hash::{Hasher, BuildHasher, Hash},
        collections::hash_map::RandomState,
    },
};



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