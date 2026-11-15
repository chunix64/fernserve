use defmt::debug;
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;

use crate::configs::DEBUG_SERVICE_INTERVAL;

#[embassy_executor::task]
pub async fn debug_service() {
    loop {
        let free = esp_alloc::HEAP.free();
        let used = esp_alloc::HEAP.used();
        let total = free + used;

        let percentage_x100 = if total > 0 {
            (used as u64 * 10_000) / (total as u64)
        } else {
            0
        };

        debug!(
            "[DEBUG] Memory free: {}B, used: {}B, total: {}B ({}.{:02}%)",
            free,
            used,
            free + used,
            percentage_x100 / 100,
            percentage_x100 % 100
        );

        Delay.delay_ms(DEBUG_SERVICE_INTERVAL as u32).await;
    }
}
