use { crate::prelude::*, std::borrow::Borrow };



#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Display, Default)]
#[display("{inner}")]
pub struct StrView<'s> {
    pub inner: Cow<'s, str>,
}
assert_impl_all!(StrView: Send, Sync);

impl<'s> StrView<'s> {
    pub fn to_static(self) -> StaticStr {
        StaticStr::from(self.inner.into_owned())
    }
}

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

impl ConstDefault for StrView<'static> {
    const DEFAULT: Self = Self { inner: Cow::Borrowed(<&str>::DEFAULT) };
}

impl Deref for StrView<'_> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl AsRef<str> for StrView<'_> {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

impl Borrow<str> for StrView<'_> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}



pub type StaticStr = StrView<'static>;
