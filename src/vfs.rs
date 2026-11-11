use alloc::vec::Vec;

use crate::{
    drivers::anyfs::AnyFs,
    models::{
        common::VirtualFileSystemError,
        filesystem::{FileSystem, FileSystemMetaData, FileSystemNode},
        path::Path,
    },
};

pub struct VirtualFileSystem {
    root_filesystem: AnyFs,
    mount_points: Vec<(&'static str, AnyFs)>,
}

impl VirtualFileSystem {
    pub async fn new(mut root_filesystem: AnyFs) -> Self {
        if let Ok(path) = Path::new("/dev") {
            // Try to create the system folder /dev
            let _ = root_filesystem.create_dir(path).await;
        }

        Self {
            root_filesystem,
            mount_points: Vec::new(),
        }
    }

    pub async fn open(
        &mut self,
        path: Path,
    ) -> Result<<AnyFs as FileSystem>::File, VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, fs_path, _)) => fs
                .open(fs_path)
                .await
                .map_err(|_| VirtualFileSystemError::Open),
            Err(error) => Err(error),
        }
    }

    pub async fn exists(&mut self, path: Path) -> Result<bool, VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, fs_path, _)) => fs
                .exists(fs_path)
                .await
                .map_err(|_| VirtualFileSystemError::Exist),
            Err(error) => Err(error),
        }
    }

    pub async fn metadata(
        &mut self,
        path: Path,
    ) -> Result<FileSystemMetaData, VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, fs_path, _)) => fs
                .metadata(fs_path)
                .await
                .map_err(|_| VirtualFileSystemError::Metadata),
            Err(error) => Err(error),
        }
    }

    pub async fn available_space(&mut self, path: Path) -> Result<usize, VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, _, _)) => fs
                .available_space()
                .await
                .map_err(|_| VirtualFileSystemError::Metadata),
            Err(error) => Err(error),
        }
    }

    pub async fn total_space(&mut self, path: Path) -> Result<usize, VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, _, _)) => fs
                .total_space()
                .await
                .map_err(|_| VirtualFileSystemError::Metadata),
            Err(error) => Err(error),
        }
    }

    pub async fn read_dir(
        &mut self,
        path: Path,
    ) -> Result<Vec<FileSystemNode>, VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, fs_path, _)) => fs
                .read_dir(fs_path)
                .await
                .map_err(|_| VirtualFileSystemError::ReadDir),
            Err(VirtualFileSystemError::Dev) => self
                .root_filesystem
                .read_dir(Path::new("/dev").map_err(|_| VirtualFileSystemError::ReadDir)?)
                .await
                .map_err(|_| VirtualFileSystemError::ReadDir),
            Err(error) => Err(error),
        }
    }

    pub async fn create_dir(&mut self, path: Path) -> Result<(), VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, fs_path, _)) => fs
                .create_dir(fs_path)
                .await
                .map_err(|_| VirtualFileSystemError::CreateDir),
            Err(error) => Err(error),
        }
    }

    pub async fn create_file(&mut self, path: Path) -> Result<(), VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, fs_path, _)) => fs
                .create_file(fs_path)
                .await
                .map_err(|_| VirtualFileSystemError::CreateFile),
            Err(error) => Err(error),
        }
    }

    pub async fn remove(&mut self, path: Path) -> Result<(), VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, fs_path, _)) => fs
                .remove(fs_path)
                .await
                .map_err(|_| VirtualFileSystemError::Remove),
            Err(error) => Err(error),
        }
    }

    pub async fn remove_all(&mut self, path: Path) -> Result<(), VirtualFileSystemError> {
        match self.get_filesystem(path).await {
            Ok((fs, fs_path, _)) => fs
                .remove_all(fs_path)
                .await
                .map_err(|_| VirtualFileSystemError::RemoveAll),
            Err(error) => Err(error),
        }
    }

    pub async fn rename(
        &mut self,
        src_path: Path,
        dest_path: Path,
    ) -> Result<(), VirtualFileSystemError> {
        let (_src_fs, fs_src_path, src_mount_index) = self.get_filesystem(src_path).await?;
        let (dest_fs, fs_dest_path, dest_mount_index) = self.get_filesystem(dest_path).await?;

        if src_mount_index == dest_mount_index {
            return dest_fs
                .rename(fs_src_path, fs_dest_path)
                .await
                .map_err(|_| VirtualFileSystemError::Rename);
        }

        // TODO: Moving file between filesystems
        Err(VirtualFileSystemError::NotImplementedYet)
    }
}

// Helpers
impl VirtualFileSystem {
    async fn get_filesystem(
        &mut self,
        path: Path,
    ) -> Result<(&mut AnyFs, Path, Option<usize>), VirtualFileSystemError> {
        if path.as_str() == "/dev" {
            return Err(VirtualFileSystemError::Dev);
        }

        let mut components = path.components();

        if components.is_empty() {
            return Err(VirtualFileSystemError::InvalidPath);
        }

        if components.len() > 1
            && components[0] == "dev"
            && let Some((mount_index, (_, target_filesystem))) = self
                .mount_points
                .iter_mut()
                .enumerate()
                .find(|(_, (key, _))| *key == components[1])
        {
            components.remove(0);

            let path = Path::from_components(components)
                .map_err(|_| VirtualFileSystemError::InvalidPath)?;

            return Ok((target_filesystem, path, Some(mount_index)));
        }

        Ok((&mut self.root_filesystem, path, None))
    }
}
