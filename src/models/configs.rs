use esp_hal::peripherals::{FLASH, WIFI};

pub struct AppPeripherals {
    pub wifi: WIFI<'static>,
    pub flash: FLASH<'static>,
}

pub struct WifiConfig {
    pub ssid: heapless::String<32>,
    pub password: heapless::String<32>,
}
