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
extern crate tinyrlibc;

// ---
mod app;
mod drivers;
mod hardwares;
mod models;
mod services;
mod utils;
mod vfs;

use embassy_executor::Spawner;
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;

use crate::models::filesystem::FileSystem;
use crate::models::path::Path;
use crate::vfs::VirtualFileSystem;
use crate::{app::app_task, hardwares::board::Board};

use crate::models::filesystem::File;
use embedded_io::*;

const ESP_LITTLEFS_PARTITION_OFFSET: usize = 0x310000;
const ESP_LITTLEFS_PARTITION_SIZE: usize = 0xF0000;

#[allow(clippy::large_stack_frames)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let board = Board::init();

    spawner.spawn(app_task(spawner).unwrap());

    loop {
        Delay.delay_ms(3600 * 1000).await;
    }
}
