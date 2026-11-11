use alloc::vec::Vec;

use crate::{
    drivers::littlefs::{LittleFsFile, LittleFsFileSystem, LittleFsStorage},
    models::{
        common::{FileError, FileSystemError},
        filesystem::{File, FileSystem, FileSystemMetaData, FileSystemNode},
        path::Path,
    },
};

use crate::ESP_LITTLEFS_PARTITION_SIZE;

// macros
macro_rules! forward_fs {
    (@one [$($variant:ident),+]
        async fn $name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;
        $call:tt
    ) => {
        async fn $name(&mut self, $($arg: $ty),*) -> $ret {
            match self {
                $(Self::$variant(fs) => fs.$name $call.await,)+
            }
        }
    };

    ($vs:tt => $(async fn $name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(
            forward_fs!(@one $vs
                async fn $name($($arg: $ty),*) -> $ret;
                ($($arg),*)
            );
        )*
    };
}

macro_rules! impl_any_file {
    ($($variant:ident),+ $(,)?) => {
        impl embedded_io_async::ErrorType for AnyFile {
            type Error = FileError;
        }

        impl File for AnyFile {
            async fn close(self) -> Result<(), FileError> {
                match self {
                    $(Self::$variant(file) => file.close().await,)+
                }
            }
        }

        impl embedded_io_async::Read for AnyFile {
            async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
                match self {
                    $(Self::$variant(file) => file.read(buffer).await,)+
                }
            }
        }

        impl embedded_io_async::Write for AnyFile {
            async fn write(&mut self, buffer: &[u8]) -> Result<usize, Self::Error> {
                match self {
                    $(Self::$variant(file) => file.write(buffer).await,)+
                }
            }

            async fn flush(&mut self) -> Result<(), Self::Error> {
                match self {
                    $(Self::$variant(file) => file.flush().await,)+
                }
            }
        }

        impl embedded_io_async::Seek for AnyFile {
            async fn seek(&mut self, pos: embedded_io_async::SeekFrom) -> Result<u64, Self::Error> {
                match self {
                    $(Self::$variant(file) => file.seek(pos).await,)+
                }
            }
        }
    };
}

// Defines
pub type InternalFlash =
    LittleFsStorage<esp_storage::FlashStorage<'static>, ESP_LITTLEFS_PARTITION_SIZE>;

pub enum AnyFile {
    LittleFsFile(LittleFsFile<'static, InternalFlash>),
}

pub enum AnyFs {
    LittleFs(LittleFsFileSystem<'static, InternalFlash>),
}

// impl
impl_any_file!(LittleFsFile);

impl FileSystem for AnyFs {
    type File = AnyFile;

    forward_fs!([LittleFs] =>
        async fn format() -> Result<(), FileSystemError>;
        async fn mount() -> Result<(), FileSystemError>;
        async fn is_mountable() -> Result<bool, FileSystemError>;
        async fn exists(path: Path) -> Result<bool, FileSystemError>;
        async fn metadata(path: Path) -> Result<FileSystemMetaData, FileSystemError>;
        async fn available_space() -> Result<usize, FileSystemError>;
        async fn total_space() -> Result<usize, FileSystemError>;
        async fn read_dir(path: Path) -> Result<Vec<FileSystemNode>, FileSystemError>;
        async fn create_dir(path: Path) -> Result<(), FileSystemError>;
        async fn create_file(path: Path) -> Result<(), FileSystemError>;
        async fn remove(path: Path) -> Result<(), FileSystemError>;
        async fn remove_all(path: Path) -> Result<(), FileSystemError>;
        async fn rename(src_path: Path, dest_path: Path) -> Result<(), FileSystemError>;
    );

    async fn open(&mut self, path: Path) -> Result<Self::File, FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.open(path).await.map(AnyFile::LittleFsFile),
        }
    }
}
