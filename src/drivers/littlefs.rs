use core::{cell::RefCell, mem::ManuallyDrop};

use alloc::{borrow::ToOwned, rc::Rc, vec::Vec};
use littlefs2::io::Write;

use crate::{
    models::{
        common::{FileError, FileSystemError},
        filesystem::{FileSystemNode, FileSystemNodeType},
        path::Path,
    },
    utils::data_structures::Heap,
};
use embedded_storage::nor_flash::NorFlash;

// Types and aliases
pub trait Flash: NorFlash {}
impl<T: NorFlash> Flash for T {}

// Storage
#[derive(Debug)]
pub struct LittleFsStorage<T: Flash, const SIZE: usize> {
    storage: T,
    start_offset: usize,
}

impl<T: Flash, const SIZE: usize> LittleFsStorage<T, SIZE> {
    pub fn new(storage: T, start_offset: usize) -> Self {
        Self {
            storage,
            start_offset,
        }
    }

    fn abs(&self, off: usize, len: usize) -> Result<u32, littlefs2::io::Error> {
        if off.checked_add(len).is_none_or(|end| end > SIZE) {
            return Err(littlefs2::io::Error::IO);
        }
        Ok((self.start_offset + off) as u32)
    }
}

impl<T: Flash, const SIZE: usize> littlefs2::driver::Storage for LittleFsStorage<T, SIZE> {
    type CACHE_SIZE = littlefs2::consts::U64;
    type LOOKAHEAD_SIZE = littlefs2::consts::U4;

    const READ_SIZE: usize = T::READ_SIZE;
    const WRITE_SIZE: usize = T::WRITE_SIZE;
    const BLOCK_SIZE: usize = T::ERASE_SIZE;
    const BLOCK_COUNT: usize = SIZE / T::ERASE_SIZE;
    const BLOCK_CYCLES: isize = 100;

    // TODO: handle errors
    fn read(&mut self, off: usize, buf: &mut [u8]) -> littlefs2::io::Result<usize> {
        let addr = self.abs(off, buf.len())?;
        match self.storage.read(addr, buf) {
            Ok(_) => Ok(0),
            Err(_) => Err(littlefs2::io::Error::IO),
        }
    }

    fn write(&mut self, off: usize, data: &[u8]) -> littlefs2::io::Result<usize> {
        let addr = self.abs(off, data.len())?;
        match self.storage.write(addr, data) {
            Ok(_) => Ok(0),
            Err(_) => Err(littlefs2::io::Error::IO),
        }
    }

    fn erase(&mut self, off: usize, len: usize) -> littlefs2::io::Result<usize> {
        let addr = self.abs(off, len)?;
        match self.storage.erase(addr, addr + len as u32) {
            Ok(_) => Ok(0),
            Err(_) => Err(littlefs2::io::Error::IO),
        }
    }
}

// File
// SAFETY: Keep the file_alloc
pub struct LittleFsFile<'a, S: littlefs2::driver::Storage> {
    pub file: ManuallyDrop<littlefs2::fs::File<'a, 'a, S>>,
    pub _file_alloc: Heap<littlefs2::fs::FileAllocation<S>>,
}

impl<'a, S: littlefs2::driver::Storage> LittleFsFile<'a, S> {
    pub fn new(
        path: Path,
        filesystem: &'a mut littlefs2::fs::Filesystem<'a, S>,
    ) -> Result<Self, FileError> {
        // SAFETY: FatFS library has already exposed the unsafe open_file() as safe anyway
        // Just manually close the file to prevent undefined behaviors
        let path = to_lfs_path(&path).ok_or(FileError::FileSystem)?;
        let file_alloc = Heap::new(littlefs2::fs::File::allocate());
        let file_alloc_0 = unsafe { &mut *(&mut *file_alloc.as_ptr() as *mut _) };

        unsafe {
            let file = match littlefs2::fs::File::open(filesystem, &mut *file_alloc_0, path) {
                Ok(file) => file,
                Err(_) => return Err(FileError::FileSystem),
            };

            Ok(Self {
                file: ManuallyDrop::new(file),
                _file_alloc: file_alloc,
            })
        }
    }
}

impl<'a, S: littlefs2::driver::Storage> crate::models::filesystem::File for LittleFsFile<'a, S> {
    async fn close(self) -> Result<(), FileError> {
        let mut this = ManuallyDrop::new(self);
        let file = unsafe { ManuallyDrop::take(&mut this.file) };
        unsafe { file.close().map_err(|_| FileError::Close) }
    }
}

