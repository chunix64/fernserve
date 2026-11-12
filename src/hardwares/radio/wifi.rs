use anyhow::Result;
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config, Password, Ssid, WifiController,
    ap::{AccessPointConfig, AccessPointInfo},
    scan::ScanConfig,
    sta::StationConfig,
};
use log::{error, info, warn};

use crate::{
    configs::{WIFI_RETRY_INTERVAL, WIFI_SCAN_MAX_LENGTH},
    models::configs::{AccessPoint, Station, WifiConfig},
};

#[embassy_executor::task]
pub async fn wifi_task(wifi_controller: WifiController<'static>) {
    let wifi_runner = WifiRunner::new(
        wifi_controller,
        WifiConfig::<Station>::default(),
        WifiConfig::<AccessPoint>::default(),
    );
    wifi_runner.run().await;
}

// ---
pub type WifiScanResult = heapless::Vec<AccessPointInfo, WIFI_SCAN_MAX_LENGTH>;

pub enum WifiCommand {
    SetStation(WifiConfig<Station>),
    SetAp(WifiConfig<AccessPoint>),
    Scan,
}

pub enum WifiCommandReply {
    Scan(WifiScanResult),
    Error(esp_radio::wifi::WifiError),
}

type WifiCommandChannel = Channel<CriticalSectionRawMutex, WifiCommand, 4>;
type WifiCommandReplyChannel = Channel<CriticalSectionRawMutex, WifiCommandReply, 4>;

pub static WIFI_COMMAND_CHANNEL: WifiCommandChannel = Channel::new();
pub static WIFI_COMMAND_REPLY_CHANNEL: WifiCommandReplyChannel = Channel::new();

// ---
// TODO: store esp_radio Configs instead WifiConfig
struct WifiRunner {
    wifi_controller: WifiController<'static>,
    station_config: WifiConfig<Station>,
    ap_config: WifiConfig<AccessPoint>,
}

impl WifiRunner {
    fn new(
        wifi_controller: WifiController<'static>,
        station_config: WifiConfig<Station>,
        ap_config: WifiConfig<AccessPoint>,
    ) -> Self {
        Self {
            wifi_controller,
            station_config,
            ap_config,
        }
    }

    fn get_station_config(ssid: &str, password: &str) -> Result<StationConfig> {
        let ssid = Ssid::try_from(ssid)?;
        let password = Password::try_from(password)?;
        let authentication = AuthenticationMethodConfig::Wpa2Personal(password);

        Ok(StationConfig::default()
            .with_ssid(ssid)
            .with_authentication(authentication))
    }

    fn get_ap_config(ssid: &str, password: &str) -> Result<AccessPointConfig> {
        let ssid = Ssid::try_from(ssid)?;
        let password = Password::try_from(password)?;
        let authentication = AuthenticationMethodConfig::Wpa2Personal(password);

        Ok(AccessPointConfig::default()
            .with_ssid(ssid)
            .with_authentication(authentication))
    }

    fn apply(&mut self) {
        let station_config = match Self::get_station_config(
            &self.station_config.ssid,
            &self.station_config.password,
        ) {
            Ok(station_config) => station_config,
            Err(error) => {
                error!("[WIFI] Station config is invalid: {:?}", error);
                StationConfig::default() // TODO: use the old config instead placeholder configs
            }
        };

        let ap_config = match Self::get_ap_config(&self.ap_config.ssid, &self.ap_config.password) {
            Ok(ap_config) => ap_config,
            Err(error) => {
                error!("[WIFI] Ap config is invalid: {:?}", error);
                AccessPointConfig::default()
            }
        };

        let wifi_config = Config::AccessPointStation(station_config, ap_config);

        if self.wifi_controller.set_config(&wifi_config).is_err() {
            warn!("[WIFI] Failed to set WiFi config!");
        };
    }

    async fn reconnect_and_apply(&mut self) {
        if self.wifi_controller.is_connected() {
            let _ = self.wifi_controller.disconnect_async().await;
        }

        self.apply();
    }

    async fn run(mut self) {
        self.reconnect_and_apply().await;

        if let Err(error) = self
            .wifi_controller
            .set_power_saving(esp_radio::wifi::PowerSaveMode::Minimum)
        {
            warn!("[WIFI] Can not set power saving mode: {:?}", error);
        }

        loop {
            let command = if self.wifi_controller.is_connected() {
                match select(
                    self.wifi_controller.wait_for_disconnect_async(),
                    WIFI_COMMAND_CHANNEL.receive(),
                )
                .await
                {
                    Either::First(_) => {
                        Delay.delay_ms(WIFI_RETRY_INTERVAL as u32).await;
                        None
                    }
                    Either::Second(command) => Some(command),
                }
            } else {
                match select(
                    self.wifi_controller.connect_async(),
                    WIFI_COMMAND_CHANNEL.receive(),
                )
                .await
                {
                    Either::First(Ok(_)) => {
                        info!("[WIFI] WiFi connected to {}", self.station_config.ssid);
                        None
                    }
                    Either::First(Err(error)) => {
                        warn!(
                            "[WIFI] WiFi cannot connect to {}, retry! Error: {:?}",
                            self.station_config.ssid, error
                        );
                        Delay.delay_ms(WIFI_RETRY_INTERVAL as u32).await;
                        None
                    }
                    Either::Second(command) => Some(command),
                }
            };

            if let Some(command) = command {
                match command {
                    WifiCommand::SetStation(station_config) => {
                        self.station_config = station_config;
                        self.reconnect_and_apply().await
                    }
                    WifiCommand::SetAp(ap_config) => {
                        self.ap_config = ap_config;
                        self.reconnect_and_apply().await
                    }
                    WifiCommand::Scan => {
                        match self
                            .wifi_controller
                            .scan_async(&ScanConfig::default().with_max(WIFI_SCAN_MAX_LENGTH))
                            .await
                        {
                            Ok(ap_list) => {
                                let mut result = WifiScanResult::new();

                                for ap in ap_list {
                                    let _ = result.push(ap);
                                }

                                WIFI_COMMAND_REPLY_CHANNEL
                                    .send(WifiCommandReply::Scan(result))
                                    .await;
                            }
                            Err(error) => {
                                WIFI_COMMAND_REPLY_CHANNEL
                                    .send(WifiCommandReply::Error(error))
                                    .await;
                            }
                        };
                    }
                }
            }
        }
    }
}
