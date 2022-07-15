use std::{fs::File, fs::OpenOptions, os::windows::prelude::FileExt, collections::HashSet, ops::Range};

use super::{Offset, Size};
use crate::app::utils::reinterpreter::*;

pub struct StackHeap {
	pub stack: File,
	pub stack_ptr: Offset,

	pub heap: File,
	pub eof: Offset,
	freed_space: HashSet<Range<Offset>>,
}

struct Alloc {
	offset: Offset,
	size: Size,
}

impl StackHeap {
	/// Makes new StackHeap struct and new directory for its files.
	pub fn new(path: &str, name: &str) -> Self {
		/* Create directory if this path doesn't exist */
		if !std::path::Path::new(path).exists() {
			std::fs::create_dir(path).unwrap();
		}
		
		Self {
			stack: OpenOptions::new().write(true).read(true).create(true).open(format!("{path}/{name}.stk")).unwrap(),
			heap:  OpenOptions::new().write(true).read(true).create(true).open(format!("{path}/{name}.hp")).unwrap(),
			stack_ptr: 0,
			eof: 0,
			freed_space: HashSet::new(),
		}
	}

	/// Saves file.
	pub fn sync(&self) -> std::io::Result<()> {
		self.stack.sync_all()?;
		self.heap.sync_all()?;

		Ok(())
	}

	/// Pushes data to stack. Returns an offset of that data.
	pub fn push(&mut self, data: &[u8]) -> Offset {
		/* Write new data */
		let offset = self.stack_ptr;
		self.stack.seek_write(data, self.stack_ptr).unwrap();

		/* Increment stack pointer */
		self.stack_ptr += data.len() as Size;

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

	/// Reads value from heap of file by offset from stack.
	pub fn read_from_heap<T: ReinterpretFromBytes + StaticSize>(&self, stack_offset: Offset) -> T {
		/* Read offset on heap from stack */
		let heap_offset: Offset = self.read_from_stack(stack_offset);

		/* Read bytes from heap */
		let mut buffer = vec![0; T::static_size()];
		self.heap.seek_read(&mut buffer, heap_offset + Offset::static_size() as Size).unwrap();

		/* Reinterpret */
		T::reinterpret_from_bytes(&buffer)
	}

	/// Allocates bytes on heap. Returns a pair of offsets on stack and on heap.
	pub fn alloc(&mut self, data: &[u8]) -> (Offset, Offset) {
		/* Test freed memory */
		let size = data.len() as Size;
		let heap_offset = match self.freed_space.iter().find(|range| range.end - range.start >= size + Offset::static_size() as Size).cloned() {
			None => self.eof,
			Some(range) => {
				self.freed_space.remove(&range);
				range.start
			}
		};

		/* Save size of data to heap */
		self.heap.seek_write(&(data.len() as Size).reinterpret_as_bytes(), heap_offset).unwrap();

		/* Save data */
		self.heap.seek_write(data, heap_offset + Offset::static_size() as Size).unwrap();

		/* Save this offset on stack */
		let stack_offset = self.push(&heap_offset.reinterpret_as_bytes());

		(stack_offset, heap_offset)
	}

	/// Marks memory as free.
	#[allow(dead_code)]
	pub fn free(&mut self, offset: Offset) {
		/* Read size from heap */
		let size = {
			let mut buffer = vec![0; Offset::static_size()];
			self.heap.seek_read(&mut buffer, offset).unwrap();
			Offset::reinterpret_from_bytes(&buffer)
		};

		/* Insert free range */
		self.freed_space.insert(offset .. offset + size);
	}
}