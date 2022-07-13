#![allow(dead_code)]

pub mod stack_heap;

use std::{marker::PhantomData, collections::HashMap, os::windows::prelude::FileExt};

use super::reinterpreter::{ReinterpretAsBytes, ReinterpretFromBytes, StaticSize};
use stack_heap::StackHeap;

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
		/* Initialyze buffer */
		let mut buffer = vec![0; T::static_size()];

		/* Read value from file */
		self.get_file_ref().stack.seek_read(
			&mut buffer,
			self.load_offset(enumerator)
		).unwrap();

		T::reinterpret_from_bytes(&buffer)
	}

	/// Writes enum-named array of values to file.
	pub fn array<'t, T: 't, F>(mut self, length: usize, enumerator: E, elem: F) -> Self
	where
		T: ReinterpretAsBytes + StaticSize,
		F: Fn(usize) -> &'t T,
	{
		/* Write the array length and get offset */
		let offset = self.get_file_mut().push(&(length as Offset).reinterpret_as_bytes());

		/* Write all elements to file */
		for i in 0..length {
			let bytes = elem(i).reinterpret_as_bytes();
			self.get_file_mut().push(&bytes);
		}

		/* Save offset of an array */
		self.save_offset(enumerator, offset);

		return self
	}

	/// Reads enum-named array of values from file.
	pub fn read_array<T: ReinterpretFromBytes + StaticSize>(&self, enumerator: E) -> Vec<T> {
		/* Getting offset of array length */
		let offset = self.load_offset(enumerator);

		/* Read array length */
		let length = {
			let mut buffer = vec![0; Offset::static_size()];
			self.get_file_ref().stack.seek_read(&mut buffer, offset).unwrap();
			Offset::reinterpret_from_bytes(&buffer)
		};

		/* Actual array offset */
		let offset = offset + Offset::static_size() as u64;

		/* Loading buffer */
		let mut buffer = vec![0; T::static_size()];

		/* Resulting collection */
		let mut result = Vec::<T>::with_capacity(length as usize);

		/* Read all elements to `result` */
		for i in 0..length {
			/* Read to buffer */
			self.get_file_ref().stack.seek_read(&mut buffer, offset + i).unwrap();

			/* Push value to `result` */
			result.push(T::reinterpret_from_bytes(&buffer));
		}

		return result
	}

	/// Saves offset.
	fn save_offset(&mut self, enumerator: E, offset: Offset) {
		self.offests.insert(enumerator.into(), offset).expect("Trying to write same data to another place");
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

	/// Gives reference to file if it initialized.
	fn get_file_ref(&self) -> &StackHeap {
		self.file.as_ref().expect("File had not created! Consider call .create() method on Save.")
	}

	/// Gives mutable reference to file if it initialized.
	fn get_file_mut(&mut self) -> &mut StackHeap {
		self.file.as_mut().expect("File had not created! Consider call .create() method on Save.")
	}
}