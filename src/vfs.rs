use alloc::vec::Vec;

use crate::{
    drivers::anyfs::AnyFs,
    models::{
        common::VirtualFileSystemError,
        filesystem::{FileSystem, FileSystemNode},
        path::Path,
    },
};

pub struct VirtualFileSystem {
    root_filesystem: AnyFs,
    mount_points: Vec<(&'static str, AnyFs)>,
}

impl VirtualFileSystem {
    pub fn new(root_filesystem: AnyFs) -> Result<Self, VirtualFileSystemError> {
        Ok(Self {
            root_filesystem,
            mount_points: Vec::new(),
        })
    }

    pub fn open(
        &mut self,
        path: Path,
    ) -> Result<<AnyFs as FileSystem>::File, VirtualFileSystemError> {
        match self.get_filesystem(path) {
            Ok((fs, fs_path, _)) => fs.open(fs_path).map_err(|_| VirtualFileSystemError::Open),
            Err(error) => Err(error),
        }
    }

    pub fn read_dir(&mut self, path: Path) -> Result<Vec<FileSystemNode>, VirtualFileSystemError> {
        match self.get_filesystem(path) {
            Ok((fs, fs_path, _)) => fs
                .read_dir(fs_path)
                .map_err(|_| VirtualFileSystemError::ReadDir),
            Err(error) => Err(error),
        }
    }

    pub fn create_dir(&mut self, path: Path) -> Result<(), VirtualFileSystemError> {
        match self.get_filesystem(path) {
            Ok((fs, fs_path, _)) => fs
                .create_dir(fs_path)
                .map_err(|_| VirtualFileSystemError::CreateDir),
            Err(error) => Err(error),
        }
    }

    pub fn create_file(&mut self, path: Path) -> Result<(), VirtualFileSystemError> {
        match self.get_filesystem(path) {
            Ok((fs, fs_path, _)) => fs
                .create_file(fs_path)
                .map_err(|_| VirtualFileSystemError::CreateFile),
            Err(error) => Err(error),
        }
    }

    pub fn remove(&mut self, path: Path) -> Result<(), VirtualFileSystemError> {
        match self.get_filesystem(path) {
            Ok((fs, fs_path, _)) => fs
                .remove(fs_path)
                .map_err(|_| VirtualFileSystemError::Remove),
            Err(error) => Err(error),
        }
    }

    pub fn rename(
        &mut self,
        src_path: Path,
        dest_path: Path,
    ) -> Result<(), VirtualFileSystemError> {
        match self.get_filesystem(src_path) {
            Ok((_src_fs, fs_src_path, src_mount_index)) => match self.get_filesystem(dest_path) {
                Ok((dest_fs, fs_dest_path, dest_mount_index)) => {
                    if src_mount_index == dest_mount_index {
                        return dest_fs
                            .rename(fs_src_path, fs_dest_path)
                            .map_err(|_| VirtualFileSystemError::Rename);
                    }

                    Err(VirtualFileSystemError::NotImplementedYet)
                }
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        }
    }

    // Helpers
    fn get_filesystem(
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
