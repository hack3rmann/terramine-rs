use { crate::prelude::*, std::borrow::Borrow };



#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Display, Default)]
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



#[derive(Clone, Debug)]
pub enum SharedStr {
    Owned(Arc<str>),
    Static(&'static str),
}

impl From<String> for SharedStr {
    fn from(value: String) -> Self {
        Self::Owned(value.into())
    }
}

impl From<&'static str> for SharedStr {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

impl From<StaticStr> for SharedStr {
    fn from(value: StaticStr) -> Self {
        match value.inner {
            Cow::Borrowed(string) => Self::Static(string),
            Cow::Owned(string) => Self::Owned(string.into()),
        }
    }
}

impl Deref for SharedStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(owned) => owned,
            Self::Static(value) => value,
        }
    }
}

impl PartialEq for SharedStr {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.deref(), other.deref())
    }
}

impl Eq for SharedStr { }

impl PartialOrd for SharedStr {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for SharedStr {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(self.deref(), other.deref())
    }
}

impl std::hash::Hash for SharedStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}