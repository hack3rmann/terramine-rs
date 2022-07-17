pub mod stack_heap;

use std::{
	marker::PhantomData, 
	collections::HashMap,
	os::windows::prelude::FileExt,
	fs::{File, OpenOptions},
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
		self.offsets_save = Some(
			OpenOptions::new()
				.read(true)
				.write(true)
				.create(false)
				.open(Self::get_meta_path(path).as_str())
				.unwrap()
		);

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
		self.store_offset(enumerator, offset);

		return self
	}

	/// Assigns value to stack
	pub fn assign<T: ReinterpretAsBytes + StaticSize>(&mut self, value: &T, enumerator: E) {
		/* Get bytes and offset */
		let bytes = value.reinterpret_as_bytes();
		let offset = self.load_offset(enumerator);

		/* Assign new data to stack */
		self.get_file_mut().write_to_stack(offset, &bytes);
	}

	/// Reads enum-named value from file.
	pub fn read<T: ReinterpretFromBytes + StaticSize>(&self, enumerator: E) -> T {
		self.get_file_ref().read_from_stack(self.load_offset(enumerator))
	}

	/// Writes enum-named array of values to file.
	#[allow(dead_code)]
	pub fn array<'t, T: 't, F>(mut self, length: usize, enumerator: E, mut elem: F) -> Self
	where
		T: ReinterpretAsBytes + StaticSize,
		F: FnMut(usize) -> &'t T,
	{
		/* Write the array length and get offset */
		let offset = self.get_file_mut().push(&(length as Size).reinterpret_as_bytes());

		/* Save offset of an array */
		self.store_offset(enumerator, offset);

		/* Write all elements to file */
		for i in 0..length {
			let bytes = elem(i).reinterpret_as_bytes();
			self.get_file_mut().push(&bytes);
		}

		return self
	}

	/// Assignes to an array
	pub fn assign_array<'t, T: 't, F>(&mut self, enumerator: E, mut elem: F)
	where
		T: ReinterpretAsBytes + StaticSize,
		F: FnMut(usize) -> &'t T,
	{
		/* Load offset */
		let offset = self.load_offset(enumerator);

		/* Read length of the data */
		let length: Size = self.get_file_ref().read_from_stack(offset);

		/* Iterate all elements and assign to them offsets */
		let elements = (0..).map(|i| elem(i));
		let offsets = (0..).map(|i| offset + i * T::static_size() as Size + Size::static_size() as Size);
		
		/* Write them to file */
		for (elem, offset) in elements.zip(offsets).take(length as usize) {
			self.get_file_mut().write_to_stack(offset, &elem.reinterpret_as_bytes());
		}
	}

	/// Assings to some element in the array
	pub fn assign_array_element<T: ReinterpretAsBytes + StaticSize>(&mut self, enumerator: E, elem: T, idx: usize) {
		/* Load offset */
		let offset = self.load_offset(enumerator);

		/* Read length of an array */
		let length: Size = self.get_file_ref().read_from_stack(offset);

		/* Test if given index passed into length */
		assert!((idx as Offset) < length, "Given index ({}) should be in 0..{}", idx, length);

		/* Get offset of an element */
		let offset = offset + idx as Offset * T::static_size() as Size + Size::static_size() as Size;

		/* Write element */
		self.get_file_mut().write_to_stack(offset, &elem.reinterpret_as_bytes());
	}

	/// Reads enum-named array of values from file.
	#[allow(dead_code)]
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

	/// Reads element of an array
	pub fn read_array_element<T: ReinterpretFromBytes + StaticSize>(&self, enumerator: E, idx: usize) -> T {
		/* Load offset */
		let offset = self.load_offset(enumerator);

		/* Read length */
		let length: Size = self.get_file_ref().read_from_stack(offset);

		/* Test if index if off available range */
		assert!((idx as Offset) < length, "Given index ({}) should be in 0..{}", idx, length);

		/* Get data offset */
		let offset = offset + idx as Offset * T::static_size() as Size + Size::static_size() as Size;

		/* Read element */
		self.get_file_ref().read_from_stack(offset)
	}

	/// Allocates data on heap of file with pointer on stack.
	#[allow(dead_code)]
	pub fn pointer(mut self, bytes: Vec<u8>, enumerator: E) -> Self {
		/* Allocate bytes */
		let offset = {
			let alloc = self.get_file_mut().alloc(bytes.len() as Size);
			self.get_file_ref().write_to_heap(alloc, &bytes);
			alloc.get_stack_offset()
		};

		/* Save offset */
		self.store_offset(enumerator, offset);

		return self
	}

	/// Replaces pointer data with new data.
	#[allow(dead_code)]
	pub fn assign_to_pointer(&mut self, bytes: Vec<u8>, enumerator: E) {
		/* Load offset */
		let stack_offset = self.load_offset(enumerator);

		/* Allocate new data */
		let alloc = self.get_file_mut().realloc(bytes.len() as Size, stack_offset);
		self.get_file_ref().write_to_heap(alloc, &bytes);
	}

	/// Reads data from heap by pointer on stack.
	#[allow(dead_code)]
	pub fn read_from_pointer<T, F: FnOnce(&[u8]) -> T>(&self, enumerator: E, item: F) -> T {
		/* Load offsets */
		let stack_offset = self.load_offset(enumerator);
		let heap_offset: Offset = self.get_file_ref().read_from_stack(stack_offset);

		/* Read data */
		let bytes = self.get_file_ref().read_from_heap(heap_offset);
		item(&bytes)
	}

	/// Allocates array of pinters on stack and array of data on heap.
	pub fn pointer_array<F: FnMut(usize) -> Vec<u8>>(mut self, length: usize, enumerator: E, mut elem: F) -> Self {
		/* Push size to stack and store its offset */
		let stack_offset = self.get_file_mut().push(&(length as Size).reinterpret_as_bytes());
		self.store_offset(enumerator, stack_offset);

		/* Write all elements to heap */
		for data in (0..length).map(|i| elem(i)) {
			let alloc = self.get_file_mut().alloc(data.len() as Size);
			self.get_file_ref().write_to_heap(alloc, &data);
		}

		return self
	}

	/// Assigns array of pointers elements.
	pub fn assign_pointer_array<F: FnMut(usize) -> Vec<u8>>(&mut self, enumerator: E, mut elem: F) {
		/* Load offset */
		let stack_offset = self.load_offset(enumerator);

		/* Read length */
		let length: Size = self.get_file_ref().read_from_stack(stack_offset);

		/* Offsets iterator */
		let offsets = (1..).map(|i| stack_offset + i * Offset::static_size() as Size);

		/* Elements iterator */
		let elements = (0..).map(|i| elem(i));

		/* Write bytes */
		for (bytes, offset) in elements.zip(offsets).take(length as usize) {
			let alloc = self.get_file_mut().realloc(bytes.len() as Size, offset); // !PANIC!
			self.get_file_ref().write_to_heap(alloc, &bytes);
		}
	}

	/// Assigns new element to an index of pointer array.
	pub fn assign_pointer_array_element(&mut self, enumerator: E, bytes: Vec<u8>, idx: usize) {
		/* Load offset */
		let offset = self.load_offset(enumerator);

		/* Read length of an array */
		let length: Size = self.get_file_ref().read_from_stack(offset);

		/* Test if index if off available range */
		assert!((idx as Offset) < length, "Given index ({}) should be in 0..{}", idx, length);

		/* Calculate offset on stack */
		let offset = offset + (idx + 1) as Offset * Offset::static_size() as Size;

		/* Rewrite data */
		let alloc = self.get_file_mut().realloc(bytes.len() as Size, offset);
		self.get_file_ref().write_to_heap(alloc, &bytes);
	}

	/// Reads array of pointers from stack and data from heap.
	pub fn read_pointer_array<T, F>(&self, enumerator: E, mut elem: F) -> Vec<T>
	where
		F: FnMut(&[u8]) -> T
	{
		/* Load stack data offset */
		let length_offset = self.load_offset(enumerator);

		/* Read array length */
		let length: Size = self.get_file_ref().read_from_stack(length_offset);

		/* Resulting vector */
		let mut result = Vec::with_capacity(length as usize);

		/* Read all elements */
		let offset_size = Size::static_size() as Size;
		for i in 1 .. length + 1 {
			/* Read offset on heap */
			let heap_offset: Offset = self.get_file_ref().read_from_stack(length_offset + i * offset_size);

			/* Read data bytes */
			let bytes = self.get_file_ref().read_from_heap(heap_offset);

			/* Reinterpret them and push to result */
			result.push(elem(&bytes));
		}

		return result
	}

	/// Saves offset.
	fn store_offset(&mut self, enumerator: E, offset: Offset) {
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
		HardData
	}

	impl Into<Offset> for DataType {
		fn into(self) -> Offset { self as Offset }
	}

	#[test]
	fn test1() {
		let pos_before = Float4::new(124.5, 124.6, 9912.5, 1145.678);
		let array_before = [2, 3, 5, 1];
		let ptr_start = Int3::new(128, 1, 5);
		let ptr_before = Float4::new(141.4, 12441.5, 1451.511, 1151.63);
		let hard_data_before = [23_i32, 1, 1, 4, 6, 1];

		use DataType::*;
		let mut save = Save::new("Test")
			.create("test")
			.write(&pos_before, Position)
			.array(array_before.len(), Array, |i| &array_before[i])
			.pointer(ptr_start.reinterpret_as_bytes(), Pointer)
			.pointer_array(hard_data_before.len(), HardData, |i| {
				let condition = hard_data_before[i] % 2 == 0;
				let num = if condition {
					hard_data_before[i] / 2
				} else { hard_data_before[i] };

				let mut bytes = (condition as u8).reinterpret_as_bytes();
				bytes.append(&mut num.reinterpret_as_bytes());

				bytes
			})
			.save().unwrap();
		save.assign_to_pointer(ptr_before.reinterpret_as_bytes(), Pointer);
		let save = Save::new("Test").open("test");

		let pos_after: Float4 = save.read(DataType::Position);
		let array_after: Vec<i32> = save.read_array(DataType::Array);
		let ptr_after = save.read_from_pointer(DataType::Pointer, |bytes| Float4::reinterpret_from_bytes(bytes));
		let hard_data_after: Vec<i32> = save.read_pointer_array(HardData, |bytes| {
			let condition = bytes[0] != 0;
			let num = i32::reinterpret_from_bytes(&bytes[1..]);

			if condition { num * 2 } else { num }
		});

		assert_eq!(pos_before, pos_after);
		assert_eq!(array_before[..], array_after[..]);
		assert_eq!(ptr_before, ptr_after);
		assert_eq!(hard_data_before[..], hard_data_after[..]);
	}

	#[derive(Clone, Copy)]
	enum Enumerator2 {
		Data
	}

	impl Into<Offset> for Enumerator2 {
		fn into(self) -> Offset { self as Offset }
	}

	#[test]
	fn test_assign() {
		let data_before = Int3::new(213, 56, 123);
		let data_expected = Int3::new(213, 28, 123);

		use Enumerator2::*;
		let mut save = Save::new("Test2")
			.create("test")
			.write(&data_before, Data)
			.save().unwrap();
		save.assign(&data_expected, Data);

		let save = Save::new("Test2").open("test");
		let data_after: Int3 = save.read(Data);

		assert_eq!(data_expected, data_after);
	}

	#[test]
	fn test_assign_array() {
		let data_before = [1234_i32, 134, 134, 1455, 41];
		let data_expected = [13441_i32, 1441888, 14, 313, 144];

		use Enumerator2::*;
		let mut save = Save::new("Test2")
			.create("test")
			.array(data_before.len(), Data, |i| &data_before[i])
			.save().unwrap();
		save.assign_array(Data, |i| &data_expected[i]);

		let save = Save::new("Test2").open("test");
		let data_after: Vec<i32> = save.read_array(Data);

		assert_eq!(data_expected[..], data_after[..]);
	}

	#[test]
	fn test_assign_array_element() {
		let data_before =   [1234_i32, 134, 134, 1455, 41];
		let data_expected = [1234_i32, 134, 999, 1455, 41];

		use Enumerator2::*;
		let mut save = Save::new("Test2")
			.create("test")
			.array(data_before.len(), Data, |i| &data_before[i])
			.save().unwrap();
		save.assign_array_element(Data, 999, 2);

		let save = Save::new("Test2").open("test");
		let data_after: Vec<i32> = save.read_array(Data);

		assert_eq!(data_expected[..], data_after[..]);
	}

	#[test]
	fn test_read_array_element() {
		let data_before = [1234_i32, 134, 134, 1455, 41];

		use Enumerator2::*;
		Save::new("Test2")
			.create("test")
			.array(data_before.len(), Data, |i| &data_before[i])
			.save().unwrap();

		let save = Save::new("Test2").open("test");
		let mut data_after = Vec::with_capacity(data_before.len());

		for num in (0..).map(|i| -> i32 { save.read_array_element(Data, i) }).take(data_before.len()) {
			data_after.push(num)
		}

		assert_eq!(data_before[..], data_after[..]);
	}

	#[test]
	fn test_assign_pointer_array() {
		let data_before = [1234_i32, 134, 134, 1455, 41];
		let data_expected = [13441_i32, 1441888, 14, 313, 144];

		use Enumerator2::*;
		let mut save = Save::new("Test2")
			.create("test")
			.pointer_array(data_before.len(), Data, |i| data_expected[i].reinterpret_as_bytes())
			.save().unwrap();
		save.assign_pointer_array(Data, |i| data_expected[i].reinterpret_as_bytes());

		let save = Save::new("Test2").open("test");
		let data_after = save.read_pointer_array(Data, |bytes| i32::reinterpret_from_bytes(bytes));

		assert_eq!(data_expected[..], data_after[..]);
	}

	#[test]
	fn test_assign_pointer_array_element() {
		let data_before   = [1234_i32, 134, 134, 1455, 41];
		let data_expected = [1234_i32, 134, 999, 1455, 41];

		use Enumerator2::*;
		let mut save = Save::new("Test2")
			.create("test")
			.pointer_array(data_before.len(), Data, |i| data_before[i].reinterpret_as_bytes())
			.save().unwrap();
		save.assign_pointer_array_element(Data, 999.reinterpret_as_bytes(), 2);

		let save = Save::new("Test2").open("test");
		let data_after = save.read_pointer_array(Data, |bytes| i32::reinterpret_from_bytes(bytes));

		assert_eq!(data_expected[..], data_after[..]);
	}
}