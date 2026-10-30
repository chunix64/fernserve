#![no_std]
#![no_main]
#![deny(clippy::mem_forget)]
#![deny(clippy::large_stack_frames)]

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    log::info!("[PANIC] {}", panic_info);
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

extern crate alloc;

// ---
mod app;
mod drivers;
mod hardwares;
mod models;
mod services;

use embassy_executor::Spawner;
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;

use crate::{app::app_task, hardwares::board::Board};

#[allow(clippy::large_stack_frames)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let _board = Board::init();

    spawner.spawn(app_task(spawner).unwrap());

    loop {
        Delay.delay_ms(3600 * 1000).await;
    }
}
