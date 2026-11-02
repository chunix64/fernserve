#[non_exhaustive]
pub enum StorageError {}

#[non_exhaustive]
pub enum FileSystemError {
    Format,
    Mount,
    UnMounted,
    UnMountable,
    File,
    ReadDir,
}

#[non_exhaustive]
pub enum FileError {
    FileSytem,
}
