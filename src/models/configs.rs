use esp_hal::peripherals::WIFI;

pub struct AppPeripherals {
    pub wifi: WIFI<'static>,
}
