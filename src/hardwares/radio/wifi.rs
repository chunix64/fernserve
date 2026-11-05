use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config, Password, PowerSaveMode, Ssid, WifiController,
    sta::StationConfig,
};
use log::{error, info, warn};

use crate::models::configs::WifiConfig;

#[embassy_executor::task]
pub async fn wifi_task(wifi_controller: WifiController<'static>, wifi_config: WifiConfig) {
    wifi_handle_connection(wifi_controller, wifi_config).await;
}

async fn wifi_handle_connection(
    mut wifi_controller: WifiController<'static>,
    wifi_config: WifiConfig,
) {
    let interval = 2000;

    let ssid = match Ssid::try_from(wifi_config.ssid.as_str()) {
        Ok(ssid) => ssid,
        Err(error) => {
            error!("[WIFI] SSID is invalid: {}", error);
            return;
        }
    };

    let password = match Password::try_from(wifi_config.password.as_str()) {
        Ok(password) => password,
        Err(error) => {
            error!("[WIFI] password is invalid: {}", error);
            return;
        }
    };

    let authentication = AuthenticationMethodConfig::Wpa2Personal(password);

    let client_config = Config::Station(
        StationConfig::default()
            .with_ssid(ssid)
            .with_authentication(authentication),
    );

    if wifi_controller.set_config(&client_config).is_err() {
        warn!("[WIFI] Failed to set WiFi config!");
        return;
    };

    if let Err(error) = wifi_controller.set_power_saving(PowerSaveMode::Minimum) {
        warn!("[WIFI] Can not set power saving mode: {:?}", error);
    }

    loop {
        if wifi_controller.is_connected() {
            let _ = wifi_controller.wait_for_disconnect_async().await;
            Delay.delay_ms(interval).await;
        } else {
            match wifi_controller.connect_async().await {
                Ok(_) => {
                    info!("[WIFI] WiFi connected to {}", wifi_config.ssid);
                }
                Err(error) => {
                    warn!(
                        "[WIFI] WiFi cannot connect to {}, retry! Error: {:?}",
                        wifi_config.ssid, error
                    );
                    Delay.delay_ms(interval).await;
                }
            }
        }
    }
}
