use {
    crate::{
        prelude::*,
        assets::FromSource,
    },
};



/// A shader defined by is's source code.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Shader {
    pub source: Source,
}
assert_impl_all!(Shader: Send, Sync);

impl Shader {
    pub fn new(source: impl Into<Source>) -> Self {
        Self { source: source.into() }
    }
}

impl FromSource for Shader {
    type Error = !;
    type Source = String;
    fn from_source(source: Self::Source) -> Result<Self, Self::Error> {
        Ok(Self::new(source))
    }
}



/// A shader source code string.
pub type Source = StaticStr;
assert_impl_all!(Source: Send, Sync);
