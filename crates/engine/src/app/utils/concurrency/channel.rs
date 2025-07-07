use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub struct Channel<T> {
    pub sender: UnboundedSender<T>,
    pub receiver: UnboundedReceiver<T>,
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }
}
