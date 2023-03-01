use {
    crate::app::utils::terrain::chunk::DetailedVertex,
    std::{
        thread::{JoinHandle, self},
        mem,
    },
};

#[derive(Debug)]
pub struct Task {
    pub handle: Option<JoinHandle<Vec<DetailedVertex>>>,
}

impl Task {
    pub fn new(f: impl FnOnce() -> Vec<DetailedVertex> + Send + 'static) -> Self {
        Self { handle: Some(thread::spawn(f)) }
    }

    pub fn try_take_result(&mut self) -> Option<Vec<DetailedVertex>> {
        match self.handle {
            Some(ref mut handle) if handle.is_finished() =>
                mem::take(&mut self.handle)
                    .unwrap()
                    .join()
                    .ok(),

            _ => None,
        }
    }

    pub fn take_result(&mut self) -> Vec<DetailedVertex> {
        let handle = mem::take(&mut self.handle)
            .expect("task cannot be taken twice!");
        handle.join()
            .expect("task thread panicked")
    }
}