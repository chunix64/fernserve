use alloc::string::{String, ToString};
use embedded_io::Write;
use embedded_io_async::Read;
use picoserve::request::RequestBodyReader;
use serde_json::{Value, json};

use crate::{
    configs::HTTP_REQUEST_CHUNK_SIZE,
    models::{filesystem::File, path::Path},
    services::webserver::WebServerError,
    vfs::VirtualFileSystem,
};

pub struct WebServerParser;

// path specifics
impl WebServerParser {
    pub fn vfs_exist(vfs: &mut VirtualFileSystem, path: &str) -> Result<String, WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        let exist = vfs
            .exist(path)
            .map_err(|_| WebServerError::Parser)?
            .to_string();

        Ok(Self::parse_json(200, "exist", &exist))
    }

    pub fn vfs_read_dir(vfs: &mut VirtualFileSystem, path: &str) -> Result<String, WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;

        match vfs.read_dir(path) {
            Ok(entries) => match serde_json::to_string(&entries) {
                Ok(data) => Ok(data),
                Err(_) => Err(WebServerError::Parser),
            },

            Err(_) => Err(WebServerError::Parser),
        }
    }

    pub fn vfs_create_dir(vfs: &mut VirtualFileSystem, path: &str) -> Result<(), WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        vfs.create_dir(path).map_err(|_| WebServerError::Parser)
    }

    pub fn vfs_remove(vfs: &mut VirtualFileSystem, path: &str) -> Result<(), WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        vfs.remove(path).map_err(|_| WebServerError::Parser)
    }

    pub fn vfs_remove_all(vfs: &mut VirtualFileSystem, path: &str) -> Result<(), WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        vfs.remove_all(path).map_err(|_| WebServerError::Parser)
    }

    pub fn vfs_rename(
        vfs: &mut VirtualFileSystem,
        src_path: &str,
        dest_path: &str,
    ) -> Result<(), WebServerError> {
        let src_path = Path::new(src_path).map_err(|_| WebServerError::Path)?;
        let dest_path = Path::new(dest_path).map_err(|_| WebServerError::Path)?;

        vfs.rename(src_path, dest_path)
            .map_err(|_| WebServerError::Parser)
    }

    pub async fn vfs_upload_file<R: embedded_io_async::Read>(
        vfs: &mut VirtualFileSystem,
        path: &str,
        mut reader: RequestBodyReader<'_, R>,
    ) -> Result<(), WebServerError> {
        let path = Path::new(path).map_err(|_| WebServerError::Path)?;
        if let Ok(false) = vfs.exist(path) {
            vfs.create_file(path).map_err(|_| WebServerError::Parser)?;
        }
        let mut file = vfs.open(path).map_err(|_| WebServerError::Parser)?;
        let mut buffer = [0u8; HTTP_REQUEST_CHUNK_SIZE];

        loop {
            let bytes_written = reader
                .read(&mut buffer)
                .await
                .map_err(|_| WebServerError::Parser)?;

            if bytes_written == 0 {
                break;
            }

            file.write_all(&buffer[..bytes_written])
                .map_err(|_| WebServerError::Parser)?;
        }

        file.close().map_err(|_| WebServerError::Parser)
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
