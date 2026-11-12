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
mod configs;
mod drivers;
mod hardwares;
mod models;
mod services;
mod utils;
mod vfs;

// ---
use embassy_executor::Spawner;
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;

use crate::configs::{ESP_LITTLEFS_PARTITION_OFFSET, ESP_LITTLEFS_PARTITION_SIZE};
use crate::drivers::filesystems::anyfs::AnyFs;
use crate::drivers::filesystems::littlefs::{LittleFsFileSystem, LittleFsStorage};
use crate::hardwares::board::Board;
use crate::hardwares::radio::wifi::wifi_task;
use crate::models::filesystem::FileSystem;
use crate::services::debug::debug_service;
use crate::services::network::{init_network_stack, net_monitor_service, net_runner_service};
use crate::services::webserver::webserver_service;
use crate::vfs::VirtualFileSystem;

// ---
#[allow(clippy::large_stack_frames)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // --- Board ---
    let board = Board::init();

    // --- WiFi & Network ---
    let wifi_controller = esp_radio::wifi::WifiController::new(
        board.app_peripherals.wifi,
        esp_radio::wifi::ControllerConfig::default(),
    )
    .expect("Failed to create WifiController");
    let (network_stack, network_runner) = init_network_stack(esp_radio::wifi::Interface::station());

    // --- VFS --
    let flash = esp_storage::FlashStorage::new(board.app_peripherals.flash);
    let littlefs_storage = LittleFsStorage::<_, ESP_LITTLEFS_PARTITION_SIZE>::new(
        flash,
        ESP_LITTLEFS_PARTITION_OFFSET,
    );
    let mut root_filesystem = AnyFs::LittleFs(
        LittleFsFileSystem::new(littlefs_storage).expect("Failed to create LittleFs filesystem"),
    );

    if !root_filesystem
        .is_mountable()
        .await
        .expect("Failed to get mountable state of root_filesystem")
    {
        log::debug!("[DEBUG] File is not mountable, try format");
        root_filesystem.format().await.expect("format filesystem");
    }

    root_filesystem
        .mount()
        .await
        .expect("Failed to mount root_filesystem");

    let vfs = VirtualFileSystem::new(root_filesystem).await;

    // --- Spawn hardware Tasks ---
    spawner.spawn(wifi_task(wifi_controller).unwrap());

    // --- Spawn service Tasks ---
    spawner.spawn(net_runner_service(network_runner).unwrap());
    spawner.spawn(net_monitor_service(network_stack).unwrap());
    spawner.spawn(webserver_service(spawner, network_stack, vfs).unwrap());
    spawner.spawn(debug_service().unwrap());

    loop {
        Delay.delay_ms(3600 * 1000).await;
    }
}
