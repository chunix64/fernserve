use crate::{
    drivers::littlefs::{LittleFsFile, LittleFsFileSystem, LittleFsStorage},
    models::filesystem::{File, FileSystem},
};

use crate::ESP_LITTLEFS_PARTITION_SIZE;

pub type InternalFlash =
    LittleFsStorage<esp_storage::FlashStorage<'static>, ESP_LITTLEFS_PARTITION_SIZE>;

pub enum AnyFile {
    LittleFsFile(LittleFsFile<'static, InternalFlash>),
}

pub enum AnyFs {
    LittleFs(LittleFsFileSystem<'static, InternalFlash>),
}

// Impl File
impl embedded_io::ErrorType for AnyFile {
    type Error = crate::models::common::FileError;
}

impl File for AnyFile {
    fn close(self) -> Result<(), crate::models::common::FileError> {
        match self {
            Self::LittleFsFile(file) => file.close(),
        }
    }
}

impl embedded_io::Read for AnyFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        match self {
            Self::LittleFsFile(file) => file.read(buf),
        }
    }
}

impl embedded_io::Write for AnyFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        match self {
            Self::LittleFsFile(file) => file.write(buf),
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        match self {
            Self::LittleFsFile(file) => file.flush(),
        }
    }
}

impl embedded_io::Seek for AnyFile {
    fn seek(&mut self, pos: embedded_io::SeekFrom) -> Result<u64, Self::Error> {
        match self {
            Self::LittleFsFile(file) => file.seek(pos),
        }
    }
}

// impl FileSystem
impl FileSystem for AnyFs {
    type File = AnyFile;

    fn format(&mut self) -> Result<(), crate::models::common::FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.format(),
        }
    }

    fn mount(&mut self) -> Result<(), crate::models::common::FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.mount(),
        }
    }

    fn is_mountable(&mut self) -> Result<bool, crate::models::common::FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.is_mountable(),
        }
    }

    fn open(
        &mut self,
        path: crate::models::path::Path,
    ) -> Result<<AnyFs as FileSystem>::File, crate::models::common::FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.open(path).map(AnyFile::LittleFsFile),
        }
    }

    fn remove(
        &mut self,
        path: crate::models::path::Path,
    ) -> Result<(), crate::models::common::FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.remove(path),
        }
    }

    fn rename(
        &mut self,
        src_path: crate::models::path::Path,
        dest_path: crate::models::path::Path,
    ) -> Result<(), crate::models::common::FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.rename(src_path, dest_path),
        }
    }

    fn read_dir(
        &mut self,
        path: crate::models::path::Path,
    ) -> Result<
        alloc::vec::Vec<crate::models::filesystem::FileSystemNode>,
        crate::models::common::FileSystemError,
    > {
        match self {
            Self::LittleFs(fs) => fs.read_dir(path),
        }
    }

    fn create_dir(
        &mut self,
        path: crate::models::path::Path,
    ) -> Result<(), crate::models::common::FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.create_dir(path),
        }
    }

    fn create_file(
        &mut self,
        path: crate::models::path::Path,
    ) -> Result<(), crate::models::common::FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.create_file(path),
        }
    }
}
