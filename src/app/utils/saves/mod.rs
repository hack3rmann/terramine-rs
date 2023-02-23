pub mod stack_heap;

use {
    crate::app::utils::{
        cfg::save::META_FILE_NAME,
        werror::prelude::*,
        reinterpreter::{
            ReinterpretAsBytes,
            ReinterpretFromBytes,
            StaticSize
        },
    },
    std::{
        marker::PhantomData, 
        collections::HashMap,
        os::windows::prelude::FileExt,
        fs::{File, OpenOptions},
    },
    stack_heap::{StackHeap, ShResult, ShError},
};

pub type Offset = u64;
pub type Size   = u64;
pub type Enumerator = u64;

/// Handle for save files framework.
#[derive(Debug)]
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
        format!("{path}/{META_FILE_NAME}")
    }

    /// Creates heap-stack folder.
    pub fn create(mut self, path: &str) -> Self {
        self.file = Some(StackHeap::new(path, &self.name));
        self.offsets_save = Some(File::create(Self::get_meta_path(path).as_str()).wunwrap());

        return self
    }

    /// Opens heap-stack folder. And reads all offsets from save [`META_FILE_NAME`].
    pub fn open(mut self, path: &str) -> Self {
        self.file = Some(StackHeap::new(path, &self.name));
        self.offsets_save = Some(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .open(Self::get_meta_path(path).as_str())
                .wunwrap()
        );

        /* Offsets save shortcut */
        let offsets_save = self.offsets_save.as_ref().wunwrap();

        /* Read number of offsets */
        let n_offsets = {
            let mut buffer = vec![0; Size::static_size()];
            offsets_save.seek_read(&mut buffer, 0).wunwrap();
            Size::reinterpret_from_bytes(&buffer)
        };

        /* Read all offsets to HashMap */
        let offset_size = Offset::static_size() as Size;
        let mut buffer = vec![0; Enumerator::static_size()];
        for i in (1..).step_by(2).take(n_offsets as usize) {
            let enumerator = {
                offsets_save.seek_read(&mut buffer, offset_size * i).wunwrap();
                Enumerator::reinterpret_from_bytes(&buffer)
            };
            let offset = {
                offsets_save.seek_read(&mut buffer, offset_size * (i + 1)).wunwrap();
                Offset::reinterpret_from_bytes(&buffer)
            };
            self.offests.insert(enumerator, offset);
        }

        return self
    }

    /// Writes enum-named value to stack file.
    pub fn write<T: ReinterpretAsBytes + StaticSize>(mut self, value: &T, enumerator: E) -> Self {
        /* Write value to file stack */
        let bytes = value.reinterpret_as_bytes();
        let offset = self.get_file_mut().push(&bytes);

        /* Saving offset of value */
        self.store_offset(enumerator, offset).wunwrap();

        return self
    }

    /// Assigns enum-named value to value on stack.
    #[allow(dead_code)]
    pub fn assign<T: ReinterpretAsBytes + StaticSize>(&mut self, value: &T, enumerator: E) {
        /* Get bytes and offset */
        let bytes = value.reinterpret_as_bytes();
        let offset = self.load_offset(enumerator);

        /* Assign new data to stack */
        self.get_file_mut().write_to_stack(offset, &bytes);
    }

    /// Reads enum-named value from stack file.
    pub fn read<T: ReinterpretFromBytes + StaticSize>(&self, enumerator: E) -> T {
        self.get_file_ref().read_from_stack(self.load_offset(enumerator))
    }

    /// Writes enum-named array of values to stack file.
    #[allow(dead_code)]
    pub fn array<'t, T: 't, F>(mut self, length: usize, enumerator: E, mut elem: F) -> Self
    where
        T: ReinterpretAsBytes + StaticSize,
        F: FnMut(usize) -> &'t T,
    {
        /* Write the array length and get offset */
        let offset = self.get_file_mut().push(&(length as Size).reinterpret_as_bytes());

        /* Save offset of an array */
        self.store_offset(enumerator, offset).wunwrap();

        /* Write all elements to file */
        for i in 0..length {
            let bytes = elem(i).reinterpret_as_bytes();
            self.get_file_mut().push(&bytes);
        }

        return self
    }

    /// Assignes new values to enum-named array.
    #[allow(dead_code)]
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

    /// Assings new value to some element in enum-named array.
    #[allow(dead_code)]
    pub fn assign_array_element<T: ReinterpretAsBytes + StaticSize>(&mut self, enumerator: E, elem: T, idx: usize) -> ShResult<()> {
        /* Load offset */
        let offset = self.load_offset(enumerator);

        /* Read length of an array */
        let length: Size = self.get_file_ref().read_from_stack(offset);

        /* Test if given index passed into length */
        Self::test_index(idx as Offset, length)?;

        /* Get offset of an element */
        let offset = offset + idx as Offset * T::static_size() as Size + Size::static_size() as Size;

        /* Write element */
        self.get_file_mut().write_to_stack(offset, &elem.reinterpret_as_bytes());

        Ok(())
    }

    /// Reads enum-named array of values from stack file.
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
            self.get_file_ref().stack.seek_read(&mut buffer, offset + i * T::static_size() as Size).wunwrap();

            /* Push value to `result` */
            result.push(T::reinterpret_from_bytes(&buffer));
        }

        return result
    }

    /// Reads element of enum-named array on stack by index `idx`.
    #[allow(dead_code)]
    pub fn read_array_element<T: ReinterpretFromBytes + StaticSize>(&self, enumerator: E, idx: usize) -> ShResult<T> {
        /* Load offset */
        let offset = self.load_offset(enumerator);

        /* Read length */
        let length: Size = self.get_file_ref().read_from_stack(offset);

        /* Test if index if off available range */
        Self::test_index(idx as Offset, length)?;

        /* Get data offset */
        let offset = offset + idx as Offset * T::static_size() as Size + Size::static_size() as Size;

        /* Read element */
        Ok(self.get_file_ref().read_from_stack(offset))
    }

    /// Allocates data on heap of file with pointer on stack and writes all given bytes.
    #[allow(dead_code)]
    pub fn pointer(mut self, bytes: Vec<u8>, enumerator: E) -> Self {
        /* Allocate bytes */
        let offset = {
            let alloc = self.get_file_mut().alloc(bytes.len() as Size);
            self.get_file_ref().write_to_heap(alloc, &bytes).wunwrap();
            alloc.get_stack_offset()
        };

        /* Save offset */
        self.store_offset(enumerator, offset).wunwrap();

        return self
    }

    /// Assignes new data to the value of pointer to heap.
    #[allow(dead_code)]
    pub fn assign_to_pointer(&mut self, bytes: Vec<u8>, enumerator: E) {
        /* Load offset */
        let stack_offset = self.load_offset(enumerator);

        /* Allocate new data */
        let alloc = self.get_file_mut().realloc(bytes.len() as Size, stack_offset);
        self.get_file_ref().write_to_heap(alloc, &bytes).wunwrap();
    }

    /// Reads data from heap by stack pointer to heap on.
    #[allow(dead_code)]
    pub fn read_from_pointer<T, F: FnOnce(&[u8]) -> T>(&self, enumerator: E, item: F) -> T {
        /* Load offsets */
        let stack_offset = self.load_offset(enumerator);
        let heap_offset: Offset = self.get_file_ref().read_from_stack(stack_offset);

        /* Read data */
        let bytes = self.get_file_ref().read_from_heap(heap_offset);
        item(&bytes)
    }

    /// Allocates an array of pinters on stack and array of data on heap.
    pub fn pointer_array<F: FnMut(usize) -> Vec<u8>>(mut self, length: usize, enumerator: E, mut elem: F) -> Self {
        /* Push size to stack and store its offset */
        let stack_offset = self.get_file_mut().push(&(length as Size).reinterpret_as_bytes());
        self.store_offset(enumerator, stack_offset).wunwrap();

        /* Write all elements to heap */
        for data in (0..length).map(|i| elem(i)) {
            let alloc = self.get_file_mut().alloc(data.len() as Size);
            self.get_file_ref().write_to_heap(alloc, &data).wunwrap();
        }

        return self
    }

    /// Assigns new array of pointers to existed one.
    #[allow(dead_code)]
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
            let alloc = self.get_file_mut().realloc(bytes.len() as Size, offset);
            self.get_file_ref().write_to_heap(alloc, &bytes).wunwrap();
        }
    }

    /// Assigns new element to the element by index of pointer on stack.
    #[allow(dead_code)]
    pub fn assign_pointer_array_element(&mut self, enumerator: E, bytes: Vec<u8>, idx: usize) -> ShResult<()> {
        /* Load offset */
        let offset = self.load_offset(enumerator);

        /* Read length of an array */
        let length: Size = self.get_file_ref().read_from_stack(offset);

        /* Test if index if off available range */
        Self::test_index(idx as Offset, length)?;

        /* Calculate offset on stack */
        let offset = offset + (idx + 1) as Offset * Offset::static_size() as Size;

        /* Rewrite data */
        let alloc = self.get_file_mut().realloc(bytes.len() as Size, offset);
        self.get_file_ref().write_to_heap(alloc, &bytes).wunwrap();

        Ok(())
    }

    /// Reads an array of data from heap.
    pub fn read_pointer_array<T, F>(&self, enumerator: E, mut elem: F) -> Vec<T>
    where
        F: FnMut(usize, &[u8]) -> T
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
            result.push(elem(i as usize - 1, &bytes));
        }

        return result
    }

    /// Reads a pointer array element at index `idx`.
    #[allow(dead_code)]
    pub fn read_pointer_array_element<T, F>(&self, enumerator: E, idx: usize, elem: F) -> ShResult<T>
    where
        F: FnOnce(&[u8]) -> T
    {
        /* Load offset */
        let offset = self.load_offset(enumerator);

        /* Read length */
        let length: Size = self.get_file_ref().read_from_stack(offset);

        /* Test if index if off available range */
        Self::test_index(idx as Offset, length)?;

        /* Calculate actual offsets */
        let offset = offset + Size::static_size() as Size + idx as Offset * Offset::static_size() as Size;
        let heap_offset = self.get_file_ref().read_from_stack(offset);

        /* Read data bytes */
        let bytes = self.get_file_ref().read_from_heap(heap_offset);

        /* Return data */
        Ok(elem(&bytes))
    }

    /// Saves offset by enumerator.
    fn store_offset(&mut self, enumerator: E, offset: Offset) -> ShResult<()> {
        match self.offests.insert(enumerator.into(), offset) {
            None => Ok(()),
            Some(old) => Err(ShError(format!(
                "Trying to write same key of some data to another place. Old data: {}.",
                old
            )))
        }
    }

    /// Loads offset by enumerator.
    fn load_offset(&self, enumerator: E) -> Offset {
        *self.offests
            .get(&enumerator.into())
            .wexpect(format!(
                "There is no data enumerated by {}",
                enumerator.into()
            ).as_str())
    }

    /// Saves the save.
    pub fn save(self) -> std::io::Result<Self> {
        /* Sync all changes to file */
        self.get_file_ref().sync()?;

        /* Get shortcut to offsets_save */
        let offsets_save = self.offsets_save.as_ref().wunwrap();

        /* Save offsets length to `meta.off` file */
        let n_offsets = self.offests.len() as Size;
        offsets_save.seek_write(&n_offsets.reinterpret_as_bytes(), 0).wunwrap();

        /* Save all offsets to `meta.off` file */
        let offset_size = Offset::static_size() as Size;
        for ((&enumerator, &offset), i) in self.offests.iter().zip((1_u64..).step_by(2)) {
            offsets_save.seek_write(&enumerator.reinterpret_as_bytes(), offset_size * i).wunwrap();
            offsets_save.seek_write(&offset.reinterpret_as_bytes(), offset_size * (i + 1)).wunwrap();
        }

        /* Sync all changes to file */
        offsets_save.sync_all().wunwrap();

        return Ok(self)
    }

    /// Gives reference to file if it initialized.
    fn get_file_ref(&self) -> &StackHeap {
        self.file.as_ref().wexpect("File had not created! Consider call .create() method on Save.")
    }

    /// Gives mutable reference to file if it initialized.
    fn get_file_mut(&mut self) -> &mut StackHeap {
        self.file.as_mut().wexpect("File had not created! Consider call .create() method on Save.")
    }

    /// Test if given index is valid.
    fn test_index(idx: Offset, len: Size) -> ShResult<()> {
        if idx < len {
            return Err(ShError(format!("Given index ({}) should be in 0..{}", idx, len)))
        } else { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::utils::math::prelude::*;

    #[derive(Clone, Copy, Debug)]
    enum DataType {
        Position,
        Array,
        Pointer,
        HardData
    }

    impl Into<Offset> for DataType {
        fn into(self) -> Offset { self as Offset }
    }

    #[derive(Clone, Copy, Debug)]
    enum MinorTestDataType { Data }

    impl Into<Offset> for MinorTestDataType {
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
        let mut save = Save::new("Test1")
            .create("test1")
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
            .save().wunwrap();
        save.assign_to_pointer(ptr_before.reinterpret_as_bytes(), Pointer);
        let save = Save::new("Test1").open("test1");

        let pos_after: Float4 = save.read(DataType::Position);
        let array_after: Vec<i32> = save.read_array(DataType::Array);
        let ptr_after = save.read_from_pointer(DataType::Pointer, |bytes| Float4::reinterpret_from_bytes(bytes));
        let hard_data_after: Vec<i32> = save.read_pointer_array(HardData, |_, bytes| {
            let condition = bytes[0] != 0;
            let num = i32::reinterpret_from_bytes(&bytes[1..]);

            if condition { num * 2 } else { num }
        });

        assert_eq!(pos_before, pos_after);
        assert_eq!(array_before[..], array_after[..]);
        assert_eq!(ptr_before, ptr_after);
        assert_eq!(hard_data_before[..], hard_data_after[..]);
    }

    #[test]
    fn test_assign() {
        let data_before = Int3::new(213, 56, 123);
        let data_expected = Int3::new(213, 28, 123);

        use MinorTestDataType::*;
        let mut save = Save::new("Test2")
            .create("test2")
            .write(&data_before, Data)
            .save().wunwrap();
        save.assign(&data_expected, Data);

        let save = Save::new("Test2").open("test2");
        let data_after: Int3 = save.read(Data);

        assert_eq!(data_expected, data_after);
    }

    #[test]
    fn test_assign_array() {
        let data_before = [1234_i32, 134, 134, 1455, 41];
        let data_expected = [13441_i32, 1441888, 14, 313, 144];

        use MinorTestDataType::*;
        let mut save = Save::new("Test3")
            .create("test3")
            .array(data_before.len(), Data, |i| &data_before[i])
            .save().wunwrap();
        save.assign_array(Data, |i| &data_expected[i]);

        let save = Save::new("Test3").open("test3");
        let data_after: Vec<i32> = save.read_array(Data);

        assert_eq!(data_expected[..], data_after[..]);
    }

    #[test]
    fn test_assign_array_element() {
        let data_before =   [1234_i32, 134, 134, 1455, 41];
        let data_expected = [1234_i32, 134, 999, 1455, 41];

        use MinorTestDataType::*;
        let mut save = Save::new("Test4")
            .create("test4")
            .array(data_before.len(), Data, |i| &data_before[i])
            .save().wunwrap();
        save.assign_array_element(Data, 999, 2).wunwrap();

        let save = Save::new("Test4").open("test4");
        let data_after: Vec<i32> = save.read_array(Data);

        assert_eq!(data_expected[..], data_after[..]);
    }

    #[test]
    fn test_read_array_element() {
        let data_before = [1234_i32, 134, 134, 1455, 41];

        use MinorTestDataType::*;
        Save::new("Test5")
            .create("test5")
            .array(data_before.len(), Data, |i| &data_before[i])
            .save().wunwrap();

        let save = Save::new("Test5").open("test5");
        let mut data_after = Vec::with_capacity(data_before.len());

        for num in (0..).map(|i| -> i32 { save.read_array_element(Data, i).wunwrap() }).take(data_before.len()) {
            data_after.push(num)
        }

        assert_eq!(data_before[..], data_after[..]);
    }

    #[test]
    fn test_assign_pointer_array() {
        let data_before = [1234_i32, 134, 134, 1455, 41];
        let data_expected = [13441_i32, 1441888, 14, 313, 144];

        use MinorTestDataType::*;
        let mut save = Save::new("Test6")
            .create("test6")
            .pointer_array(data_before.len(), Data, |i| data_expected[i].reinterpret_as_bytes())
            .save().wunwrap();
        save.assign_pointer_array(Data, |i| data_expected[i].reinterpret_as_bytes());

        let save = Save::new("Test6").open("test6");
        let data_after = save.read_pointer_array(Data, |_, bytes| i32::reinterpret_from_bytes(bytes));

        assert_eq!(data_expected[..], data_after[..]);
    }

    #[test]
    fn test_assign_pointer_array_element() {
        let data_before   = [1234_i32, 134, 134, 1455, 41];
        let data_expected = [1234_i32, 134, 999, 1455, 41];

        use MinorTestDataType::*;
        let mut save = Save::new("Test7")
            .create("test7")
            .pointer_array(data_before.len(), Data, |i| data_before[i].reinterpret_as_bytes())
            .save().wunwrap();
        save.assign_pointer_array_element(Data, 999.reinterpret_as_bytes(), 2).wunwrap();

        let save = Save::new("Test7").open("test7");
        let data_after = save.read_pointer_array(Data, |_, bytes| i32::reinterpret_from_bytes(bytes));

        assert_eq!(data_expected[..], data_after[..]);
    }

    #[test]
    fn test_read_pointer_array_element() {
        let data_before = [1234_i32, 134, 134, 1455, 41];

        use MinorTestDataType::*;
        Save::new("Test8")
            .create("test8")
            .pointer_array(data_before.len(), Data, |i| data_before[i].reinterpret_as_bytes())
            .save().wunwrap();

        let save = Save::new("Test8").open("test8");
        let mut data_after = Vec::with_capacity(data_before.len());

        let nums = (0..).map(|i|
            save.read_pointer_array_element(Data, i, |bytes| i32::reinterpret_from_bytes(bytes)).wunwrap()
        );

        for num in nums.take(data_before.len()) {
            data_after.push(num)
        }

        assert_eq!(data_before[..], data_after[..]);
    }
}