impl<'a, S: littlefs2::driver::Storage> embedded_io_async::ErrorType for LittleFsFile<'a, S> {
    type Error = FileError;
}

impl<'a, S: littlefs2::driver::Storage> embedded_io_async::Read for LittleFsFile<'a, S> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.file.read(buf).map_err(|_| FileError::FileSystem)
    }
}

impl<'a, S: littlefs2::driver::Storage> embedded_io_async::Write for LittleFsFile<'a, S> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.file.flush().map_err(|_| FileError::Flush)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.file.write(buf).map_err(|_| FileError::Write)
    }
}

impl<'a, S: littlefs2::driver::Storage> embedded_io_async::Seek for LittleFsFile<'a, S> {
    async fn seek(&mut self, pos: embedded_io_async::SeekFrom) -> Result<u64, Self::Error> {
        let pos = to_lfs_seek(pos).ok_or(FileError::Seek)?;

        self.file
            .seek(pos)
            .map(|value| value as u64)
            .map_err(|_| FileError::Seek)
    }
}

impl<'a, S: littlefs2::driver::Storage> Drop for LittleFsFile<'a, S> {
    fn drop(&mut self) {
        let file = unsafe { ManuallyDrop::take(&mut self.file) };
        let _ = unsafe { file.close() };
    }
}
// Filesystem
pub struct LittleFsFileSystem<'a, S: littlefs2::driver::Storage> {
    fs: Option<Rc<RefCell<littlefs2::fs::Filesystem<'a, S>>>>,
    alloc: littlefs2::fs::Allocation<S>,
    storage: S,
}

impl<'a, S: littlefs2::driver::Storage> LittleFsFileSystem<'a, S> {
    pub fn new(storage: S) -> Result<Self, FileSystemError> {
        let alloc = littlefs2::fs::Filesystem::allocate();

        Ok(Self {
            storage,
            alloc,
            fs: None,
        })
    }
}

