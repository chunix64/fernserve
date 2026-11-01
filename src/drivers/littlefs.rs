use alloc::string::String;

use crate::{models::common::FileSystemError, utils::data_structures::Heap};

// Storage
pub struct LittleFsStorage<T, const SIZE: usize>
where
    T: embedded_storage::nor_flash::NorFlash,
{
    storage: T,
}

impl<T, const SIZE: usize> LittleFsStorage<T, SIZE>
where
    T: embedded_storage::nor_flash::NorFlash,
{
    fn new(storage: T) -> Self {
        Self { storage }
    }
}

impl<T, const SIZE: usize> littlefs2::driver::Storage for LittleFsStorage<T, SIZE>
where
    T: embedded_storage::nor_flash::NorFlash,
{
    type CACHE_SIZE = littlefs2::consts::U64;
    type LOOKAHEAD_SIZE = littlefs2::consts::U4;

    const READ_SIZE: usize = T::READ_SIZE;
    const WRITE_SIZE: usize = T::WRITE_SIZE;
    const BLOCK_SIZE: usize = T::ERASE_SIZE;
    const BLOCK_COUNT: usize = SIZE / T::ERASE_SIZE;
    const BLOCK_CYCLES: isize = 100;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> littlefs2::io::Result<usize> {
        match self.storage.read(off as u32, buf) {
            Ok(_) => Ok(0),
            Err(_) => Err(littlefs2::io::Error::IO),
        }
    }

    fn write(&mut self, off: usize, data: &[u8]) -> littlefs2::io::Result<usize> {
        match self.storage.write(off as u32, data) {
            Ok(_) => Ok(0),
            Err(_) => Err(littlefs2::io::Error::IO),
        }
    }

    fn erase(&mut self, off: usize, len: usize) -> littlefs2::io::Result<usize> {
        match self.storage.erase(off as u32, (off + len) as u32) {
            Ok(_) => Ok(0),
            Err(_) => Err(littlefs2::io::Error::IO),
        }
    }
}

// File
pub struct LittleFsFile<'a, T, const SIZE: usize>
where
    T: embedded_storage::nor_flash::NorFlash + littlefs2::driver::Storage,
{
    file: littlefs2::fs::File<'a, 'a, LittleFsStorage<T, SIZE>>,
    file_alloc: Heap<littlefs2::fs::FileAllocation<LittleFsStorage<T, SIZE>>>,
}

impl<'a, T, const SIZE: usize> LittleFsFile<'a, T, SIZE>
where
    T: embedded_storage::nor_flash::NorFlash + littlefs2::driver::Storage,
{
    pub fn new(
        file: littlefs2::fs::File<'a, 'a, LittleFsStorage<T, SIZE>>,
        file_alloc: Heap<littlefs2::fs::FileAllocation<LittleFsStorage<T, SIZE>>>,
    ) -> Result<Self, crate::models::common::FileError> {
        Ok(Self { file, file_alloc })
    }
}

impl<'a, T, const SIZE: usize> crate::models::filesystem::File<'a> for LittleFsFile<'a, T, SIZE>
where
    T: embedded_storage::nor_flash::NorFlash + littlefs2::driver::Storage,
{
    type FileSystem = littlefs2::fs::File<'a, 'a, LittleFsStorage<T, SIZE>>;
}

// Filesystem
pub struct LittleFsFileSystem<'a, T, const SIZE: usize>
where
    T: embedded_storage::nor_flash::NorFlash + littlefs2::driver::Storage,
{
    storage: LittleFsStorage<T, SIZE>,
    alloc: littlefs2::fs::Allocation<LittleFsStorage<T, SIZE>>,
    fs: Option<littlefs2::fs::Filesystem<'a, LittleFsStorage<T, SIZE>>>,
}

impl<'a, T, const SIZE: usize> crate::models::filesystem::FileSystem<'a>
    for LittleFsFileSystem<'a, T, SIZE>
where
    T: embedded_storage::nor_flash::NorFlash + littlefs2::driver::Storage,
{
    type Storage = T;
    type File = LittleFsFile<'a, T, SIZE>;

    fn new(storage: T) -> Result<Self, FileSystemError> {
        let mut storage = LittleFsStorage::new(storage);

        if !littlefs2::fs::Filesystem::is_mountable(&mut storage) {
            match littlefs2::fs::Filesystem::format(&mut storage) {
                Ok(()) => {}
                Err(_) => return Err(FileSystemError::Format),
            }
        }

        let alloc = littlefs2::fs::Filesystem::allocate();

        Ok(Self {
            storage,
            alloc,
            fs: None,
        })
    }

    fn mount(&'a mut self) -> Result<(), FileSystemError> {
        match littlefs2::fs::Filesystem::mount(&mut self.alloc, &mut self.storage) {
            Ok(fs) => {
                self.fs = Some(fs);
                Ok(())
            }
            Err(_) => Err(FileSystemError::Mount),
        }
    }

    fn open(&'a mut self, mut path: String) -> Result<Self::File, FileSystemError> {
        if let Some(fs) = &self.fs {
            let file_alloc = Heap::new(littlefs2::fs::File::allocate());

            path.push('\0');
            let path = match littlefs2::path::Path::from_str_with_nul(&path) {
                Ok(path) => path,
                Err(_) => return Err(FileSystemError::File),
            };

            // FatFS library has already exposed the unsafe open_file() as safe anyway
            // Just manually close the file to prevent undefined behaviors

            unsafe {
                let file = match littlefs2::fs::File::open(fs, &mut *file_alloc.as_ptr(), path) {
                    Ok(file) => file,
                    Err(_) => return Err(FileSystemError::File),
                };

                return match LittleFsFile::new(file, file_alloc) {
                    Ok(file) => Ok(file),
                    Err(_) => Err(FileSystemError::File),
                };
            }
        }

        Err(FileSystemError::UnMounted)
    }
}
