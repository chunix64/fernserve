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
use log::{info, warn};

use crate::{
    configs::{WIFI_RETRY_INTERVAL, WIFI_SCAN_MAX_LENGTH},
    models::configs::{AccessPoint, Station, WifiConfig},
};

#[embassy_executor::task]
pub async fn wifi_task(wifi_controller: WifiController<'static>) {
    let mut wifi_runner = WifiRunner::new(
        wifi_controller,
        StationConfig::default(),
        AccessPointConfig::default(),
    );

    let default_wifi_station_config = WifiConfig::<Station>::default();
    let default_wifi_ap_config = WifiConfig::<AccessPoint>::default();

    let _ = wifi_runner.set_station_config(
        &default_wifi_station_config.ssid,
        &default_wifi_station_config.password,
    );

    let _ = wifi_runner.set_ap_config(
        &default_wifi_ap_config.ssid,
        &default_wifi_ap_config.password,
    );

    wifi_runner.run().await;
}

// ---
pub type WifiScanResult = heapless::Vec<AccessPointInfo, WIFI_SCAN_MAX_LENGTH>;

pub enum WifiCommand {
    SetStation(WifiConfig<Station>),
    SetAccessPoint(WifiConfig<AccessPoint>),
    Scan,
}

pub enum WifiScanResponse {
    Scan(WifiScanResult),
}

pub enum WifiSetConfigResponse {
    Ok,
}

#[derive(Debug)]
pub enum WifiCommandError {
    WifiError(esp_radio::wifi::WifiError),
}

type WifiCommandChannel = Channel<CriticalSectionRawMutex, WifiCommand, 4>;
type WifiScanReplyChannel =
    Channel<CriticalSectionRawMutex, Result<WifiScanResponse, WifiCommandError>, 4>;
type WifiSetConfigReplyChannel =
    Channel<CriticalSectionRawMutex, Result<WifiSetConfigResponse, WifiCommandError>, 4>;

pub static WIFI_COMMAND_CHANNEL: WifiCommandChannel = Channel::new();
pub static WIFI_SCAN_REPLY_CHANNEL: WifiScanReplyChannel = Channel::new();
pub static WIFI_SET_CONFIG_REPLY_CHANNEL: WifiSetConfigReplyChannel = Channel::new();

// ---
// TODO: store esp_radio Configs instead WifiConfig
struct WifiRunner {
    wifi_controller: WifiController<'static>,
    station_config: StationConfig,
    ap_config: AccessPointConfig,
}

impl WifiRunner {
    fn new(
        wifi_controller: WifiController<'static>,
        station_config: StationConfig,
        ap_config: AccessPointConfig,
    ) -> Self {
        Self {
            wifi_controller,
            station_config,
            ap_config,
        }
    }

    fn parse_credentials(
        role: &str,
        ssid: &str,
        password: &str,
        need_password: bool,
    ) -> Result<(Ssid, Option<Password>), esp_radio::wifi::WifiError> {
        let ssid = Ssid::try_from(ssid).inspect_err(|error| {
            log::warn!(
                "[WIFI] Get {} ssid failed, keep the old config: {:?}",
                role,
                error
            )
        })?;

        if !need_password {
            return Ok((ssid, None));
        }

        let password = Password::try_from(password).inspect_err(|error| {
            log::warn!(
                "[WIFI] Get {} password failed, keep the old config: {:?}",
                role,
                error
            )
        })?;

        Ok((ssid, Some(password)))
    }

    fn set_station_config(
        &mut self,
        ssid: &str,
        password: &str,
    ) -> Result<(), esp_radio::wifi::WifiError> {
        let need_password = true;
        let result = Self::parse_credentials("Station", ssid, password, need_password)?;
        let (ssid, _) = result;
        let mut authentication = AuthenticationMethodConfig::Open;

        if need_password {
            let (_, Some(password)) = result else {
                return Err(esp_radio::wifi::WifiError::Other);
            };

            authentication = AuthenticationMethodConfig::Wpa2Personal(password);
        }

        self.station_config = StationConfig::default()
            .with_ssid(ssid)
            .with_authentication(authentication);

        Ok(())
    }

    fn set_ap_config(
        &mut self,
        ssid: &str,
        password: &str,
    ) -> Result<(), esp_radio::wifi::WifiError> {
        let need_password = true;
        let result = Self::parse_credentials("Access Point", ssid, password, need_password)?;
        let (ssid, _) = result;
        let mut authentication = AuthenticationMethodConfig::Open;

        if need_password {
            let (_, Some(password)) = result else {
                return Err(esp_radio::wifi::WifiError::Other);
            };

            authentication = AuthenticationMethodConfig::Wpa2Personal(password);
        }

        self.ap_config = AccessPointConfig::default()
            .with_ssid(ssid)
            .with_authentication(authentication);

        Ok(())
    }

    fn apply(&mut self) {
        let wifi_config =
            Config::AccessPointStation(self.station_config.clone(), self.ap_config.clone());

        if self.wifi_controller.set_config(&wifi_config).is_err() {
            warn!("[WIFI] Failed to set WiFi config!");
        };
    }

    async fn run(mut self) {
        self.apply();

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
                        info!(
                            "[WIFI] WiFi connected to {}",
                            self.station_config.ssid().as_str()
                        );
                        None
                    }
                    Either::First(Err(error)) => {
                        warn!(
                            "[WIFI] WiFi cannot connect to {}, retry! Error: {:?}",
                            self.station_config.ssid().as_str(),
                            error
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
                        match self
                            .set_station_config(&station_config.ssid, &station_config.password)
                        {
                            Ok(_) => {
                                WIFI_SET_CONFIG_REPLY_CHANNEL
                                    .send(Ok(WifiSetConfigResponse::Ok))
                                    .await
                            }
                            Err(error) => {
                                WIFI_SET_CONFIG_REPLY_CHANNEL
                                    .send(Err(WifiCommandError::WifiError(error)))
                                    .await
                            }
                        };
                        self.apply()
                    }
                    WifiCommand::SetAccessPoint(ap_config) => {
                        match self.set_ap_config(&ap_config.ssid, &ap_config.password) {
                            Ok(_) => {
                                WIFI_SET_CONFIG_REPLY_CHANNEL
                                    .send(Ok(WifiSetConfigResponse::Ok))
                                    .await
                            }
                            Err(error) => {
                                WIFI_SET_CONFIG_REPLY_CHANNEL
                                    .send(Err(WifiCommandError::WifiError(error)))
                                    .await
                            }
                        };
                        self.apply()
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

                                WIFI_SCAN_REPLY_CHANNEL
                                    .send(Ok(WifiScanResponse::Scan(result)))
                                    .await;
                            }
                            Err(error) => {
                                WIFI_SCAN_REPLY_CHANNEL
                                    .send(Err(WifiCommandError::WifiError(error)))
                                    .await;
                            }
                        };
                    }
                }
            }
        }
    }
}
