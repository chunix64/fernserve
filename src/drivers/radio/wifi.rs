use log::warn;

use crate::configs::WIFI_SCAN_MAX_LENGTH;
use crate::{
    hardwares::radio::wifi::{
        WIFI_COMMAND_CHANNEL, WIFI_COMMAND_REPLY_CHANNEL, WifiCommand, WifiCommandReply,
    },
    models::wifi::AccessPointInfo,
};

pub struct WifiDriver;

impl WifiDriver {
    pub async fn scan_wifi() -> heapless::Vec<AccessPointInfo, WIFI_SCAN_MAX_LENGTH> {
        WIFI_COMMAND_CHANNEL.send(WifiCommand::Scan).await;

        match WIFI_COMMAND_REPLY_CHANNEL.receive().await {
            WifiCommandReply::Scan(ap_list) => {
                return ap_list
                    .into_iter()
                    .filter_map(|ap| AccessPointInfo::try_from(ap).ok())
                    .collect();
            }
            WifiCommandReply::Error(error) => warn!("[WIFI] Scan wifi failed: {:?}", error),
        }

        heapless::Vec::new()
    }
}
