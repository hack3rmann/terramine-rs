use crate::prelude::*;



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Name {
    pub value: StaticStr,
}
assert_impl_all!(Name: Send, Sync, Component);

impl Name {
    pub fn new(value: impl Into<StaticStr>) -> Self {
        Self { value: value.into() }
    }
}