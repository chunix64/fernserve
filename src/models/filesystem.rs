use alloc::vec::Vec;

use crate::models::{common::FileSystemError, path::Path};

pub trait File: Sized {}

pub trait Dir {}

pub trait FileSystem: Sized {
    type Storage;
    type File;

    fn new(storage: Self::Storage) -> Result<Self, FileSystemError>;
    fn format(&mut self) -> Result<(), FileSystemError>;
    fn mount(&mut self) -> Result<(), FileSystemError>;
    fn open(&mut self, path: Path) -> Result<Self::File, FileSystemError>;
    fn is_mountable(&mut self) -> Result<bool, FileSystemError>;
    fn read_dir(&mut self, path: Path) -> Result<Vec<FileSystemNode>, FileSystemError>;
    fn create_dir(&mut self, path: Path) -> Result<(), FileSystemError>;
}

pub enum FileSystemNodeType {
    Dir,
    File,
}

pub struct FileSystemNode {
    pub path: Path,
    pub node_type: FileSystemNodeType,
}
