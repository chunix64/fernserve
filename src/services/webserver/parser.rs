use alloc::string::{String, ToString};
use log::warn;
use serde_json::Value;

use crate::{models::path::Path, services::webserver::WebServerError, vfs::VirtualFileSystem};

pub struct WebServerParser;

impl WebServerParser {
    // path specifics
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

    // helpers
    pub fn parse_json_key(body: &str, key: &str) -> Result<String, WebServerError> {
        let value: Value = serde_json::from_str(body).map_err(|_| WebServerError::Parser)?;

        Ok(value[key]
            .as_str()
            .ok_or(WebServerError::Parser)?
            .to_string())
    }

    pub fn parse_error(error: WebServerError) -> String {
        match serde_json::to_string(&error) {
            Ok(text) => text,
            Err(error) => {
                warn!("[WEBSERVER] parse error failed: {}", error);
                String::new()
            }
        }
    }
}
