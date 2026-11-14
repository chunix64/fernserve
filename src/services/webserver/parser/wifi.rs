use alloc::string::String;

use anyhow::{Context, Result};

use super::WebServerParser;
use crate::{
    drivers::radio::wifi::WifiDriver,
    models::configs::{AccessPoint, Station, WifiConfig},
};

impl WebServerParser {
    pub async fn scan_wifi() -> Result<String> {
        Ok(serde_json::to_string(&WifiDriver::scan_wifi().await)?)
    }

    pub async fn set_station(ssid: &str, password: &str) -> Result<()> {
        let station_config = WifiConfig::<Station>::try_new(ssid, password)
            .context("Get WifiConfig for Station failed")?;

        WifiDriver::set_station(station_config).await
    }

    pub async fn set_access_point(ssid: &str, password: &str) -> Result<()> {
        let access_point_config = WifiConfig::<AccessPoint>::try_new(ssid, password)
            .context("Get WifiConfig for Access Point failed")?;

        WifiDriver::set_access_point(access_point_config).await
    }
}
