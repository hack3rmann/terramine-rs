use crate::prelude::*;



pub trait Plugin: Sized {
    fn init(self, world: &mut World) -> impl Future<Output = AnyResult<()>> + Send;
}