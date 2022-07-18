use std::{fs::File, fs::OpenOptions, os::windows::prelude::FileExt, collections::HashSet, ops::Range};

use super::{Offset, Size};
use crate::app::utils::reinterpreter::*;

#[derive(Debug)]
pub struct ShError(pub String);

pub type ShResult<T> = Result<T, ShError>;

#[derive(Clone, Copy)]
pub struct Alloc {
	pub(in self) stack_offset: Offset,
	pub(in self) heap_offset: Offset,
	pub(in self) size: Size,
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
	/// Makes new StackHeap struct and new directory for their files.
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

	/// Saves the files.
	pub fn sync(&self) -> std::io::Result<()> {
		self.stack.sync_all()?;
		self.heap.sync_all()?;

		Ok(())
	}

	/// Pushes data to stack. Returns an offset of the data.
	pub fn push(&mut self, data: &[u8]) -> Offset {
		/* Write new data */
		let offset = self.stack_ptr;
		self.stack.seek_write(data, offset).unwrap();

		/* Increment stack pointer */
		self.stack_ptr += data.len() as Size;

		return offset
	}

	/// Writes data to stack by its offset.
	pub fn write_to_stack(&mut self, offset: Offset, data: &[u8]) {
		self.stack.seek_write(data, offset).unwrap();
	}

	/// Reads value from stack.
	pub fn read_from_stack<T: ReinterpretFromBytes + StaticSize>(&self, offset: Offset) -> T {
		/* Read bytes */
		let mut buffer = vec![0; T::static_size()];
		self.stack.seek_read(&mut buffer, offset).unwrap();

		/* Reinterpret */
		T::reinterpret_from_bytes(&buffer)
	}

	/// Reads value from heap by `heap_offset`.
	/// * Note: `heap_offset` should be point on Size mark of the data.
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

	/// Reads value from heap of file by offset on stack.
	#[allow(dead_code)]
	pub fn heap_read<T: ReinterpretFromBytes + StaticSize>(&self, stack_offset: Offset) -> ShResult<T> {
		/* Read offset on heap from stack */
		let heap_offset: Offset = self.read_from_stack(stack_offset);

		/* Read bytes */
		let bytes = self.read_from_heap(heap_offset);

		/* Read bytes from heap */
		if bytes.len() == T::static_size() {
			Ok(T::reinterpret_from_bytes(&bytes))
		} else {
			Err(ShError("Data lengthes on given offset and on passed type T are not equal!".to_owned()))
		}
	}

	/// Allocates space on heap. Returns an Alloc struct that contains all information about this allocation.
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

	/// Reallocates space on heap. Returns an Alloc struct that contains all information about this allocation.
	/// * Note: it can avoid alocation if new size isn't greater than old.
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
			/* If new size cause free memory then use it */
			if before_size != size {
				self.insert_free(heap_offset + full_size, before_size - size);
			}

			/* Write size to heap */
			self.heap.seek_write(&size.reinterpret_as_bytes(), heap_offset).unwrap();

