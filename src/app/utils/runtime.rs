use {
    tokio::runtime::{Runtime, Builder},
    lazy_static::lazy_static,
};

lazy_static! {
    pub static ref RUNTIME: Runtime = {
        Builder::new_multi_thread()
            .enable_all()
            .worker_threads(6)
            .thread_name("terramine-runtime-worker")
            .build()
            .expect("failed to build tokio runtime")
    };
}
