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

	/// Writes data to stack by its offset
	pub fn write_to_stack(&mut self, offset: Offset, data: &[u8]) {
		self.stack.seek_write(data, offset).unwrap();
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
	#[allow(dead_code)]
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
		let heap_offset = self.get_available_offset(full_size);

		/* Save size of data to heap */
		self.heap.seek_write(&size.reinterpret_as_bytes(), heap_offset).unwrap();

		/* Save this offset on stack */
		let stack_offset = self.push(&heap_offset.reinterpret_as_bytes());

		Alloc { stack_offset, heap_offset, size }
	}

	/// Allocates space on heap. Returns a pair of offsets on stack and on a heap.
	pub fn realloc(&mut self, size: Size, stack_offset: Offset) -> Alloc {
		/* Read size that was before */
		let heap_offset = self.read_from_stack(stack_offset);
		let before_size = {
			let mut buffer = vec![0; Size::static_size()];
			self.heap.seek_read(&mut buffer, heap_offset).unwrap();
			Size::reinterpret_from_bytes(&buffer)
		};

		/* Calculate size include sizes bytes */
		let full_size = size + Size::static_size() as Size;

		if before_size < size {
			/* Free data on offset */
			self.free(stack_offset);

			/* Test freed memory */
			let heap_offset = self.get_available_offset(full_size);

			/* Save size of data to heap */
			self.heap.seek_write(&size.reinterpret_as_bytes(), heap_offset).unwrap();

			/* Save this offset on stack */
			self.write_to_stack(stack_offset, &heap_offset.reinterpret_as_bytes());

			Alloc { stack_offset, heap_offset, size }
		} else { 
			/* Write size to heap */
			self.heap.seek_write(&size.reinterpret_as_bytes(), heap_offset).unwrap();

			Alloc { stack_offset, heap_offset, size }
		}
	}

	/// Gives available offset on heap.
	fn get_available_offset(&mut self, size: Size) -> Offset {
		match self.freed_space.iter().find(|range| range.end - range.start >= size).cloned() {
			None => {
				let offset = self.eof;
				self.eof += size;
				offset
			},
			Some(range) => {
				/* Remove range from freed_space */
				self.freed_space.remove(&range);

				/* Compare sizes and insert remainder range */
				let range_size = range.end - range.start;
				if range_size != size {
					self.freed_space.insert(range.start .. range.start + size);
				}

				range.start
			}
		}
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
			let size = {
				let mut buffer = vec![0; Size::static_size()];
				self.heap.seek_read(&mut buffer, heap_offset).unwrap();
				Size::reinterpret_from_bytes(&buffer)
			};
			
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
			let merged = {
				let min = repeated.iter().map(|range| range.start).min().unwrap();
				let max = repeated.iter().map(|range| range.end  ).max().unwrap();
				min .. max
			};
			self.freed_space.insert(merged);
		}
	}
}