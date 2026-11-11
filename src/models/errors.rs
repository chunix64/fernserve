use embedded_io::ErrorKind;
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum VirtualFileSystemError {
    #[error("Operations on /dev are not supported")]
    Dev,
    #[error("Invalid path")]
    InvalidPath,
    #[error("Not implemented yet")]
    NotImplementedYet,
    #[error("Filesystem error: {0}")]
    FileSystemError(#[from] FileSystemError),
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("Failed to format filesystem")]
    Format,
    #[error("Failed to mount filesystem")]
    Mount,
    #[error("Filesystem is not mounted")]
    UnMounted,
    #[error("Failed to get available space")]
    AvailableSpace,
    #[error("Failed to get total space")]
    TotalSpace,
    #[error("Failed to check existence")]
    Exists,
    #[error("Failed to get metadata")]
    Metadata,
    #[error("Failed to open file")]
    Open,
    #[error("Failed to read directory")]
    ReadDir,
    #[error("Failed to create directory")]
    CreateDir,
    #[error("Failed to create file")]
    CreateFile,
    #[error("Failed to remove entry")]
    Remove,
    #[error("Failed to remove directory recursively")]
    RemoveAll,
    #[error("Failed to rename entry")]
    Rename,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum FileError {
    #[error("Filesystem error")]
    FileSystem,
    #[error("Failed to close file")]
    Close,
    #[error("Failed to read file")]
    Read,
    #[error("Failed to flush file")]
    Flush,
    #[error("Failed to write file")]
    Write,
    #[error("Failed to seek in file")]
    Seek,
    #[error("Failed to truncate file")]
    Truncate,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PathError {
    #[error("Path exceeds buffer capacity")]
    Overflow,
}

impl From<PathError> for VirtualFileSystemError {
    fn from(_: PathError) -> Self {
        Self::InvalidPath
    }
}

impl embedded_io::Error for FileError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}
