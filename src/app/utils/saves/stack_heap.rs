use std::{fs::File, os::windows::prelude::FileExt, collections::HashMap};

use super::Offset;
use crate::app::utils::reinterpreter::*;

pub struct StackHeap {
	pub stack: File,
	pub stack_ptr: Offset,

	pub heap: File,
	pub eof: Offset,
	freed_space: HashMap<Offset, Offset>,
}

impl StackHeap {
	/// Makes new StackHeap struct and new directory for its files.
	pub fn new(path: &str, name: &str) -> Self {
		Self {
			stack: File::create(format!("{path}/{name}.stk")).unwrap(),
			heap:  File::create(format!("{path}/{name}.hp")).unwrap(),
			stack_ptr: 0,
			eof: 0,
			freed_space: HashMap::new(),
		}
	}

	/// Pushes data to stack. Returns an offset of that data.
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

	pub fn read_from_heap<T: ReinterpretFromBytes + StaticSize>(&self, stack_offset: Offset) -> T {
		/* Read offset on heap from stack */
		let heap_offset: Offset = self.read_from_stack(stack_offset);

		/* Read bytes from heap */
		let mut buffer = vec![0; T::static_size()];
		self.heap.seek_read(&mut buffer, heap_offset).unwrap();

		/* Reinterpret */
		T::reinterpret_from_bytes(&buffer)
	}

	/// Allocates bytes on heap. Returns a pair of offsets on stack and on heap.
	pub fn allocate(&mut self, data: &[u8]) -> (Offset, Offset) {
		/* TODO: test freed memory */

		/* Write to heap */
		let heap_offset = self.eof;
		self.heap.seek_write(data, heap_offset).unwrap();

		/* Save this offset on stack */
		let stack_offset = self.push(&heap_offset.reinterpret_as_bytes());

		(stack_offset, heap_offset)
	}
}