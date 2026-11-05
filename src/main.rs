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
mod drivers;
mod hardwares;
mod models;
mod services;
mod utils;
mod vfs;

use embassy_executor::Spawner;
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use log::warn;

use crate::drivers::anyfs::AnyFs;
use crate::drivers::littlefs::{LittleFsFileSystem, LittleFsStorage};
use crate::hardwares::board::Board;
use crate::hardwares::radio::wifi::wifi_task;
use crate::models::configs::WifiConfig;
use crate::models::filesystem::FileSystem;
use crate::services::debug::debug_service;
use crate::services::network::{init_network_stack, net_monitor_service, net_runner_service};
use crate::services::webserver::webserver_service;
use crate::vfs::VirtualFileSystem;

pub const ESP_LITTLEFS_PARTITION_OFFSET: usize = 0x310000;
pub const ESP_LITTLEFS_PARTITION_SIZE: usize = 0xF0000;

#[allow(clippy::large_stack_frames)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let board = Board::init();

    // --- WiFi & Network ---
    let wifi_controller = esp_radio::wifi::WifiController::new(
        board.app_peripherals.wifi,
        esp_radio::wifi::ControllerConfig::default(),
    );

    let (network_stack, network_runner) = init_network_stack(esp_radio::wifi::Interface::station());

    // --- Configs ---
    let wifi_config = WifiConfig {
        ssid: heapless::String::try_from("YOUR_SSID").unwrap(),
        password: heapless::String::try_from("YOUR_SSID_PASSWORD").unwrap(),
    };

    // --- VFS --
    let flash = esp_storage::FlashStorage::new(board.app_peripherals.flash);
    let littlefs_storage = LittleFsStorage::<_, ESP_LITTLEFS_PARTITION_SIZE>::new(
        flash,
        ESP_LITTLEFS_PARTITION_OFFSET,
    );
    let mut root_filesystem =
        AnyFs::LittleFs(LittleFsFileSystem::new(littlefs_storage).expect("test"));

    if !root_filesystem.is_mountable().unwrap_or(false) {
        log::debug!("[DEBUG] File is not mountable, try format");
        root_filesystem.format().expect("format filesystem");
    }

    root_filesystem.mount().expect("mount");

    let vfs = VirtualFileSystem::new(root_filesystem).expect("test");

    // --- Spawn hardware Tasks ---
    match wifi_controller {
        Ok(wifi_controller) => {
            spawner.spawn(wifi_task(wifi_controller, wifi_config).unwrap());
        }
        Err(error) => {
            warn!("[MAIN] Failed to start wifi task: {}", error);
        }
    }

    // --- Spawn service Tasks ---
    spawner.spawn(net_runner_service(network_runner).unwrap());
    spawner.spawn(net_monitor_service(network_stack).unwrap());
    spawner.spawn(webserver_service(network_stack, vfs).unwrap());
    spawner.spawn(debug_service().unwrap());

    loop {
        Delay.delay_ms(3600 * 1000).await;
    }
}
