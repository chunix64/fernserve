use alloc::vec::Vec;
use serde::Serialize;

use crate::models::{
    common::{FileError, FileSystemError},
    path::Path,
};

pub trait File:
    Sized
    + embedded_io_async::ErrorType
    + embedded_io_async::Read
    + embedded_io_async::Write
    + embedded_io_async::Seek
{
    async fn close(self) -> Result<(), FileError>;
}

pub trait FileSystem: Sized {
    type File;

    async fn format(&mut self) -> Result<(), FileSystemError>;
    async fn mount(&mut self) -> Result<(), FileSystemError>;
    async fn is_mountable(&mut self) -> Result<bool, FileSystemError>;
    async fn available_space(&mut self) -> Result<usize, FileSystemError>;
    async fn total_space(&mut self) -> Result<usize, FileSystemError>;
    async fn exist(&mut self, path: Path) -> Result<bool, FileSystemError>;
    async fn metadata(&mut self, path: Path) -> Result<FileSystemMetaData, FileSystemError>;
    async fn open(&mut self, path: Path) -> Result<Self::File, FileSystemError>;
    async fn read_dir(&mut self, path: Path) -> Result<Vec<FileSystemNode>, FileSystemError>;
    async fn create_dir(&mut self, path: Path) -> Result<(), FileSystemError>;
    async fn create_file(&mut self, path: Path) -> Result<(), FileSystemError>;
    async fn remove(&mut self, path: Path) -> Result<(), FileSystemError>;
    async fn remove_all(&mut self, path: Path) -> Result<(), FileSystemError>;
    async fn rename(&mut self, src_path: Path, dest_path: Path) -> Result<(), FileSystemError>;
}

#[derive(Debug, Serialize)]
pub enum FileSystemNodeType {
    Dir,
    File,
}

#[derive(Debug, Serialize)]
pub struct FileSystemNode {
    pub path: Path,
    pub metadata: FileSystemMetaData,
}

#[derive(Debug, Serialize)]
pub struct FileSystemMetaData {
    pub node_type: FileSystemNodeType,
    pub is_dir: bool,
    pub is_empty: bool,
    pub len: usize,
}
