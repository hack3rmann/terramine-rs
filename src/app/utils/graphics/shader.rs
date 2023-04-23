use {
    crate::{
        prelude::*,
        assets::FromSource,
    },
};

/// A shader defined by is's source code.
#[derive(Clone, Debug, PartialEq, Eq, Default, TypeUuid)]
#[uuid = "8faec540-e1d6-11ed-b9fb-0800200c9a66"]
pub struct Shader {
    pub source: Source,
}
assert_impl_all!(Shader: Send, Sync);

impl Shader {
    pub fn new(source: impl Into<Source>) -> Self {
        Self { source: source.into() }
    }
}

/// A shader source code string.
pub type Source = Cow<'static, str>;
assert_impl_all!(Source: Send, Sync);

impl FromSource for Shader {
    type Error = !;
    type Source = String;
    fn from_source(source: Self::Source) -> Result<Self, Self::Error> {
        Ok(Self::new(source))
    }
}
