use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum WebServerError {
    Parser,
    Path,
    ParseJsonKey,
    ParseParamKey,
    Reader,
    VfsOpen,
    VfsClose,
    VfsExists,
    VfsAvailableSpace,
    VfsTotalSpace,
    VfsReadDir,
    VfsCreateDir,
    VfsCreateFile,
    VfsRemove,
    VfsRemoveAll,
    VfsRename,
    VfsWriteAll,
    VfsMetadata,
    VfsSeek,
    VfsTruncate,
}

macro_rules! impl_display {
    ($type:ty) => {
        impl core::fmt::Display for $type {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(formatter, "{self:?}")
            }
        }
    };
}

impl_display!(WebServerError);
