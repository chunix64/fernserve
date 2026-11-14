use anyhow::Result;
use log::warn;

use crate::configs::WIFI_SCAN_MAX_LENGTH;
use crate::hardwares::radio::wifi::{
    WIFI_SCAN_REPLY_CHANNEL, WIFI_SET_CONFIG_REPLY_CHANNEL, WifiCommandError, WifiScanResponse,
    WifiSetConfigResponse,
};
use crate::models::configs::{AccessPoint, Station, WifiConfig};
use crate::{
    hardwares::radio::wifi::{WIFI_COMMAND_CHANNEL, WifiCommand},
    models::wifi::AccessPointInfo,
};

pub struct WifiDriver;

impl WifiDriver {
    pub async fn scan_wifi() -> heapless::Vec<AccessPointInfo, WIFI_SCAN_MAX_LENGTH> {
        WIFI_COMMAND_CHANNEL.send(WifiCommand::Scan).await;

        match WIFI_SCAN_REPLY_CHANNEL.receive().await {
            Ok(WifiScanResponse::Scan(ap_list)) => {
                return ap_list
                    .into_iter()
                    .filter_map(|ap| AccessPointInfo::try_from(ap).ok())
                    .collect();
            }
            Err(WifiCommandError::WifiError(error)) => {
                warn!("[WIFI] Scan wifi failed: {:?}", error)
            }
        }

        heapless::Vec::new()
    }

    pub async fn set_station(station_config: WifiConfig<Station>) -> Result<()> {
        WIFI_COMMAND_CHANNEL
            .send(WifiCommand::SetStation(station_config))
            .await;

        match WIFI_SET_CONFIG_REPLY_CHANNEL.receive().await {
            Ok(WifiSetConfigResponse::Ok) => {}
            Err(error) => {
                warn!("[WIFI] Set Station failed: {:?}", error)
            }
        }

        Ok(())
    }

    pub async fn set_access_point(access_point_config: WifiConfig<AccessPoint>) -> Result<()> {
        WIFI_COMMAND_CHANNEL
            .send(WifiCommand::SetAccessPoint(access_point_config))
            .await;

        match WIFI_SET_CONFIG_REPLY_CHANNEL.receive().await {
            Ok(WifiSetConfigResponse::Ok) => {}
            Err(error) => {
                warn!("[WIFI] Set Access Point failed: {:?}", error)
            }
        }

        Ok(())
    }
}
