use std::sync::mpsc::{Receiver, RecvError, TryRecvError, TryIter};


pub struct Promise<T>(pub Receiver<T>);

impl<T> Promise<T> {
	/// Polls promise.
	pub fn poll(&self) -> Result<T, TryRecvError> {
		self.0.try_recv()
	}

	/// Polls promise and executes task.
	#[allow(dead_code)]
	pub fn poll_do<F: FnMut(T) -> ()>(&self, mut task: F) {
		if let Ok(item) = self.poll() {
			task(item);
		}
	}

	/// Polls promise and executes task. Then executes cleanup function.
	pub fn poll_do_cleanup<Task, Clean>(&self, mut task: Task, mut clean: Clean)
	where
		Task:  FnMut(T) -> (),
		Clean: FnMut()  -> ()
	{
		match self.poll() {
			Ok(item) => task(item),
			Err(TryRecvError::Disconnected) => clean(),
			_ => (),
		}
	}

	/// Waits promise to be ready.
	#[allow(dead_code)]
	pub fn wait(&self) -> Result<T, RecvError> {
		self.0.recv()
	}

	/// Gives an iterator over elements in promise.
	pub fn iter(&self) -> TryIter<T> {
		self.0.try_iter()
	}
}