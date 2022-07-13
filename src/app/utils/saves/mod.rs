pub mod stack_heap;

use std::{marker::PhantomData, collections::HashMap, os::windows::prelude::FileExt};

use super::reinterpreter::{ReinterpretAsBytes, StaticSize};
use stack_heap::StackHeap;

pub type Offset = u64;

/// Handle for save files framework.
pub struct Save<E> {
	name: String,
	file: Option<StackHeap>,
	offests: HashMap<u64, Offset>,
	eof: Offset,

	_phantom_data: PhantomData<E>
}

impl<E: Copy + Into<u64>> Save<E> {
	/// Creates new Save struct.
	pub fn new(name: &str) -> Self {
		Self {
			name: name.to_owned(),
			file: None,
			offests: HashMap::new(),
			eof: 0,

			_phantom_data: PhantomData
		}
	}

	/// Creates heap-stack folder.
	pub fn create(mut self, path: &str) -> Self {
		self.file = Some(StackHeap::new(path, &self.name));
		return self
	}

	/// Writes enum-named value to file.
	pub fn write<T: ReinterpretAsBytes + StaticSize>(mut self, value: &T, enumerator: E) -> Self {
		/* Write value to file stack */
		let bytes = value.reinterpret_as_bytes();
		self.file.as_ref().unwrap().stack.seek_write(&bytes, self.eof).unwrap();

		/* Saving offset of value */
		self.offests.insert(enumerator.into(), T::static_size() as Offset).expect("Trying to write same data to another place");

		/* Increment of `end of file` */
		self.eof += T::static_size() as Offset;

		return self
	}
}