impl<'a, S: littlefs2::driver::Storage> crate::models::filesystem::FileSystem
    for LittleFsFileSystem<'a, S>
{
    type File = LittleFsFile<'a, S>;

    async fn format(&mut self) -> Result<(), FileSystemError> {
        littlefs2::fs::Filesystem::format(&mut self.storage).map_err(|_| FileSystemError::Format)
    }

    async fn mount(&mut self) -> Result<(), FileSystemError> {
        // SAFETY: `fs` borrows `alloc` and `storage`, which live in the same struct.
        // The extended borrows are only valid as long as `self` is not moved or
        // dropped while `fs` is alive.
        let (alloc, storage) = unsafe {
            (
                &mut *(&mut self.alloc as *mut _),
                &mut *(&mut self.storage as *mut _),
            )
        };

        let fs =
            littlefs2::fs::Filesystem::mount(alloc, storage).map_err(|_| FileSystemError::Mount)?;

        self.fs = Some(Rc::new(RefCell::new(fs)));

        Ok(())
    }

    async fn is_mountable(&mut self) -> Result<bool, FileSystemError> {
        Ok(littlefs2::fs::Filesystem::is_mountable(&mut self.storage))
    }

    async fn available_space(&mut self) -> Result<usize, FileSystemError> {
        let fs = self.fs.as_ref().ok_or(FileSystemError::AvailableSpace)?;
        littlefs2::fs::Filesystem::available_space(&fs.borrow_mut())
            .map_err(|_| FileSystemError::AvailableSpace)
    }

    async fn total_space(&mut self) -> Result<usize, FileSystemError> {
        let fs = self.fs.as_ref().ok_or(FileSystemError::AvailableSpace)?;
        Ok(littlefs2::fs::Filesystem::total_space(&fs.borrow_mut()))
    }

    async fn exist(&mut self, path: Path) -> Result<bool, FileSystemError> {
        let fs = self.fs.as_ref().ok_or(FileSystemError::Exist)?.borrow_mut();
        let path = to_lfs_path(&path).ok_or(FileSystemError::Exist)?;

        Ok(fs.exists(path))
    }

    async fn open(&mut self, path: Path) -> Result<Self::File, FileSystemError> {
        let fs = self.fs.as_ref().ok_or(FileSystemError::UnMounted)?;

        unsafe {
            let fs_ref: &'a mut littlefs2::fs::Filesystem<'a, S> = &mut *fs.as_ptr();
            let file = LittleFsFile::new(path, fs_ref).map_err(|_| FileSystemError::Open)?;

            Ok(file)
        }
    }

    async fn read_dir(&mut self, path: Path) -> Result<Vec<FileSystemNode>, FileSystemError> {
        let fs = self
            .fs
            .as_ref()
            .ok_or(FileSystemError::ReadDir)?
            .borrow_mut();
        let path = to_lfs_path(&path).ok_or(FileSystemError::ReadDir)?;

        fs.read_dir_and_then(path, |read_dir| {
            let mut nodes = Vec::new();

            for entry in read_dir {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => {
                        continue;
                    }
                };

                let node_type = match entry.metadata().file_type() {
                    littlefs2::fs::FileType::Dir => FileSystemNodeType::Dir,
                    littlefs2::fs::FileType::File => FileSystemNodeType::File,
                };

                let path = entry.path().as_str().to_owned();
                let path = Path::new(&path).map_err(|_| littlefs2::io::Error::IO)?;

                nodes.push(FileSystemNode { path, node_type });
            }

            Ok(nodes)
        })
        .map_err(|_| FileSystemError::ReadDir)
    }

    async fn create_dir(&mut self, path: Path) -> Result<(), FileSystemError> {
        let fs = self.fs.as_ref().ok_or(FileSystemError::CreateDir)?;
        let path = to_lfs_path(&path).ok_or(FileSystemError::CreateDir)?;

        fs.borrow_mut()
            .create_dir(path)
            .map_err(|_| FileSystemError::CreateDir)
    }

    async fn create_file(&mut self, path: Path) -> Result<(), FileSystemError> {
        let fs = self.fs.as_ref().ok_or(FileSystemError::CreateDir)?;
        let path = to_lfs_path(&path).ok_or(FileSystemError::CreateDir)?;

        fs.borrow_mut()
            .create_file_and_then(path, |_| Ok(()))
            .map_err(|_| FileSystemError::CreateFile)
    }

    async fn remove(&mut self, path: Path) -> Result<(), FileSystemError> {
        let fs = self.fs.as_ref().ok_or(FileSystemError::Remove)?;
        let path = to_lfs_path(&path).ok_or(FileSystemError::Remove)?;

        fs.borrow_mut()
            .remove_dir(path)
            .map_err(|_| FileSystemError::Remove)
    }

    async fn remove_all(&mut self, path: Path) -> Result<(), FileSystemError> {
        let fs = self.fs.as_ref().ok_or(FileSystemError::Remove)?;
        let fs = fs.borrow_mut();
        let path = to_lfs_path(&path).ok_or(FileSystemError::Remove)?;
        let metadata = fs.metadata(path).map_err(|_| FileSystemError::RemoveAll)?;

        if metadata.is_file() {
            fs.remove(path).map_err(|_| FileSystemError::RemoveAll)
        } else {
            fs.remove_dir_all(path)
                .map_err(|_| FileSystemError::RemoveAll)
        }
    }

    async fn rename(&mut self, src_path: Path, dest_path: Path) -> Result<(), FileSystemError> {
        let fs = self.fs.as_ref().ok_or(FileSystemError::Remove)?;
        let src_path = to_lfs_path(&src_path).ok_or(FileSystemError::Remove)?;
        let dest_path = to_lfs_path(&dest_path).ok_or(FileSystemError::Remove)?;

        fs.borrow_mut()
            .rename(src_path, dest_path)
            .map_err(|_| FileSystemError::Rename)
    }
}

// Helpers
fn to_lfs_path(path: &Path) -> Option<&littlefs2::path::Path> {
    littlefs2::path::Path::from_str_with_nul(path.as_str_with_nul()).ok()
}

fn to_lfs_seek(pos: embedded_io_async::SeekFrom) -> Option<littlefs2::io::SeekFrom> {
    match pos {
        embedded_io_async::SeekFrom::Start(n) => {
            u32::try_from(n).map(littlefs2::io::SeekFrom::Start).ok()
        }
        embedded_io_async::SeekFrom::End(n) => {
            i32::try_from(n).map(littlefs2::io::SeekFrom::End).ok()
        }
        embedded_io_async::SeekFrom::Current(n) => {
            i32::try_from(n).map(littlefs2::io::SeekFrom::Current).ok()
        }
    }
}
