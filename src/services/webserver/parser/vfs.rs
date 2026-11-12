use alloc::string::{String, ToString};

use anyhow::{Result, anyhow, bail};
use embedded_io_async::{Read, Seek, SeekFrom, Write};
use picoserve::request::RequestBodyReader;

use super::WebServerParser;
use crate::{
    configs::{HTTP_BUFFER_SIZE, UPLOAD_CHUNK_SIZE},
    models::{filesystem::File, path::Path},
    vfs::VirtualFileSystem,
};

impl WebServerParser {
    pub async fn vfs_exists(vfs: &mut VirtualFileSystem, path: &str) -> Result<String> {
        let path = Path::new(path)?;
        let exist = vfs.exists(path).await?.to_string();

        Ok(Self::parse_json(200, "exist", &exist))
    }

    pub async fn vfs_available_space(vfs: &mut VirtualFileSystem, path: &str) -> Result<String> {
        let path = Path::new(path)?;
        let available_space = vfs.available_space(path).await?;

        Ok(Self::parse_json(
            200,
            "available_space",
            &available_space.to_string(),
        ))
    }

    pub async fn vfs_total_space(vfs: &mut VirtualFileSystem, path: &str) -> Result<String> {
        let path = Path::new(path)?;
        let total_space = vfs.total_space(path).await?;

        Ok(Self::parse_json(
            200,
            "total_space",
            &total_space.to_string(),
        ))
    }

    pub async fn vfs_read_dir(vfs: &mut VirtualFileSystem, path: &str) -> Result<String> {
        let path = Path::new(path)?;
        let entries = vfs.read_dir(path).await?;

        Ok(serde_json::to_string(&entries)?)
    }

    pub async fn vfs_create_dir(vfs: &mut VirtualFileSystem, path: &str) -> Result<()> {
        let path = Path::new(path)?;
        vfs.create_dir(path).await?;

        Ok(())
    }

    pub async fn vfs_remove(vfs: &mut VirtualFileSystem, path: &str) -> Result<()> {
        let path = Path::new(path)?;
        vfs.remove(path).await?;

        Ok(())
    }

    pub async fn vfs_remove_all(vfs: &mut VirtualFileSystem, path: &str) -> Result<()> {
        let path = Path::new(path)?;
        vfs.remove_all(path).await?;

        Ok(())
    }

    pub async fn vfs_rename(
        vfs: &mut VirtualFileSystem,
        src_path: &str,
        dest_path: &str,
    ) -> Result<()> {
        let src_path = Path::new(src_path)?;
        let dest_path = Path::new(dest_path)?;
        vfs.rename(src_path, dest_path).await?;

        Ok(())
    }

    pub async fn vfs_upload_file_raw<R: embedded_io_async::Read>(
        vfs: &mut VirtualFileSystem,
        path: &str,
        mut reader: RequestBodyReader<'_, R>,
    ) -> Result<()> {
        let path = Path::new(path)?;

        if let Ok(false) = vfs.exists(path).await {
            vfs.create_file(path).await?;
        }

        let mut file = vfs.open(path).await?;
        let mut buffer = [0u8; HTTP_BUFFER_SIZE];

        loop {
            let written_bytes = reader
                .read(&mut buffer)
                .await
                .map_err(|_| anyhow!("Reader failed to read"))?;

            if written_bytes == 0 {
                break;
            }

            file.write_all(&buffer[..written_bytes]).await?;
        }

        file.close().await?;

        Ok(())
    }

    pub async fn vfs_upload_file_chunk<R: embedded_io_async::Read>(
        vfs: &mut VirtualFileSystem,
        path: &str,
        chunk_index: usize,
        total_chunks: usize,
        mut reader: RequestBodyReader<'_, R>,
    ) -> Result<String> {
        if total_chunks == 0 || chunk_index >= total_chunks {
            bail!("Chunks configs are invalid")
        }

        let path = Path::new(path)?;
        let offset = chunk_index * UPLOAD_CHUNK_SIZE;

        if chunk_index == 0 {
            if let Ok(true) = vfs.exists(path).await {
                vfs.remove(path).await?;
            }

            vfs.create_file(path).await?;
        } else if let Ok(false) = vfs.exists(path).await {
            bail!("Chunk 0 never arrived")
        }

        let mut file = vfs.open(path).await?;

        // A chunk may only start at or before the current end of file
        if chunk_index > 0 && vfs.metadata(path).await?.len < offset {
            bail!("Chunk missing")
        }

        file.seek(SeekFrom::Start(offset as u64)).await?;

        let mut buffer = [0u8; HTTP_BUFFER_SIZE];
        let mut written_total = 0;

        loop {
            let n = reader
                .read(&mut buffer)
                .await
                .map_err(|_| anyhow!("Reader failed to read"))?;

            if n == 0 {
                break;
            }

            file.write_all(&buffer[..n]).await?;
            written_total += n;
        }

        if chunk_index + 1 == total_chunks {
            file.truncate(offset + written_total).await?;
        }

        file.close().await?;

        Ok(Self::parse_json(
            200,
            "chunk_index",
            &chunk_index.to_string(),
        ))
    }
}
