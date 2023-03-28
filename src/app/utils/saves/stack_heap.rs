use {
    crate::app::utils::{
        cfg::save::{HEAP_FILE_EXTENSION, STACK_FILE_EXTENSION},
        reinterpreter::*,
    },
    super::{Offset, Size},
    std::{
        collections::HashSet,
        ops::Range,
        path::Path,
    },
    thiserror::Error,
    tokio::{
        fs::{self, File, OpenOptions},
        io::{self, AsyncReadExt, AsyncWriteExt, AsyncSeekExt, SeekFrom},
    },
};

#[derive(Debug, Error)]
pub enum StackHeapError {
    #[error("data lengths on given offset and on passed type T are not equal! Expected: {expected}, got: {read}")]
    AllocSizeIsDifferent {
        read: usize,
        expected: usize,
    },

    #[error("data size ({data_len}) passed to this function should be not greater than allowed allocation ({alloc_size})!")]
    NotEnoughMemory {
        data_len: usize,
        alloc_size: usize,
    },

    #[error("io failed: {0}")]
    Io(#[from] io::Error),
}

pub type StackHeapResult<T> = Result<T, StackHeapError>;

#[derive(Clone, Copy, Debug)]
pub struct Alloc {
    pub(self) stack_offset: Offset,
    pub(self) heap_offset: Offset,
    pub(self) size: Size,
}

#[allow(dead_code)]
impl Alloc {
    pub fn get_stack_offset(self) -> Offset { self.stack_offset }
    pub fn get_heap_offset(self) -> Offset { self.heap_offset }
    pub fn get_size(self) -> Offset { self.size }
}

#[derive(Debug)]
pub struct StackHeap {
    pub stack: File,
    pub stack_ptr: Offset,

    pub heap: File,
    pub eof: Offset,
    freed_space: HashSet<Range<Offset>>,
}

impl StackHeap {
    /// Makes new StackHeap struct and new directory for their files.
    pub async fn new(path: &str, name: &str) -> io::Result<Self> {
        /* Create directory if this path doesn't exist */
        if !Path::new(path).exists() {
            fs::create_dir(path).await?;
        }

        let stack = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(format!("{path}/{name}.{STACK_FILE_EXTENSION}"))
            .await?;

        let heap = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(format!("{path}/{name}.{HEAP_FILE_EXTENSION}"))
            .await?;
        
        Ok(Self {
            stack,
            heap,
            stack_ptr: 0,
            eof: 0,
            freed_space: HashSet::new(),
        })
    }

    /// Saves the files.
    pub async fn sync(&self) -> std::io::Result<()> {
        self.stack.sync_all().await?;
        self.heap.sync_all().await?;

        Ok(())
    }

    pub async fn seek_write(file: &mut File, bytes: &[u8], offset: Offset) -> io::Result<()> {
        file.seek(SeekFrom::Start(offset)).await?;
        file.write(bytes).await?;

        Ok(())
    }

    pub async fn seek_read(file: &mut File, buffer: &mut [u8], offset: Offset) -> io::Result<()> {
        file.seek(SeekFrom::Start(offset)).await?;
        file.read(buffer).await?;

        Ok(())
    }

    /// Pushes data to stack. Returns an offset of the data.
    pub async fn push(&mut self, data: &[u8]) -> io::Result<Offset> {
        /* Write new data */
        let offset = self.stack_ptr;
        Self::seek_write(&mut self.stack, data, offset).await?;

        /* Increment stack pointer */
        self.stack_ptr += data.len() as Size;

        return Ok(offset)
    }

    /// Writes data to stack by its offset.
    pub async fn write_to_stack(&mut self, offset: Offset, data: &[u8]) -> io::Result<()> {
        Self::seek_write(&mut self.stack, data, offset).await?;
        Ok(())
    }

    /// Reads value from stack.
    pub async fn read_from_stack<T: FromBytes + StaticSize>(&mut self, offset: Offset) -> io::Result<T> {
        /* Read bytes */
        let mut buffer = vec![0; T::static_size()];
        Self::seek_read(&mut self.stack, &mut buffer, offset).await?;

        /* Reinterpret */
        Ok(T::from_bytes(&buffer).expect("failed to make T from bytes"))
    }

