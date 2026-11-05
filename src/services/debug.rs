use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use log::debug;

#[embassy_executor::task]
pub async fn debug_service() {
    loop {
        let free = esp_alloc::HEAP.free();
        let used = esp_alloc::HEAP.used();

        debug!(
            "[DEBUG] Memory free: {}B, used: {}B, total: {}B ({:.2}%)",
            free,
            used,
            free + used,
            used as f32 / (free + used) as f32 * 100.0
        );

        Delay.delay_ms(5000).await;
    }
}
