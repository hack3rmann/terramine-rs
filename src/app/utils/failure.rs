//! Set of simple macros similar to [`panic!`] but with fuction returning behaviour.

use { crate::prelude::*, derive_more::{Error, Display} };



#[derive(Clone, Debug, Display, Error, Constructor)]
pub struct StrError {
    pub inner: StaticStr,
}
assert_impl_all!(StrError: Send, Sync, Into<AnyError>);

impl<IntoStr: Into<StaticStr>> From<IntoStr> for StrError {
    fn from(value: IntoStr) -> Self {
        Self::new(value.into())
    }
}



pub macro ensure_or($cond:expr, $diverging_expr:expr) {
    if !$cond { $diverging_expr }
}

pub macro ensure($cond:expr, $err:expr) {
    ensure_or!($cond, return Err($err.into()))
}

pub macro ensure_eq($lhs:expr, $rhs:expr, $err:expr) {
    ensure!($lhs == $rhs, $err);
}

pub macro ensure_ne($lhs:expr, $rhs:expr, $err:expr) {
    ensure!($lhs != $rhs, $err);
}

pub macro bail($err:expr) {
    return Err($err.into())
}

pub macro bail_str($($args:tt)*) {
    return Err($crate::failure::StrError::from(format!($($args)*)).into())
}