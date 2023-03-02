use {
    crate::app::utils::{
        terrain::chunk::{DetailedVertex, LoweredVertex},
        profiler::prelude::*,
    },
    std::future::Future,
    tokio::task::JoinHandle,
};

#[derive(Debug)]
pub struct Task<Item> {
    pub handle: Option<JoinHandle<Item>>,
}

pub type FullTask = Task<Vec<DetailedVertex>>;
pub type LowTask  = Task<Vec<LoweredVertex>>;

impl<Item: Send + 'static> Task<Item> {
    pub fn spawn(f: impl Future<Output = Item> + Send + 'static) -> Self {
        Self { handle: Some(tokio::spawn(f)) }
    }

    #[profile]
    pub async fn try_take_result(&mut self) -> Option<Item> {
        match self.handle.take() {
            Some(handle) if handle.is_finished() =>
                handle.await.ok(),

            Some(handle) => {
                self.handle = Some(handle);
                None
            },

            None => None,
        }
    }

    pub async fn take_result(&mut self) -> Item {
        self.handle.take()
            .expect("task cannot be taken twice!")
            .await
            .expect("task thread panicked")
    }
}