use alloc::vec::Vec;

use crate::{
    drivers::littlefs::{LittleFsFile, LittleFsFileSystem, LittleFsStorage},
    models::{
        common::{FileError, FileSystemError},
        filesystem::{File, FileSystem, FileSystemNode},
        path::Path,
    },
};

use crate::ESP_LITTLEFS_PARTITION_SIZE;

// macros
macro_rules! forward_fs {
    (@one [$($variant:ident),+]
        fn $name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;
        $call:tt
    ) => {
        fn $name(&mut self, $($arg: $ty),*) -> $ret {
            match self {
                $(Self::$variant(fs) => fs.$name $call,)+
            }
        }
    };

    ($vs:tt => $(fn $name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(
            forward_fs!(@one $vs
                fn $name($($arg: $ty),*) -> $ret;
                ($($arg),*)
            );
        )*
    };
}

macro_rules! impl_any_file {
    ($($variant:ident),+ $(,)?) => {
        impl embedded_io::ErrorType for AnyFile {
            type Error = FileError;
        }

        impl File for AnyFile {
            fn close(self) -> Result<(), FileError> {
                match self {
                    $(Self::$variant(f) => File::close(f),)+
                }
            }
        }

        impl embedded_io::Read for AnyFile {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
                match self {
                    $(Self::$variant(f) => embedded_io::Read::read(f, buf),)+
                }
            }
        }

        impl embedded_io::Write for AnyFile {
            fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
                match self {
                    $(Self::$variant(f) => embedded_io::Write::write(f, buf),)+
                }
            }

            fn flush(&mut self) -> Result<(), Self::Error> {
                match self {
                    $(Self::$variant(f) => embedded_io::Write::flush(f),)+
                }
            }
        }

        impl embedded_io::Seek for AnyFile {
            fn seek(&mut self, pos: embedded_io::SeekFrom) -> Result<u64, Self::Error> {
                match self {
                    $(Self::$variant(f) => embedded_io::Seek::seek(f, pos),)+
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
        fn format() -> Result<(), FileSystemError>;
        fn mount() -> Result<(), FileSystemError>;
        fn is_mountable() -> Result<bool, FileSystemError>;
        fn exist(path: Path) -> Result<bool, FileSystemError>;
        fn available_space() -> Result<usize, FileSystemError>;
        fn total_space() -> Result<usize, FileSystemError>;
        fn read_dir(path: Path) -> Result<Vec<FileSystemNode>, FileSystemError>;
        fn create_dir(path: Path) -> Result<(), FileSystemError>;
        fn create_file(path: Path) -> Result<(), FileSystemError>;
        fn remove(path: Path) -> Result<(), FileSystemError>;
        fn remove_all(path: Path) -> Result<(), FileSystemError>;
        fn rename(src_path: Path, dest_path: Path) -> Result<(), FileSystemError>;
    );

    fn open(&mut self, path: Path) -> Result<Self::File, FileSystemError> {
        match self {
            Self::LittleFs(fs) => fs.open(path).map(AnyFile::LittleFsFile),
        }
    }
}
