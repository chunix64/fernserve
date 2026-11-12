use alloc::string::String;

use anyhow::Result;

use super::WebServerParser;
use crate::drivers::radio::wifi::WifiDriver;

impl WebServerParser {
    pub async fn scan_wifi() -> Result<String> {
        Ok(serde_json::to_string(&WifiDriver::scan_wifi().await)?)
    }
}
