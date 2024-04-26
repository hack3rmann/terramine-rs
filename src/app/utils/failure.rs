//! Set of simple macros similar to [`panic!`] but with fuction returning behaviour.

use { crate::prelude::*, derive_more::{Error, Display} };



assert_impl_all!(StrError: Send, Sync, Into<AnyError>);
#[derive(Clone, Debug, Display, Error, Constructor)]
pub struct StrError {
    pub inner: StaticStr,
}

impl<IntoStr: Into<StaticStr>> From<IntoStr> for StrError {
    fn from(value: IntoStr) -> Self {
        Self::new(value.into())
    }
}

#[macro_export]
macro_rules! fmt_error {
    ($(fmt:tt)*) => { $crate::failure::StrError::from(format!($($fmt)*)) };
}

pub use fmt_error;



#[macro_export]
macro_rules! ensure_or {
    ($cond:expr, $diverging_expr:expr $(,)?) => { if !$cond { $diverging_expr } };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr $(,)?) => { ensure_or!($cond, return Err($err.into())) };
}

#[macro_export]
macro_rules! ensure_fmt {
    ($cond:expr, $($fmt:tt)*) => {
        $crate::failure::ensure!(
            $cond,
            $crate::failure::fmt_error!($($fmt)*)
        )
    };
}

#[macro_export]
macro_rules! ensure_eq {
    ($lhs:expr, $rhs:expr, $err:expr) => { ensure!($lhs == $rhs, $err); };
}

#[macro_export]
macro_rules! ensure_ne {
    ($lhs:expr, $rhs:expr, $err:expr) => { ensure!($lhs != $rhs, $err); };
}

#[macro_export]
macro_rules! bail {
    ($err:expr) => { return Err($err.into()) };
}

#[macro_export]
macro_rules! bail_str {
    ($($args:tt)*) => {
        return Err($crate::failure::StrError::from(format!($($args)*)).into())
    };
}

pub use {ensure_or, ensure, ensure_fmt, ensure_eq, ensure_ne, bail, bail_str};