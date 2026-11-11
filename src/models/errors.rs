use core::fmt;
use embedded_io::ErrorKind;

#[non_exhaustive]
#[derive(Debug)]
pub enum VirtualFileSystemError {
    Dev, // Operations on /dev
    InvalidPath,
    Open,
    Exist,
    Metadata,
    ReadDir,
    CreateDir,
    CreateFile,
    Remove,
    RemoveAll,
    Rename,
    NotImplementedYet,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum FileSystemError {
    Format,
    Mount,
    UnMounted,
    AvailableSpace,
    TotalSpace,
    Exist,
    Metadata,
    Open,
    ReadDir,
    CreateDir,
    CreateFile,
    Remove,
    RemoveAll,
    Rename,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum FileError {
    FileSystem,
    Close,
    Read,
    Flush,
    Write,
    Seek,
    Truncate,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum PathError {
    Overflow,
}

impl fmt::Display for FileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::FileSystem => formatter.write_str("FileSystem"),
            FileError::Close => formatter.write_str("Close"),
            FileError::Read => formatter.write_str("Read"),
            FileError::Flush => formatter.write_str("Flush"),
            FileError::Write => formatter.write_str("Write"),
            FileError::Seek => formatter.write_str("Seek"),
            FileError::Truncate => formatter.write_str("Truncate"),
        }
    }
}

impl core::error::Error for FileError {}

impl embedded_io::Error for FileError {
    fn kind(&self) -> ErrorKind {
        match self {
            FileError::FileSystem => ErrorKind::Other,
            FileError::Close => ErrorKind::Other,
            FileError::Read => ErrorKind::Other,
            FileError::Flush => ErrorKind::Other,
            FileError::Write => ErrorKind::Other,
            FileError::Seek => ErrorKind::Other,
            FileError::Truncate => ErrorKind::Other,
        }
    }
}
