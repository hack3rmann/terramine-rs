use {
    tokio::runtime::{Runtime, Builder},
};

pub mod prelude {
    pub use super::{
        runtime,
    };

}

static mut RUNTIME: Option<Runtime> = None;

pub fn initialize() {
    unsafe {
        RUNTIME.replace(
            Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build tokio runtime")
        );
    }
}

pub fn runtime<'l>() -> &'l Runtime {
    unsafe {
        RUNTIME.as_ref()
            .expect("runtime is not initialized!")
    }
}