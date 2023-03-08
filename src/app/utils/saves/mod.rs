use tokio::io::AsyncSeekExt;

pub mod stack_heap;

use {
    crate::app::utils::{
        cfg::save::META_FILE_NAME,
        reinterpreter::{
            ReinterpretAsBytes,
            ReinterpretFromBytes,
            StaticSize
        },
    },
    std::{
        marker::PhantomData, 
        collections::HashMap,
        future::Future,
    },
    tokio::{
        io::{self, SeekFrom, AsyncReadExt, AsyncWriteExt},
        fs::{File, OpenOptions},
    },
    stack_heap::{StackHeap, StackHeapError},
    thiserror::Error,
};

pub type Offset = u64;
pub type Size   = u64;
pub type Enumerator = u64;

#[derive(Error, Debug)]
pub enum SaveError {
    #[error("io failed: {0}")]
    Io(#[from] io::Error),
    
    #[error("error from `StackHeap`")]
    StackHeap(#[from] StackHeapError),

    #[error("index shpuld be in 0..{size} but {idx}")]
    IndexOutOfBounds {
        idx: Offset,
        size: Size,
    },

    #[error("Trying to write same key of some data to another place. Old enumerator value: {0}.")]
    DataOverride(Enumerator),
}

pub type SaveResult<T> = Result<T, SaveError>;

/// Handle for save files framework.
#[derive(Debug)]
pub struct Save<E> {
    #[allow(dead_code)]
    name: String,

    file: StackHeap,
    offsets: HashMap<Enumerator, Offset>,
    offsets_save: File,

    _phantom_data: PhantomData<E>,
}

#[derive(Debug)]
pub struct SaveBuilder<E> {
    name: String,
    offsets: HashMap<Enumerator, Offset>,

    _phantom_data: PhantomData<E>,
}

impl<E: Copy + Into<Enumerator>> SaveBuilder<E> {
    /// Creates heap-stack folder.
    pub async fn create(self, path: &str) -> io::Result<Save<E>> {
        let file = StackHeap::new(path, &self.name).await?;
        let offsets_save = File::create(
            Save::<E>::get_meta_path(path).as_str()
        ).await?;

        let SaveBuilder { name, offsets, _phantom_data } = self;

        Ok(Save { name, file, offsets, offsets_save, _phantom_data })
    }

    /// Opens heap-stack folder. And reads all offsets from save [`META_FILE_NAME`].
    pub async fn open(mut self, path: &str) -> io::Result<Save<E>> {
        let file = StackHeap::new(path, &self.name).await?;
        let mut offsets_save = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(Save::<E>::get_meta_path(path).as_str())
            .await?;

        /* Read number of offsets */
        let n_offsets = {
            let mut buffer = vec![0; Size::static_size()];

            offsets_save.seek(SeekFrom::Start(0)).await?;
            offsets_save.read(&mut buffer).await?;

            Size::from_bytes(&buffer)
        };

        /* Read all offsets to HashMap */
        let offset_size = Offset::static_size() as Size;
        let mut buffer = vec![0; Enumerator::static_size()];
        for i in (1..).step_by(2).take(n_offsets as usize) {
            let enumerator = {
                offsets_save.seek(SeekFrom::Start(offset_size * i)).await?;
                offsets_save.read(&mut buffer).await?;
                
                Enumerator::from_bytes(&buffer)
            };

            let offset = {
                offsets_save.seek(SeekFrom::Start(offset_size * (i + 1))).await?;
                offsets_save.read(&mut buffer).await?;

                Offset::from_bytes(&buffer)
            };

            self.offsets.insert(enumerator, offset);
        }

        let SaveBuilder { name, offsets, _phantom_data } = self;

        Ok(Save { name, file, offsets, offsets_save, _phantom_data })
    }
}


impl<E: Copy + Into<Enumerator>> Save<E> {
    /// Creates new [`SaveBuilder`] struct.
    pub fn new(name: impl Into<String>) -> SaveBuilder<E> {
        SaveBuilder {
            name: name.into(),
            offsets: HashMap::new(),
            _phantom_data: PhantomData,
        }
    }

    /// Gives meta file path from given directory.
    fn get_meta_path(path: &str) -> String {
        format!("{path}/{META_FILE_NAME}")
    }

    /// Writes enum-named value to stack file.
    pub async fn write<T: ReinterpretAsBytes + StaticSize>(mut self, value: &T, enumerator: E) -> Self {
        /* Write value to file stack */
        let bytes = value.as_bytes();
        let offset = self.file.push(&bytes).await
            .expect("failed to push bytes to file");

        /* Saving offset of value */
        self.store_offset(enumerator, offset)
            .expect("failed to store offset");

        self
    }

    /// Assigns enum-named value to value on stack.
    #[allow(dead_code)]
    pub async fn assign<T: ReinterpretAsBytes + StaticSize>(&mut self, value: &T, enumerator: E) {
        /* Get bytes and offset */
        let bytes = value.as_bytes();
        let offset = self.load_offset(enumerator);

        /* Assign new data to stack */
        self.file.write_to_stack(offset, &bytes)
            .await
            .expect("failed to wtrie to stack");
    }

    /// Reads enum-named value from stack file.
    pub async fn read<T: ReinterpretFromBytes + StaticSize>(&mut self, enumerator: E) -> T {
        self.file.read_from_stack(self.load_offset(enumerator))
            .await
            .expect("failed to read from stack")
    }

    /// Writes enum-named array of values to stack file.
    #[allow(dead_code)]
    pub async fn array<'t, T, F>(mut self, length: usize, enumerator: E, mut elem: F) -> Self
    where
        T: ReinterpretAsBytes + StaticSize + 't,
        F: FnMut(usize) -> &'t T,
    {
        /* Write the array length and get offset */
        let offset = self.file.push(&(length as Size).as_bytes())
            .await
            .expect("failed to push length to file");

        /* Save offset of an array */
        self.store_offset(enumerator, offset)
            .expect("failed to store offset");

        /* Write all elements to file */
        for i in 0..length {
            let bytes = elem(i).as_bytes();

            self.file.push(&bytes)
                .await
                .expect("failed to push bytes to file");
        }

        self
    }

    /// Assignes new values to enum-named array.
    #[allow(dead_code)]
    pub async fn assign_array<'t, T: 't, F>(&mut self, enumerator: E, mut elem: F)
    where
        T: ReinterpretAsBytes + StaticSize,
        F: FnMut(usize) -> &'t T,
    {
        /* Load offset */
        let offset = self.load_offset(enumerator);

        /* Read length of the data */
        let length: Size = self.file.read_from_stack(offset)
            .await
            .expect("failed to read from stack");

        /* Iterate all elements and assign to them offsets */
        let elements = (0..).map(|i| elem(i));
        let offsets = (0..).map(|i| offset + i * T::static_size() as Size + Size::static_size() as Size);
        
        /* Write them to file */
        for (elem, offset) in elements.zip(offsets).take(length as usize) {
            self.file.write_to_stack(offset, &elem.as_bytes())
                .await
                .expect("failed to write to stack");
        }
    }

    /// Assings new value to some element in enum-named array.
    #[allow(dead_code)]
    pub async fn assign_array_element<T>(&mut self, enumerator: E, elem: T, idx: usize) -> SaveResult<()>
    where
        T: ReinterpretAsBytes + StaticSize,
    {
        /* Load offset */
        let offset = self.load_offset(enumerator);

        /* Read length of an array */
        let length: Size = self.file.read_from_stack(offset)
            .await
            .expect("failed to read from stack");

        /* Test if given index passed into length */
        Self::test_index(idx as Offset, length)?;

        /* Get offset of an element */
        let offset = offset
                   + idx as Offset * T::static_size() as Size
                   + Size::static_size() as Size;

        /* Write element */
        self.file.write_to_stack(offset, &elem.as_bytes())
            .await
            .expect("failed to write to stack");

        Ok(())
    }

    /// Reads enum-named array of values from stack file.
    #[allow(dead_code)]
    pub async fn read_array<T: ReinterpretFromBytes + StaticSize>(&mut self, enumerator: E) -> Vec<T> {
        /* Getting offset of array length */
        let offset = self.load_offset(enumerator);

        /* Read array length */
        let length: Size = self.file.read_from_stack(offset)
            .await
            .expect("failed to read from stack");

        /* Actual array offset */
        let offset = offset + Offset::static_size() as Size;

        /* Loading buffer */
        let mut buffer = vec![0; T::static_size()];

        /* Resulting collection */
        let mut result = Vec::<T>::with_capacity(length as usize);

        /* Read all elements to `result` */
        for i in 0..length {
            /* Read to buffer */
            StackHeap::seek_read(&mut self.file.stack, &mut buffer, offset + i * T::static_size() as Size)
                .await
                .expect("failed to seek-read");

            /* Push value to `result` */
            result.push(T::from_bytes(&buffer));
        }

        return result
    }

    /// Reads element of enum-named array on stack by index `idx`.
    #[allow(dead_code)]
    pub async fn read_array_element<T>(&mut self, enumerator: E, idx: usize) -> SaveResult<T>
    where
        T: ReinterpretFromBytes + StaticSize,
    {
        /* Load offset */
        let offset = self.load_offset(enumerator);

        /* Read length */
        let length: Size = self.file.read_from_stack(offset)
            .await
            .expect("failed to read from stack");

        /* Test if index if off available range */
        Self::test_index(idx as Offset, length)?;

        /* Get data offset */
        let offset = offset + idx as Offset * T::static_size() as Size + Size::static_size() as Size;

        /* Read element */
        Ok(
            self.file.read_from_stack(offset)
                .await
                .expect("failed to read from stack")
        )
    }

    /// Allocates data on heap of file with pointer on stack and writes all given bytes.
    #[allow(dead_code)]
    pub async fn pointer(mut self, bytes: Vec<u8>, enumerator: E) -> Self {
        /* Allocate bytes */
        let offset = {
            let alloc = self.file.alloc(bytes.len() as Size)
                .await
                .expect("alloc failed");

            self.file.write_to_heap(alloc, &bytes)
                .await
                .expect("failed to write to heap");

            alloc.get_stack_offset()
        };

        /* Save offset */
        self.store_offset(enumerator, offset)
            .expect("failed to store offset");

        return self
    }

    /// Assignes new data to the value of pointer to heap.
    #[allow(dead_code)]
    pub async fn assign_to_pointer(&mut self, bytes: &[u8], enumerator: E) {
        /* Load offset */
        let stack_offset = self.load_offset(enumerator);

        /* Allocate new data */
        let alloc = self.file.realloc(bytes.len() as Size, stack_offset)
            .await
            .expect("realloc failed");

        self.file.write_to_heap(alloc, bytes)
            .await
            .expect("failed to write to heap");
    }

    /// Reads data from heap by stack pointer to heap on.
    #[allow(dead_code)]
    pub async fn read_from_pointer<T, F: FnOnce(&[u8]) -> T>(&mut self, enumerator: E, item: F) -> T {
        /* Load offsets */
        let stack_offset = self.load_offset(enumerator);
        let heap_offset: Offset = self.file.read_from_stack(stack_offset)
            .await
            .expect("failed to read from stack");

        /* Read data */
        let bytes = self.file.read_from_heap(heap_offset)
            .await
            .expect("failed to read from heap");

        item(&bytes)
    }

    /// Allocates an array of pinters on stack and array of data on heap.
    pub async fn pointer_array<F, Fut>(mut self, len: usize, enumerator: E, mut elem: F) -> Self
    where
        F: FnMut(usize) -> Fut,
        Fut: Future<Output = Vec<u8>>,
    {
        /* Push size to stack and store its offset */
        let stack_offset = self.file.push(&(len as Size).as_bytes())
            .await
            .expect("failed to push");

        self.store_offset(enumerator, stack_offset)
            .expect("failed to store offset");

        /* Write all elements to heap */
        for i in 0..len {
            let data = elem(i).await;
            let alloc = self.file.alloc(data.len() as Size)
                .await
                .expect("alloc failed");

            self.file.write_to_heap(alloc, &data)
                .await
                .expect("failed to write to a heap");
        }

        return self
    }

    /// Assigns new array of pointers to existed one.
    #[allow(dead_code)]
    pub async fn assign_pointer_array<F>(&mut self, enumerator: E, mut elem: F)
    where
        F: FnMut(usize) -> Vec<u8>,
    {
        /* Load offset */
        let stack_offset = self.load_offset(enumerator);

        /* Read length */
        let length: Size = self.file.read_from_stack(stack_offset)
            .await
            .expect("failed to read from stack");

        /* Offsets iterator */
        let offsets = (1..).map(|i| stack_offset + i * Offset::static_size() as Size);

        /* Elements iterator */
        let elements = (0..).map(|i| elem(i));

        /* Write bytes */
        for (bytes, offset) in elements.zip(offsets).take(length as usize) {
            let alloc = self.file.realloc(bytes.len() as Size, offset)
                .await
                .expect("realloc failed");

            self.file.write_to_heap(alloc, &bytes)
                .await
                .expect("failed to write to a heap");
        }
    }

    /// Assigns new element to the element by index of pointer on stack.
    #[allow(dead_code)]
    pub async fn assign_pointer_array_element(&mut self, enumerator: E, bytes: Vec<u8>, idx: usize) -> SaveResult<()> {
        /* Load offset */
        let offset = self.load_offset(enumerator);

        /* Read length of an array */
        let length: Size = self.file.read_from_stack(offset).await?;

        /* Test if index if off available range */
        Self::test_index(idx as Offset, length)?;

        /* Calculate offset on stack */
        let offset = offset + (idx + 1) as Offset * Offset::static_size() as Size;

        /* Rewrite data */
        let alloc = self.file.realloc(bytes.len() as Size, offset).await?;
        self.file.write_to_heap(alloc, &bytes).await?;

        Ok(())
    }

    /// Reads an array of data from heap.
    pub async fn read_pointer_array<T, F, Fut>(&mut self, enumerator: E, mut elem: F) -> Vec<T>
    where
        F: FnMut(usize, Vec<u8>) -> Fut,
        Fut: Future<Output = T>,
    {
        /* Load stack data offset */
        let length_offset = self.load_offset(enumerator);

        /* Read array length */ 
        let length: Size = self.file.read_from_stack(length_offset)
            .await
            .expect("failed to read from stack");

        /* Resulting vector */
        let mut result = Vec::with_capacity(length as usize);

        /* Read all elements */
        let offset_size = Size::static_size() as Size;
        for i in 1..=length {
            /* Read offset on heap */
            let heap_offset: Offset = self.file.read_from_stack(length_offset + i * offset_size)
                .await
                .expect("failed to read from stack");

            /* Read data bytes */
            let bytes = self.file.read_from_heap(heap_offset)
                .await
                .expect("failed to read from heap");

            /* Reinterpret them and push to result */
            result.push(elem(i as usize - 1, bytes).await);
        }

        return result
    }

    /// Reads a pointer array element at index `idx`.
    #[allow(dead_code)]
    pub async fn read_pointer_array_element<T, F>(&mut self, enumerator: E, idx: usize, elem: F) -> SaveResult<T>
    where
        F: FnOnce(&[u8]) -> T
    {
        /* Load offset */
        let offset = self.load_offset(enumerator);

        /* Read length */
        let length: Size = self.file.read_from_stack(offset)
            .await
            .expect("failed to read from stack");

        /* Test if index if off available range */
        Self::test_index(idx as Offset, length)?;

        /* Calculate actual offsets */
        let offset = offset + Size::static_size() as Size + idx as Offset * Offset::static_size() as Size;

        let heap_offset = self.file.read_from_stack(offset)
            .await
            .expect("failed to red from stack");

        /* Read data bytes */
        let bytes = self.file.read_from_heap(heap_offset)
            .await
            .expect("failed to read from heap");

        /* Return data */
        Ok(elem(&bytes))
    }

    /// Saves offset by enumerator.
    fn store_offset(&mut self, enumerator: E, offset: Offset) -> SaveResult<()> {
        match self.offsets.insert(enumerator.into(), offset) {
            None => Ok(()),
            Some(old) => Err(SaveError::DataOverride(old))
        }
    }

    /// Loads offset by enumerator.
    fn load_offset(&self, enumerator: E) -> Offset {
        *self.offsets
            .get(&enumerator.into())
            .expect(&format!(
                "There is no data enumerated by {}",
                enumerator.into()
            ))
    }

    async fn offsets_async_write(file: &mut File, bytes: &[u8], offset: Offset) -> io::Result<()> {
        file.seek(SeekFrom::Start(offset))
            .await?;
        let _n_bytes = file.write(bytes).await?;

        Ok(())
    }

    /// Saves the save.
    pub async fn save(mut self) -> io::Result<Self> {
        /* Sync all changes to file */
        self.file.sync().await?;

        /* Save offsets length to `meta.off` file */
        let n_offsets = self.offsets.len() as Size;
        Self::offsets_async_write(
            &mut self.offsets_save,
            &n_offsets.as_bytes(),
            0
        ).await?;

        /* Save all offsets to `meta.off` file */
        let offset_size = Offset::static_size() as Size;
        for ((&enumerator, &offset), i) in self.offsets.iter().zip((1_u64..).step_by(2)) {
            Self::offsets_async_write(
                &mut self.offsets_save,
                &enumerator.as_bytes(),
                offset_size * i
            ).await?;

            Self::offsets_async_write(
                &mut self.offsets_save,
                &offset.as_bytes(),
                offset_size * (i + 1)
            ).await?;
        }

        /* Sync all changes to file */
        self.offsets_save.sync_all().await?;

        Ok(self)
    }

    /// Test if given index is valid.
    fn test_index(idx: Offset, len: Size) -> SaveResult<()> {
        match idx < len {
            true => Err(SaveError::IndexOutOfBounds { idx, size: len }),
            false => Ok(()),
        }
    }
}
