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