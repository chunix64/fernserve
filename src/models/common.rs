#[non_exhaustive]
pub enum StorageError {}

#[non_exhaustive]
#[derive(Debug)]
pub enum FileSystemError {
    Format,
    Mount,
    UnMounted,
    UnMountable,
    File,
    ReadDir,
    CreateDir,
}

#[non_exhaustive]
pub enum FileError {
    FileSytem,
}
