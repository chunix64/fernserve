use esp_hal::peripherals::{FLASH, WIFI};

pub struct AppPeripherals {
    pub wifi: WIFI<'static>,
    pub flash: FLASH<'static>,
}
