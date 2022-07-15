pub mod stack_heap;

use std::{
	marker::PhantomData, 
	collections::HashMap,
	os::windows::prelude::FileExt,
	fs::File,
};

use stack_heap::StackHeap;
use crate::app::utils::{
	reinterpreter::{
		ReinterpretAsBytes,
		ReinterpretFromBytes,
		StaticSize
	},
};

pub type Offset = u64;
pub type Size   = u64;
pub type Enumerator = u64;

/// Handle for save files framework.
pub struct Save<E> {
	name: String,
	file: Option<StackHeap>,
	offests: HashMap<Enumerator, Offset>,
	offsets_save: Option<File>,

	_phantom_data: PhantomData<E>
}

impl<E: Copy + Into<Enumerator>> Save<E> {
	/// Creates new Save struct.
	pub fn new(name: &str) -> Self {
		Self {
			name: name.to_owned(),
			file: None,
			offests: HashMap::new(),
			offsets_save: None,

			_phantom_data: PhantomData
		}
	}

	/// Gives meta file path from given directory.
	fn get_meta_path(path: &str) -> String {
		format!("{path}/meta.off")
	}

	/// Creates heap-stack folder.
	pub fn create(mut self, path: &str) -> Self {
		self.file = Some(StackHeap::new(path, &self.name));
		self.offsets_save = Some(File::create(Self::get_meta_path(path).as_str()).unwrap());

		return self
	}

	/// Opens heap-stack folder.
	pub fn open(mut self, path: &str) -> Self {
		self.file = Some(StackHeap::new(path, &self.name));
		self.offsets_save = Some(File::open(Self::get_meta_path(path).as_str()).unwrap());

		/* Offsets save shortcut */
		let offsets_save = self.offsets_save.as_ref().unwrap();

		/* Read number of offsets */
		let n_offsets = {
			let mut buffer = vec![0; Size::static_size()];
			offsets_save.seek_read(&mut buffer, 0).unwrap();
			Size::reinterpret_from_bytes(&buffer)
		};

		/* Read all offsets to HashMap */
		let offset_size = Offset::static_size() as Size;
		let mut buffer = vec![0; Enumerator::static_size()];
		for i in (1..).step_by(2).take(n_offsets as usize) {
			let enumerator = {
				offsets_save.seek_read(&mut buffer, offset_size * i).unwrap();
				Enumerator::reinterpret_from_bytes(&buffer)
			};
			let offset = {
				offsets_save.seek_read(&mut buffer, offset_size * (i + 1)).unwrap();
				Offset::reinterpret_from_bytes(&buffer)
			};
			self.offests.insert(enumerator, offset);
		}

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
	pub fn array<'t, T: 't, F>(mut self, length: usize, enumerator: E, mut elem: F) -> Self
	where
		T: ReinterpretAsBytes + StaticSize,
		F: FnMut(usize) -> &'t T,
	{
		/* Write the array length and get offset */
		let offset = self.get_file_mut().push(&(length as Size).reinterpret_as_bytes());

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
		let length: Size = self.get_file_ref().read_from_stack(offset);

		/* Actual array offset */
		let offset = offset + Offset::static_size() as Size;

		/* Loading buffer */
		let mut buffer = vec![0; T::static_size()];

		/* Resulting collection */
		let mut result = Vec::<T>::with_capacity(length as usize);

		/* Read all elements to `result` */
		for i in 0..length {
			/* Read to buffer */
			self.get_file_ref().stack.seek_read(&mut buffer, offset + i * T::static_size() as Size).unwrap();

			/* Push value to `result` */
			result.push(T::reinterpret_from_bytes(&buffer));
		}

		return result
	}

	/// Allocates data on heap of file with pointer on stack.
	#[allow(dead_code)]
	pub fn pointer<T: ReinterpretAsBytes + StaticSize>(mut self, value: &T, enumerator: E) -> Self {
		/* Allocate bytes */
		let bytes = value.reinterpret_as_bytes();
		let offset = {
			let alloc = self.get_file_mut().alloc(bytes.len() as Size);
			self.get_file_ref().write_to_heap(alloc, &bytes);
			alloc.get_stack_offset()
		};

		/* Save offset */
		self.save_offset(enumerator, offset);

		return self
	}

	/// Reads data from heap by pointer on stack.
	#[allow(dead_code)]
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
		/* Sync all changes to file */
		self.get_file_ref().sync()?;

		/* Get shortcut to offsets_save */
		let offsets_save = self.offsets_save.as_ref().unwrap();

		/* Save offsets length to `meta.off` file */
		let n_offsets = self.offests.len() as Size;
		offsets_save.seek_write(&n_offsets.reinterpret_as_bytes(), 0).unwrap();

		/* Save all offsets to `meta.off` file */
		let offset_size = Offset::static_size() as Size;
		for ((&enumerator, &offset), i) in self.offests.iter().zip((1_u64..).step_by(2)) {
			offsets_save.seek_write(&enumerator.reinterpret_as_bytes(), offset_size * i).unwrap();
			offsets_save.seek_write(&offset.reinterpret_as_bytes(), offset_size * (i + 1)).unwrap();
		}

		/* Sync all changes to file */
		offsets_save.sync_all().unwrap();

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