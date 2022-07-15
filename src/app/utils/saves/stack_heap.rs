use std::{fs::File, fs::OpenOptions, os::windows::prelude::FileExt, collections::HashSet, ops::Range};

use super::{Offset, Size};
use crate::app::utils::reinterpreter::*;

#[derive(Clone, Copy)]
pub struct Alloc {
	pub(in super::stack_heap) stack_offset: Offset,
	pub(in super::stack_heap) heap_offset: Offset,
	pub(in super::stack_heap) size: Size,
}

#[allow(dead_code)]
impl Alloc {
	pub fn get_stack_offset(self) -> Offset { self.stack_offset }
	pub fn get_heap_offset(self) -> Offset { self.heap_offset }
	pub fn get_size(self) -> Offset { self.size }
}

pub struct StackHeap {
	pub stack: File,
	pub stack_ptr: Offset,

	pub heap: File,
	pub eof: Offset,
	freed_space: HashSet<Range<Offset>>,
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
		self.stack.seek_write(data, offset).unwrap();

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

	/// Reads value from stack
	pub fn read_from_heap(&self, heap_offset: Offset) -> Vec<u8> {
		/* Read size */
		let size = {
			let mut buffer = vec![0; Size::static_size()];
			self.heap.seek_read(&mut buffer, heap_offset).unwrap();
			Size::reinterpret_from_bytes(&buffer)
		};

		/* Read data */
		let mut buffer = vec![0; size as usize];
		self.heap.seek_read(&mut buffer, heap_offset + Size::static_size() as Size).unwrap();

		return buffer
	}

	/// Reads value from heap of file by offset from stack.
	pub fn heap_read<T: ReinterpretFromBytes + StaticSize>(&self, stack_offset: Offset) -> T {
		/* Read offset on heap from stack */
		let heap_offset: Offset = self.read_from_stack(stack_offset);

		/* Read bytes from heap */
		T::reinterpret_from_bytes(&self.read_from_heap(heap_offset))
	}

	/// Allocates space on heap. Returns a pair of offsets on stack and on a heap.
	pub fn alloc(&mut self, size: Size) -> Alloc {
		/* Test freed memory */
		let full_size = size + Offset::static_size() as Size;
		let heap_offset = match self.freed_space.iter().find(|range| range.end - range.start >= full_size).cloned() {
			None => {
				let offset = self.eof;
				self.eof += full_size;
				offset
			},
			Some(range) => {
				self.freed_space.remove(&range);
				range.start
			}
		};

		/* Save size of data to heap */
		self.heap.seek_write(&size.reinterpret_as_bytes(), heap_offset).unwrap();

		/* Save this offset on stack */
		let stack_offset = self.push(&heap_offset.reinterpret_as_bytes());

		Alloc { stack_offset, heap_offset, size }
	}

	/// Writes bytes to heap.
	pub fn write_to_heap(&self, Alloc { size, heap_offset: offset, .. }: Alloc, data: &[u8]) {
		assert!(size >= data.len() as Size, "Data size passed to this function should be not greater than allowed allocation!");
		self.heap.seek_write(data, offset + Size::static_size() as Size).unwrap();
	}

	/// Marks memory as free.
	#[allow(dead_code)]
	pub fn free(&mut self, stack_offset: Offset) {
		/* Construct Alloc struct */
		let alloc = {
			let heap_offset: Offset = self.read_from_stack(stack_offset);
			let size = Size::reinterpret_from_bytes(&self.read_from_heap(heap_offset));
			Alloc { stack_offset, heap_offset, size }
		};
		
		/* Insert free range */
		self.insert_free(alloc);
	}

	/// Inserts free space Alloc to set.
	fn insert_free(&mut self, alloc: Alloc) {
		/* Insert new free space marker */
		let free_range = alloc.heap_offset .. alloc.heap_offset + alloc.size;
		self.freed_space.insert(free_range.clone());

		/* Seek all mergable ranges */
		let mut repeated = Vec::with_capacity(3);
		for (curr, next) in self.freed_space.iter().zip(self.freed_space.iter().skip(1)) {
			if curr.end == next.start {
				repeated.push(curr.clone());
				repeated.push(next.clone());
			}
		}

		/* Then merge them */
		if !repeated.is_empty() {
			repeated.iter().for_each(|range| { self.freed_space.remove(range); });
			let merged = repeated.first().unwrap().start .. repeated.last().unwrap().end;
			self.freed_space.insert(merged);
		}
	}
}