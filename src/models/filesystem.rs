use alloc::string::String;

use crate::models::common::{FileError, FileSystemError};

pub trait File<'a>: Sized {
    type FileSystem;
    // fn new(filesystem: Self::FileSystem) -> Result<Self, FileError>;
}

pub trait Dir {}

pub trait FileSystem<'a>: Sized {
    type Storage;
    type File;

    fn new(storage: Self::Storage) -> Result<Self, FileSystemError>;
    fn mount(&'a mut self) -> Result<(), FileSystemError>;
    fn open(&'a mut self, path: String) -> Result<Self::File, FileSystemError>;
}
