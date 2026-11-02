use core::cell::RefCell;

use alloc::{borrow::ToOwned, rc::Rc, vec::Vec};

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

type InnerFile<'a, T, const SIZE: usize> = littlefs2::fs::File<'a, 'a, LittleFsStorage<T, SIZE>>;
type InnerFileAlloc<T, const SIZE: usize> = littlefs2::fs::FileAllocation<LittleFsStorage<T, SIZE>>;

// Storage
pub struct LittleFsStorage<T: Flash, const SIZE: usize> {
    storage: T,
}

impl<T: Flash, const SIZE: usize> LittleFsStorage<T, SIZE> {
    fn new(storage: T) -> Self {
        Self { storage }
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
pub struct LittleFsFile<'a, T: Flash, const SIZE: usize> {
    pub file: littlefs2::fs::File<'a, 'a, LittleFsStorage<T, SIZE>>,
}

impl<'a, T: Flash, const SIZE: usize> LittleFsFile<'a, T, SIZE> {
    pub fn new(
        path: Path,
        filesystem: &'a mut littlefs2::fs::Filesystem<'a, LittleFsStorage<T, SIZE>>,
        file_alloc: &'a mut littlefs2::fs::FileAllocation<LittleFsStorage<T, SIZE>>,
    ) -> Result<Self, FileError> {
        let path = match to_lfs_path(&path) {
            Some(path) => path,
            None => return Err(FileError::FileSytem),
        };

        // SAFETY: FatFS library has already exposed the unsafe open_file() as safe anyway
        // Just manually close the file to prevent undefined behaviors

        unsafe {
            let file = match littlefs2::fs::File::open(filesystem, file_alloc, path) {
                Ok(file) => file,
                Err(_) => return Err(FileError::FileSytem),
            };

            Ok(Self { file })
        }
    }
}

impl<'a, T: Flash, const SIZE: usize> crate::models::filesystem::File
    for LittleFsFile<'a, T, SIZE>
{
}

// Filesystem
pub struct LittleFsFileSystem<'a, T: Flash, const SIZE: usize> {
    files: Vec<Rc<RefCell<Heap<LittleFsFile<'a, T, SIZE>>>>>,
    fs: Option<Rc<RefCell<littlefs2::fs::Filesystem<'a, LittleFsStorage<T, SIZE>>>>>,
    alloc: littlefs2::fs::Allocation<LittleFsStorage<T, SIZE>>,
    storage: LittleFsStorage<T, SIZE>,
}

impl<'a, T: Flash, const SIZE: usize> crate::models::filesystem::FileSystem
    for LittleFsFileSystem<'a, T, SIZE>
where
    T: littlefs2::driver::Storage,
{
    type Storage = T;
    type File = Rc<RefCell<Heap<LittleFsFile<'a, T, SIZE>>>>;

    fn new(storage: Self::Storage) -> Result<Self, FileSystemError> {
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
            files: Vec::new(),
        })
    }

    fn format(&mut self) -> Result<(), FileSystemError> {
        match littlefs2::fs::Filesystem::format(&mut self.storage) {
            Ok(()) => Ok(()),
            Err(_) => Err(FileSystemError::Format),
        }
    }

    fn mount(&mut self) -> Result<(), FileSystemError> {
        // SAFETY: `fs` borrows `alloc` and `storage`, which live in the same struct.
        // The extended borrows are only valid as long as `self` is not moved or
        // dropped while `fs` is alive.
        let (alloc, storage) = unsafe {
            (
                &mut *(&mut self.alloc as *mut _),
                &mut *(&mut self.storage as *mut _),
            )
        };

        let Ok(fs) = littlefs2::fs::Filesystem::mount(alloc, storage) else {
            return Err(FileSystemError::Mount);
        };

        self.fs = Some(Rc::new(RefCell::new(fs)));

        Ok(())
    }

    fn is_mountable(&mut self) -> Result<bool, FileSystemError> {
        match littlefs2::fs::Filesystem::is_mountable(&mut self.storage) {
            true => Ok(true),
            false => Err(FileSystemError::UnMountable),
        }
    }

    fn open(&mut self, path: Path) -> Result<Self::File, FileSystemError> {
        let Some(fs) = &self.fs else {
            return Err(FileSystemError::UnMounted);
        };

        unsafe {
            let fs_ref: &'a mut littlefs2::fs::Filesystem<'a, LittleFsStorage<T, SIZE>> =
                &mut *fs.as_ptr();
            let file_alloc = Heap::new(littlefs2::fs::File::allocate());
            let Ok(file) = LittleFsFile::new(path, fs_ref, &mut *file_alloc.as_ptr()) else {
                return Err(FileSystemError::File);
            };

            let file_rc = Rc::new(RefCell::new(Heap::new(file)));
            self.files.push(Rc::clone(&file_rc));
            Ok(Rc::clone(&file_rc))
        }
    }

    fn read_dir(&mut self, path: Path) -> Result<Vec<FileSystemNode>, FileSystemError> {
        if let Some(fs) = &self.fs {
            let fs = fs.borrow_mut();
            let path = match to_lfs_path(&path) {
                Some(path) => path,
                None => return Err(FileSystemError::ReadDir),
            };

            match fs.read_dir_and_then(path, |read_dir| {
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

                    nodes.push(FileSystemNode {
                        path: Path::new(&path),
                        node_type,
                    });
                }

                Ok(nodes)
            }) {
                Ok(nodes) => return Ok(nodes),
                Err(_) => return Err(FileSystemError::ReadDir),
            }
        }

        Err(FileSystemError::ReadDir)
    }
}

// Helpers
fn to_lfs_path(path: &Path) -> Option<&littlefs2::path::Path> {
    littlefs2::path::Path::from_str_with_nul(path.as_str()).ok()
}