    /// Reads value from heap by `heap_offset`.
    /// * Note: `heap_offset` should be point on Size mark of the data.
    pub async fn read_from_heap(&mut self, heap_offset: Offset) -> io::Result<Vec<u8>> {
        /* Read size */
        let size = {
            let mut buffer = vec![0; Size::static_size()];
            Self::seek_read(&mut self.heap, &mut buffer, heap_offset).await?;
            Size::from_bytes(&buffer)
                .expect("failed to make Size from bytes")
        };

        /* Read data */
        let mut buffer = vec![0; size as usize];
        Self::seek_read(&mut self.heap, &mut buffer, heap_offset + Size::static_size() as Size).await?;

        Ok(buffer)
    }

    /// Reads value from heap of file by offset on stack.
    #[allow(dead_code)]
    pub async fn heap_read<T: FromBytes + StaticSize>(&mut self, stack_offset: Offset) -> StackHeapResult<T> {
        /* Read offset on heap from stack */
        let heap_offset: Offset = self.read_from_stack(stack_offset).await?;

        /* Read bytes */
        let bytes = self.read_from_heap(heap_offset).await?;

        /* Read bytes from heap */
        if bytes.len() == T::static_size() {
            Ok(T::from_bytes(&bytes).expect("failed to make T from bytes"))
        } else {
            Err(StackHeapError::AllocSizeIsDifferent { read: bytes.len(), expected: T::static_size() })
        }
    }

    /// Allocates space on heap. Returns an Alloc struct that contains all information about this allocation.
    pub async fn alloc(&mut self, size: Size) -> io::Result<Alloc> {
        /* Test freed memory */
        let full_size = size + Offset::static_size() as Size;
        let heap_offset = self.get_available_offset(full_size);

        /* Save size of data to heap */
        Self::seek_write(&mut self.heap, &size.as_bytes(), heap_offset).await?;

        /* Save this offset on stack */
        let stack_offset = self.push(&heap_offset.as_bytes()).await?;

        Ok(Alloc { stack_offset, heap_offset, size })
    }

    /// Reallocates space on heap. Returns an Alloc struct that contains all information about this allocation.
    /// * Note: it can avoid alocation if new size isn't greater than old.
    pub async fn realloc(&mut self, size: Size, stack_offset: Offset) -> io::Result<Alloc> {
        /* Read size that was before */
        let heap_offset = self.read_from_stack(stack_offset).await?;
        let before_size = {
            let mut buffer = vec![0; Size::static_size()];
            Self::seek_read(&mut self.heap, &mut buffer, heap_offset).await?;
            Size::from_bytes(&buffer)
                .expect("failed to make Size from bytes")
        };

        /* Calculate size include sizes bytes */
        let full_size = size + Size::static_size() as Size;

        if before_size < size {
            /* Free data on offset */
            self.free(stack_offset).await?;

            /* Test freed memory */
            let heap_offset = self.get_available_offset(full_size);

            /* Save size of data to heap */
            Self::seek_write(&mut self.heap, &size.as_bytes(), heap_offset).await?;

            /* Save this offset on stack */
            self.write_to_stack(stack_offset, &heap_offset.as_bytes()).await?;

            Ok(Alloc { stack_offset, heap_offset, size })
        } else {
            /* If new size cause free memory then use it */
            if before_size != size {
                self.insert_free(heap_offset + full_size, before_size - size);
            }

            /* Write size to heap */
            Self::seek_write(&mut self.heap, &size.as_bytes(), heap_offset).await?;

            Ok(Alloc { stack_offset, heap_offset, size })
        }
    }

