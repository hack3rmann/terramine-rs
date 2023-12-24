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

pub macro fmt_error($(fmt:tt)*) {
    crate::failure::StrError::from(format!($($fmt)*))
}



pub macro ensure_or($cond:expr, $diverging_expr:expr $(,)?) {
    if !$cond { $diverging_expr }
}

pub macro ensure($cond:expr, $err:expr $(,)?) {
    ensure_or!($cond, return Err($err.into()))
}

pub macro ensure_fmt($cond:expr, $($fmt:tt)*) {
    crate::failure::ensure!(
        $cond,
        crate::failure::fmt_error!($($fmt)*)
    )
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