			Alloc { stack_offset, heap_offset, size }
		}
	}

	/// Stoles available offset from heap. It can edit freed_space so it is expensive.
	/// * Note: size is a full size of allocation, include size mark in heap
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
					self.freed_space.insert(range.start + size .. range.end);
				}

				range.start
			}
		}
	}

	/// Writes bytes to heap. Alloc struct must be passed in. It's a contract to write to available allocated chunk of bytes.
	pub fn write_to_heap(&self, Alloc { size, heap_offset: offset, .. }: Alloc, data: &[u8]) -> ShResult<()> {
		if size >= data.len() as Size {
			self.heap.seek_write(data, offset + Size::static_size() as Size).unwrap();
			Ok(())
		} else {
			Err(ShError(format!(
				"Data size ({}) passed to this function should be not greater than allowed allocation ({})!",
				size, data.len()
			)))
		}
	}

	/// Marks memory as free.
	#[allow(dead_code)]
	pub fn free(&mut self, stack_offset: Offset) {
		/* Read offset and size */
		let heap_offset: Offset = self.read_from_stack(stack_offset);
		let size = {
			let mut buffer = vec![0; Size::static_size()];
			self.heap.seek_read(&mut buffer, heap_offset).unwrap();

			/* Note: Size mark in heap is included */
			Size::reinterpret_from_bytes(&buffer) + Size::static_size() as Size
		};
		
		/* Insert free range */
		self.insert_free(heap_offset, size);
	}

	/// Inserts free space to freed_space HashSet and merges free ranges.
	fn insert_free(&mut self, heap_offset: Offset, size: Size) {
		/* Insert new free space marker */
		let free_range = heap_offset .. heap_offset + size;

		/* Seek all mergable ranges */
		let mut repeated = [free_range.clone(), free_range.clone(), free_range.clone()];
		for range in self.freed_space.iter() {
			if range.end   == free_range.start {
				repeated[0] = range.clone();
			} else
			if range.start == free_range.end {
				repeated[2] = range.clone();
			}			
		}

		/* Remove all found ranges */
		self.freed_space.remove(&repeated[0]);
		self.freed_space.remove(&repeated[2]);

		/* Insert all-include range */
		self.freed_space.insert(repeated[0].start .. repeated[2].end);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_allocation() {
		let name = "test_allocation";
		let mut file = StackHeap::new(name, name);

		let bytes_64:  Vec<_> = (0_u64..).flat_map(|num| num.reinterpret_as_bytes()).take(64) .collect();
		let bytes_128: Vec<_> = (0_u64..).flat_map(|num| num.reinterpret_as_bytes()).take(128).collect();

		let bytes_64_rev: Vec<_> = bytes_64.iter().map(|&byte| byte).rev().collect();

		let alloc_64 = file.alloc(64);
		file.write_to_heap(alloc_64, &bytes_64).unwrap();

		let alloc_128 = file.realloc(128, alloc_64.get_stack_offset());
		file.write_to_heap(alloc_128, &bytes_128).unwrap();

		let alloc_64_rev = file.alloc(64);
		file.write_to_heap(alloc_64_rev, &bytes_64_rev).unwrap();

		let bytes_after = file.read_from_heap(alloc_128.get_heap_offset());
		let bytes_rev_after = file.read_from_heap(alloc_64_rev.get_heap_offset());

		assert_eq!(bytes_128, bytes_after);
		assert_eq!(bytes_64_rev, bytes_rev_after);
		assert_eq!(file.stack_ptr, 2 * Offset::static_size() as Size);
		assert_eq!(file.eof, 64 + 128 + 2 * Size::static_size() as Size);
	}

	#[test]
	fn test_merging() {
		let name = "test_merging";
		let mut file = StackHeap::new(name, name);

		let bytes_64:  Vec<_> = (0_u64..).flat_map(|num| num.reinterpret_as_bytes()).take(64) .collect();
		let bytes_128: Vec<_> = (0_u64..).flat_map(|num| num.reinterpret_as_bytes()).take(128).collect();

		let alloc_64_1 = file.alloc(64);
		file.write_to_heap(alloc_64_1, &bytes_64).unwrap();

		let alloc_64_2 = file.alloc(64);
		file.write_to_heap(alloc_64_2, &bytes_64).unwrap();

		assert_eq!(bytes_64, file.read_from_heap(alloc_64_1.get_heap_offset()));
		assert_eq!(bytes_64, file.read_from_heap(alloc_64_2.get_heap_offset()));

		file.free(alloc_64_1.get_stack_offset());
		file.free(alloc_64_2.get_stack_offset());

		let alloc_128 = file.alloc(128);
		file.write_to_heap(alloc_128, &bytes_128).unwrap();

		assert_eq!(bytes_128, file.read_from_heap(alloc_128.get_heap_offset()));
		assert_eq!(file.stack_ptr, 3 * Offset::static_size() as Size);
		assert_eq!(file.eof, 2 * 64 + 2 * Size::static_size() as Size);

		let range = 128 + Size::static_size() as Size .. 2 * (64 + Size::static_size() as Size);
		assert!(file.freed_space.contains(&range));
	}
}