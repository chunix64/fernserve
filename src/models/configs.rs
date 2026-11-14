use core::marker::PhantomData;

use esp_hal::peripherals::{FLASH, WIFI};
use heapless::String;

use crate::configs::{
    DEFAULT_WIFI_AP_PASSWORD, DEFAULT_WIFI_AP_SSID, DEFAULT_WIFI_PASSWORD, DEFAULT_WIFI_SSID,
};

pub struct AppPeripherals {
    pub wifi: WIFI<'static>,
    pub flash: FLASH<'static>,
}

pub struct WifiConfig<Mode> {
    pub ssid: String<32>,
    pub password: String<64>,
    _type: PhantomData<Mode>,
}

pub struct Station;
pub struct AccessPoint;

impl<Mode> WifiConfig<Mode> {
    pub fn try_new(ssid: &str, password: &str) -> Option<Self> {
        Some(Self {
            ssid: String::try_from(ssid).ok()?,
            password: String::try_from(password).ok()?,
            _type: PhantomData,
        })
    }
}

impl Default for WifiConfig<Station> {
    fn default() -> Self {
        Self {
            ssid: String::try_from(DEFAULT_WIFI_SSID).unwrap_or_default(),
            password: String::try_from(DEFAULT_WIFI_PASSWORD).unwrap_or_default(),
            _type: PhantomData,
        }
    }
}

impl Default for WifiConfig<AccessPoint> {
    fn default() -> Self {
        Self {
            ssid: String::try_from(DEFAULT_WIFI_AP_SSID).unwrap_or_default(),
            password: String::try_from(DEFAULT_WIFI_AP_PASSWORD).unwrap_or_default(),
            _type: PhantomData,
        }
    }
}
