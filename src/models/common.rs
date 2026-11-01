#[non_exhaustive]
pub enum StorageError {}

#[non_exhaustive]
pub enum FileSystemError {
    Format,
    Mount,
    UnMounted,
    File,
}

#[non_exhaustive]
pub enum FileError {}
