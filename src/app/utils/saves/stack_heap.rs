use std::{fs::File, os::windows::prelude::FileExt};

use super::Offset;
use crate::app::utils::reinterpreter::*;

pub struct StackHeap {
	pub stack: File,
	pub heap: File,
	pub stack_ptr: Offset
}

impl StackHeap {
	/// Makes new StackHeap struct and new directory for its files.
	pub fn new(path: &str, name: &str) -> Self {
		Self {
			stack: File::create(format!("{path}/{name}.stk")).unwrap(),
			heap:  File::create(format!("{path}/{name}.hp")).unwrap(),
			stack_ptr: 0
		}
	}

	/// Pushes data to stack.
	pub fn push(&mut self, data: &[u8]) -> Offset {
		/* Write new data */
		let offset = self.stack_ptr;
		self.stack.seek_write(data, self.stack_ptr).unwrap();

		/* Increment stack pointer */
		self.stack_ptr += data.len() as Offset;

		return offset
	}

	/// Reads value from stack
	pub fn read_from_stack<T: ReinterpretFromBytes + StaticSize>(&self, offset: Offset) -> T {
		/* Read bytes */
		let mut buffer = vec![0; T::static_size()];
		self.stack.seek_read(&mut buffer, offset).unwrap();

		/* Reinterpret */
		T::reinterpret_from_bytes(&buffer)
	}
}