use crate::prelude::*;



#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Display, Default)]
#[display("{inner}")]
pub struct StrView<'s> {
    pub inner: Cow<'s, str>,
}
assert_impl_all!(StrView: Send, Sync);

impl<'s> From<StrView<'s>> for Cow<'s, str> {
    fn from(value: StrView<'s>) -> Self {
        value.inner
    }
}

impl<'s> From<&'s str> for StrView<'s> {
    fn from(value: &'s str) -> Self {
        Self { inner: Cow::Borrowed(value) }
    }
}

impl From<String> for StrView<'static> {
    fn from(value: String) -> Self {
        Self { inner: Cow::Owned(value) }
    }
}

impl<'s> From<&'s String> for StrView<'s> {
    fn from(value: &'s String) -> Self {
        Self { inner: Cow::Borrowed(value.as_str()) }
    }
}

impl std::ops::Deref for StrView<'_> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}



pub type StaticStr = StrView<'static>;