    /// Stoles available offset from heap. It can edit freed_space so it is expensive.
    /// * Note: size is a full size of allocation, include size mark in heap
    fn get_available_offset(&mut self, size: Size) -> Offset {
        match self.freed_space.iter().find(|range| range.end >= size + range.start).cloned() {
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
    pub async fn write_to_heap(&mut self, Alloc { size, heap_offset: offset, .. }: Alloc, data: &[u8]) -> StackHeapResult<()> {
        if size >= data.len() as Size {
            Self::seek_write(&mut self.heap, data, offset + Size::static_size() as Size).await?;
            Ok(())
        } else {
            Err(StackHeapError::NotEnoughMemory {
                data_len: data.len(),
                alloc_size: size as usize
            })
        }
    }

    /// Marks memory as free.
    #[allow(dead_code)]
    pub async fn free(&mut self, stack_offset: Offset) -> io::Result<()> {
        /* Read offset and size */
        let heap_offset: Offset = self.read_from_stack(stack_offset).await?;
        let size = {
            let mut buffer = vec![0; Size::static_size()];
            Self::seek_read(&mut self.heap, &mut buffer, heap_offset).await?;

            /* Note: Size mark in heap is included */
            let size = Size::from_bytes(&buffer)
                .expect("failed to make Size from bytes");

            size + Size::static_size() as Size
        };
        
        /* Insert free range */
        self.insert_free(heap_offset, size);

        Ok(())
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
    use crate::app::utils::runtime::RUNTIME;

    #[test]
    fn test_allocation() {
        RUNTIME.block_on(async {
            let name = "test_allocation";
            let mut file = StackHeap::new(name, name)
                .await
                .expect("failed to create StackHeap");

            let bytes_64:  Vec<_> = (0_u64..).flat_map(|num| num.as_bytes()).take(64) .collect();
            let bytes_128: Vec<_> = (0_u64..).flat_map(|num| num.as_bytes()).take(128).collect();

            let bytes_64_rev: Vec<_> = bytes_64.iter().map(|&byte| byte).rev().collect();

            let alloc_64 = file.alloc(64).await.unwrap();
            file.write_to_heap(alloc_64, &bytes_64).await.unwrap();

            let alloc_128 = file.realloc(128, alloc_64.get_stack_offset()).await.unwrap();
            file.write_to_heap(alloc_128, &bytes_128).await.unwrap();

            let alloc_64_rev = file.alloc(64).await.unwrap();
            file.write_to_heap(alloc_64_rev, &bytes_64_rev).await.unwrap();

            let bytes_after = file.read_from_heap(alloc_128.get_heap_offset()).await.unwrap();
            let bytes_rev_after = file.read_from_heap(alloc_64_rev.get_heap_offset()).await.unwrap();

            assert_eq!(bytes_128, bytes_after);
            assert_eq!(bytes_64_rev, bytes_rev_after);
            assert_eq!(file.stack_ptr, 2 * Offset::static_size() as Size);
            assert_eq!(file.eof, 64 + 128 + 2 * Size::static_size() as Size);
        });
    }

    #[test]
    fn test_merging() {
        RUNTIME.block_on(async {
            let name = "test_merging";
            let mut file = StackHeap::new(name, name)
                .await
                .expect("failed to create StackHeap");

            let bytes_64:  Vec<_> = (0_u64..).flat_map(|num| num.as_bytes()).take(64) .collect();
            let bytes_128: Vec<_> = (0_u64..).flat_map(|num| num.as_bytes()).take(128).collect();

            let alloc_64_1 = file.alloc(64).await.unwrap();
            file.write_to_heap(alloc_64_1, &bytes_64).await.unwrap();

            let alloc_64_2 = file.alloc(64).await.unwrap();
            file.write_to_heap(alloc_64_2, &bytes_64).await.unwrap();

            assert_eq!(bytes_64, file.read_from_heap(alloc_64_1.get_heap_offset()).await.unwrap());
            assert_eq!(bytes_64, file.read_from_heap(alloc_64_2.get_heap_offset()).await.unwrap());

            file.free(alloc_64_1.get_stack_offset()).await.unwrap();
            file.free(alloc_64_2.get_stack_offset()).await.unwrap();

            let alloc_128 = file.alloc(128).await.unwrap();
            file.write_to_heap(alloc_128, &bytes_128).await.unwrap();

            assert_eq!(bytes_128, file.read_from_heap(alloc_128.get_heap_offset()).await.unwrap());
            assert_eq!(file.stack_ptr, 3 * Offset::static_size() as Size);
            assert_eq!(file.eof, 2 * 64 + 2 * Size::static_size() as Size);

            let range = 128 + Size::static_size() as Size .. 2 * (64 + Size::static_size() as Size);
            assert!(file.freed_space.contains(&range));
        });
    }
}