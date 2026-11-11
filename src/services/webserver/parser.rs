use alloc::string::{String, ToString};

use embedded_io_async::{Read, Seek, SeekFrom, Write};
use picoserve::request::RequestBodyReader;
use serde_json::{Value, json};

use crate::{
    configs::{HTTP_BUFFER_SIZE, UPLOAD_CHUNK_SIZE},
    models::{filesystem::File, path::Path},
    services::webserver::errors::WebServerError,
    vfs::VirtualFileSystem,
};

pub struct WebServerParser;

// path specifics
impl WebServerParser {
    pub async fn vfs_exists(
        vfs: &mut VirtualFileSystem,
        path: &str,
    ) -> Result<String, WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        let exist = vfs
            .exists(path)
            .await
            .map_err(|_| WebServerError::VfsExists)?
            .to_string();

        Ok(Self::parse_json(200, "exist", &exist))
    }

    pub async fn vfs_available_space(
        vfs: &mut VirtualFileSystem,
        path: &str,
    ) -> Result<String, WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;

        match vfs.available_space(path).await {
            Ok(available_space) => Ok(Self::parse_json(
                200,
                "available_space",
                &available_space.to_string(),
            )),
            Err(_) => Err(WebServerError::VfsAvailableSpace),
        }
    }

    pub async fn vfs_total_space(
        vfs: &mut VirtualFileSystem,
        path: &str,
    ) -> Result<String, WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;

        match vfs.total_space(path).await {
            Ok(total_space) => Ok(Self::parse_json(
                200,
                "total_space",
                &total_space.to_string(),
            )),
            Err(_) => Err(WebServerError::VfsTotalSpace),
        }
    }

    pub async fn vfs_read_dir(
        vfs: &mut VirtualFileSystem,
        path: &str,
    ) -> Result<String, WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;

        match vfs.read_dir(path).await {
            Ok(entries) => match serde_json::to_string(&entries) {
                Ok(data) => Ok(data),
                Err(_) => Err(WebServerError::Parser),
            },

            Err(_) => Err(WebServerError::VfsReadDir),
        }
    }

    pub async fn vfs_create_dir(
        vfs: &mut VirtualFileSystem,
        path: &str,
    ) -> Result<(), WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        vfs.create_dir(path)
            .await
            .map_err(|_| WebServerError::VfsCreateDir)
    }

    pub async fn vfs_remove(vfs: &mut VirtualFileSystem, path: &str) -> Result<(), WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        vfs.remove(path)
            .await
            .map_err(|_| WebServerError::VfsRemove)
    }

    pub async fn vfs_remove_all(
        vfs: &mut VirtualFileSystem,
        path: &str,
    ) -> Result<(), WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        vfs.remove_all(path)
            .await
            .map_err(|_| WebServerError::VfsRemoveAll)
    }

    pub async fn vfs_rename(
        vfs: &mut VirtualFileSystem,
        src_path: &str,
        dest_path: &str,
    ) -> Result<(), WebServerError> {
        let src_path = Path::new(src_path).map_err(|_| WebServerError::Path)?;
        let dest_path = Path::new(dest_path).map_err(|_| WebServerError::Path)?;

        vfs.rename(src_path, dest_path)
            .await
            .map_err(|_| WebServerError::VfsRename)
    }

    pub async fn vfs_upload_file_raw<R: embedded_io_async::Read>(
        vfs: &mut VirtualFileSystem,
        path: &str,
        mut reader: RequestBodyReader<'_, R>,
    ) -> Result<(), WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        if let Ok(false) = vfs.exists(path).await {
            vfs.create_file(path)
                .await
                .map_err(|_| WebServerError::VfsCreateFile)?;
        }
        let mut file = vfs.open(path).await.map_err(|_| WebServerError::VfsOpen)?;
        let mut buffer = [0u8; HTTP_BUFFER_SIZE];

        loop {
            let written_bytes = reader
                .read(&mut buffer)
                .await
                .map_err(|_| WebServerError::Reader)?;

            if written_bytes == 0 {
                break;
            }

            file.write_all(&buffer[..written_bytes])
                .await
                .map_err(|_| WebServerError::VfsWriteAll)?;
        }

        file.close().await.map_err(|_| WebServerError::VfsClose)
    }

    pub async fn vfs_upload_file_chunk<R: embedded_io_async::Read>(
        vfs: &mut VirtualFileSystem,
        path: &str,
        chunk_index: usize,
        total_chunks: usize,
        mut reader: RequestBodyReader<'_, R>,
    ) -> Result<String, WebServerError> {
        if total_chunks == 0 || chunk_index >= total_chunks {
            return Err(WebServerError::ParseParamKey);
        }

        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        let offset = chunk_index * UPLOAD_CHUNK_SIZE;

        if chunk_index == 0 {
            if let Ok(true) = vfs.exists(path).await {
                vfs.remove(path)
                    .await
                    .map_err(|_| WebServerError::VfsRemove)?;
            }
            vfs.create_file(path)
                .await
                .map_err(|_| WebServerError::VfsCreateFile)?;
        } else if let Ok(false) = vfs.exists(path).await {
            return Err(WebServerError::ParseParamKey); // chunk 0 never arrived
        }

        let mut file = vfs.open(path).await.map_err(|_| WebServerError::VfsOpen)?;

        // A chunk may only start at or before the current end of file
        if chunk_index > 0
            && vfs
                .metadata(path)
                .await
                .map_err(|_| WebServerError::VfsMetadata)?
                .len
                < offset
        {
            return Err(WebServerError::ParseParamKey); // gap: missing chunk
        }

        file.seek(SeekFrom::Start(offset as u64))
            .await
            .map_err(|_| WebServerError::VfsSeek)?;

        let mut buffer = [0u8; HTTP_BUFFER_SIZE];
        let mut written_total = 0;

        loop {
            let n = reader
                .read(&mut buffer)
                .await
                .map_err(|_| WebServerError::Reader)?;
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n])
                .await
                .map_err(|_| WebServerError::VfsWriteAll)?;
            written_total += n;
        }

        if chunk_index + 1 == total_chunks {
            file.truncate(offset + written_total)
                .await
                .map_err(|_| WebServerError::VfsTruncate)?;
        }

        file.close().await.map_err(|_| WebServerError::VfsClose)?;

        Ok(Self::parse_json(
            200,
            "chunk_index",
            &chunk_index.to_string(),
        ))
    }
}

// helpers
impl WebServerParser {
    pub fn parse_query_key(query: &str, key: &str) -> Result<Option<String>, WebServerError> {
        let query = query.strip_prefix('?').unwrap_or(query);
        Ok(form_urlencoded::parse(query.as_bytes())
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.into_owned()))
    }

    pub fn parse_json_key(body: &str, key: &str) -> Result<String, WebServerError> {
        let value: Value = serde_json::from_str(body).map_err(|_| WebServerError::Parser)?;

        Ok(value[key]
            .as_str()
            .ok_or(WebServerError::Parser)?
            .to_string())
    }

    pub fn parse_json(status_code: u16, key: &str, value: &str) -> String {
        let json = json!({
            "status": status_code,
            key: value
        });

        json.to_string()
    }

    pub fn parse_error_json(status_code: u16, error: &str) -> String {
        let json = json!({
            "status": status_code,
            "error": error
        });

        json.to_string()
    }
}
