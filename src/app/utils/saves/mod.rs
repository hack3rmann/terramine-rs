#![allow(dead_code)]

pub mod stack_heap;

use std::{marker::PhantomData, collections::HashMap, os::windows::prelude::FileExt};

use stack_heap::StackHeap;
use crate::app::utils::{
	reinterpreter::{
		ReinterpretAsBytes,
		ReinterpretFromBytes,
		StaticSize
	},
};

pub type Offset = u64;

/// Handle for save files framework.
pub struct Save<E> {
	name: String,
	file: Option<StackHeap>,
	offests: HashMap<u64, Offset>,

	_phantom_data: PhantomData<E>
}

impl<E: Copy + Into<u64>> Save<E> {
	/// Creates new Save struct.
	pub fn new(name: &str) -> Self {
		Self {
			name: name.to_owned(),
			file: None,
			offests: HashMap::new(),

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
		let offset = self.get_file_mut().push(&bytes);

		/* Saving offset of value */
		self.save_offset(enumerator, offset);

		return self
	}

	/// Reads enum-named value from file.
	pub fn read<T: ReinterpretFromBytes + StaticSize>(&self, enumerator: E) -> T {
		self.get_file_ref().read_from_stack(self.load_offset(enumerator))
	}

	/// Writes enum-named array of values to file.
	pub fn array<'t, T: 't, F>(mut self, length: usize, enumerator: E, elem: F) -> Self
	where
		T: ReinterpretAsBytes + StaticSize,
		F: Fn(usize) -> &'t T,
	{
		/* Write the array length and get offset */
		let offset = self.get_file_mut().push(&(length as Offset).reinterpret_as_bytes());

		/* Save offset of an array */
		self.save_offset(enumerator, offset);

		/* Write all elements to file */
		for i in 0..length {
			let bytes = elem(i).reinterpret_as_bytes();
			self.get_file_mut().push(&bytes);
		}

		return self
	}

	/// Reads enum-named array of values from file.
	pub fn read_array<T: ReinterpretFromBytes + StaticSize>(&self, enumerator: E) -> Vec<T> {
		/* Getting offset of array length */
		let offset = self.load_offset(enumerator);

		/* Read array length */
		let length: Offset = self.get_file_ref().read_from_stack(offset);

		/* Actual array offset */
		let offset = offset + Offset::static_size() as Offset;

		/* Loading buffer */
		let mut buffer = vec![0; T::static_size()];

		/* Resulting collection */
		let mut result = Vec::<T>::with_capacity(length as usize);

		/* Read all elements to `result` */
		for i in 0..length {
			/* Read to buffer */
			self.get_file_ref().stack.seek_read(&mut buffer, offset + i * T::static_size() as Offset).unwrap();

			/* Push value to `result` */
			result.push(T::reinterpret_from_bytes(&buffer));
		}

		return result
	}

	/// Allocates data on heap of file with pointer on stack.
	pub fn pointer<T: ReinterpretAsBytes + StaticSize>(mut self, value: &T, enumerator: E) -> Self {
		/* Allocate bytes */
		let bytes = value.reinterpret_as_bytes();
		let (offset, _) = self.get_file_mut().alloc(&bytes);

		/* Save offset */
		self.save_offset(enumerator, offset);

		return self
	}

	/// Reads data from heap by pointer on stack.
	pub fn read_from_pointer<T: ReinterpretFromBytes + StaticSize>(&self, enumerator: E) -> T {
		self.get_file_ref().read_from_heap(self.load_offset(enumerator))
	}

	/// Saves offset.
	fn save_offset(&mut self, enumerator: E, offset: Offset) {
		match self.offests.insert(enumerator.into(), offset) {
			None => (),
			Some(_) => panic!("Trying to write same data to another place")
		}
	}

	/// Loads offset.
	fn load_offset(&self, enumerator: E) -> Offset {
		*self.offests
			.get(&enumerator.into())
			.expect(format!(
				"There is no data enumerated by {}",
				enumerator.into()
			).as_str())
	}

	/// Saves the save.
	pub fn save(self) -> std::io::Result<Self> {
		self.get_file_ref().sync()?;
		return Ok(self)
	}

	/// Gives reference to file if it initialized.
	fn get_file_ref(&self) -> &StackHeap {
		self.file.as_ref().expect("File had not created! Consider call .create() method on Save.")
	}

	/// Gives mutable reference to file if it initialized.
	fn get_file_mut(&mut self) -> &mut StackHeap {
		self.file.as_mut().expect("File had not created! Consider call .create() method on Save.")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::app::utils::math::prelude::*;

	#[derive(Clone, Copy)]
	enum DataType {
		Position,
		Array,
		Pointer,
	}

	impl Into<Offset> for DataType {
		fn into(self) -> Offset { self as Offset }
	}

	#[test]
	fn test() {
		let array_before = [2, 3, 5, 1];
		let pos_before = 123;
		let ptr_before = Int3::new(34, 1, 5);

		let save = Save::new("Test")
			.create("")
			.write(&pos_before, DataType::Position)
			.array(array_before.len(), DataType::Array, |i| &array_before[i])
			.pointer(&ptr_before, DataType::Pointer)
			.save().unwrap();

		let pos_after: i32 = save.read(DataType::Position);
		let array_after: Vec<i32> = save.read_array(DataType::Array);
		let ptr_after: Int3 = save.read_from_pointer(DataType::Pointer);

		assert_eq!(pos_before, pos_after);
		assert_eq!(array_before[..], array_after[..]);
		assert_eq!(ptr_before, ptr_after);
	}
}