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