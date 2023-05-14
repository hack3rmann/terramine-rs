#![macro_use]

#[macro_export]
macro_rules! define_atomic_id {
    ($atomic_id_type:ident) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, derive_deref_rs::Deref)]
        pub struct $atomic_id_type(core::num::NonZeroU64);

        // We use new instead of default to indicate that each ID created will be unique.
        #[allow(clippy::new_without_default)]
        impl $atomic_id_type {
            pub fn new() -> Self {
                use std::sync::atomic::{AtomicU64, Ordering};

                static COUNTER: AtomicU64 = AtomicU64::new(1);

                let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
                Self(core::num::NonZeroU64::new(counter).unwrap_or_else(|| {
                    panic!(
                        "The system ran out of unique `{}`s.",
                        stringify!($atomic_id_type)
                    );
                }))
            }
        }
    };
}

#[macro_export]
macro_rules! sum_errors {
    () => { };

    (
        $ErrName:ident { $($VariantName:ident => $ErrType:ty),+ $(,)? },
        $rest:tt $(,)?
    ) => {
        sum_errors! { $ErrName { $($VariantName => $ErrType,)+ } }
        sum_errors! { rest }
    };

    ($ErrName:ident { $($VariantName:ident => $ErrType:ty),+ $(,)? }) => {
        #[derive(Debug, thiserror::Error)]
        pub enum $ErrName {
            $(
                #[error(transparent)]
                $VariantName(#[from] $ErrType),
            )+
        }
    };
}