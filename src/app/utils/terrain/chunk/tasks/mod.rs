use {
    crate::{
        prelude::*,
        terrain::chunk::VoxelId,
        graphics::Mesh,
    },
    std::future::Future,
    tokio::task::JoinHandle,
};



#[derive(Debug)]
pub struct Task<Item> {
    pub handle: Nullable<JoinHandle<Item>>,
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



impl<Item: Send + 'static> Task<Item> {
    pub fn spawn(f: impl Future<Output = Item> + Send + 'static) -> Self {
        Self { handle: Nullable::new(tokio::spawn(f)) }
    }

    pub async fn try_take_result(&mut self) -> Option<Item> {
        ensure_or!(!self.handle.is_null() && self.handle.is_finished(), return None);

        let handle = self.handle.take();
        handle.await.ok()
    }

    pub async fn take_result(&mut self) -> Item {
        self.try_take_result().await
            .expect("cannot take a result twice")
    }

    pub async fn try_take_results<K, V>(tasks: impl IntoIterator<Item = (K, V)>) -> SmallVec<[(K, Item); 16]>
    where
        V: AsMut<Self> + AsRef<Self>,
    {
        let futs = tasks.into_iter()
            .filter(|(_, task)| {
                let handle = &task.as_ref().handle;
                !handle.is_null() && handle.is_finished()
            })
            .map(|(key, mut task)| async move {
                (key, task.as_mut().take_result().await)
            });

        let mut result = SmallVec::new();

        for future in futs {
            result.push(future.await)
        }

        result
    }
}

impl<Item> Drop for Task<Item> {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            self.handle.abort();
        }
    }
}



pub type FullTask = Task<Mesh>;
pub type LowTask  = Task<Mesh>;
pub type GenTask  = Task<Vec<VoxelId>>;
pub type PartitionTask = Task<[Mesh; 8]>;