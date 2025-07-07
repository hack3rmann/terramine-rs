use {
    crate::{
        prelude::*,
        terrain::chunk::{FullVertex, Id, LowVertex},
    },
    std::future::Future,
    tokio::task::JoinHandle,
};

#[derive(Debug)]
pub struct Task<Item> {
    pub handle: Option<JoinHandle<Item>>,
}

impl<Item> AsRef<Task<Item>> for Task<Item> {
    fn as_ref(&self) -> &Task<Item> {
        self
    }
}

impl<Item> AsMut<Task<Item>> for Task<Item> {
    fn as_mut(&mut self) -> &mut Task<Item> {
        self
    }
}

pub type FullTask = Task<Vec<FullVertex>>;
pub type LowTask = Task<Vec<LowVertex>>;
pub type GenTask = Task<Vec<Atomic<Id>>>;
pub type PartitionTask = Task<[Vec<FullVertex>; 8]>;

impl<Item: Send + 'static> Task<Item> {
    pub fn spawn(f: impl Future<Output = Item> + Send + 'static) -> Self {
        Self {
            handle: Some(tokio::spawn(f)),
        }
    }

    pub async fn try_take_result(&mut self) -> Option<Item> {
        match self.handle.take() {
            Some(handle) if handle.is_finished() => handle.await.ok(),

            Some(handle) => {
                self.handle = Some(handle);
                None
            }

            None => None,
        }
    }

    pub async fn take_result(&mut self) -> Item {
        self.handle
            .take()
            .expect("task cannot be taken twice!")
            .await
            .expect("task thread panicked")
    }

    pub async fn try_take_results<K, V>(
        tasks: impl Iterator<Item = (K, V)>,
    ) -> SmallVec<[(K, Item); 16]>
    where
        V: AsMut<Self> + AsRef<Self>,
    {
        let futs = tasks
            .into_iter()
            .filter(|(_, task)| match task.as_ref().handle {
                Some(ref handle) => handle.is_finished(),
                None => false,
            })
            .map(|(key, mut task)| async move { (key, task.as_mut().take_result().await) });

        let mut result = SmallVec::new();

        for future in futs {
            result.push(future.await)
        }

        result
    }
}

impl<Item> Drop for Task<Item> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}
