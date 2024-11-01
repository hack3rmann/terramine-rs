use crate::prelude::*;



pub trait System {
    fn execute(&self, world: &mut World) -> impl Future<Output = AnyResult<()>>;
}



impl System for fn(&mut World) {
    async fn execute(&self, world: &mut World) -> AnyResult<()> {
        self(world);
        Ok(())
    }
}

impl System for fn(&mut World) -> AnyResult<()> {
    async fn execute(&self, world: &mut World) -> AnyResult<()> {
        self(world)
    }
}

impl System for fn(&World) {
    async fn execute(&self, world: &mut World) -> AnyResult<()> {
        self(world);
        Ok(())
    }
}

impl System for fn(&World) -> AnyResult<()> {
    async fn execute(&self, world: &mut World) -> AnyResult<()> {
        self(world)
    }
}