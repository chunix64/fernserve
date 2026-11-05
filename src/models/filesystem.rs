use alloc::vec::Vec;
use serde::Serialize;

use crate::models::{
    common::{FileError, FileSystemError},
    path::Path,
};

pub trait File:
    Sized + embedded_io::ErrorType + embedded_io::Read + embedded_io::Write + embedded_io::Seek
{
    fn close(self) -> Result<(), FileError>;
}

pub trait FileSystem: Sized {
    type File;

    fn format(&mut self) -> Result<(), FileSystemError>;
    fn mount(&mut self) -> Result<(), FileSystemError>;
    fn open(&mut self, path: Path) -> Result<Self::File, FileSystemError>;
    fn is_mountable(&mut self) -> Result<bool, FileSystemError>;
    fn read_dir(&mut self, path: Path) -> Result<Vec<FileSystemNode>, FileSystemError>;
    fn create_dir(&mut self, path: Path) -> Result<(), FileSystemError>;
    fn create_file(&mut self, path: Path) -> Result<(), FileSystemError>;
    fn remove(&mut self, path: Path) -> Result<(), FileSystemError>;
    fn rename(&mut self, src_path: Path, dest_path: Path) -> Result<(), FileSystemError>;
}

#[derive(Debug, Serialize)]
pub enum FileSystemNodeType {
    Dir,
    File,
}

#[derive(Debug, Serialize)]
pub struct FileSystemNode {
    pub path: Path,
    pub node_type: FileSystemNodeType,
}
