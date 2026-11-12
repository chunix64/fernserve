use core::net::Ipv4Addr;

use embassy_net::{Ipv4Cidr, StackResources};
use esp_hal::rng::Rng;
use static_cell::StaticCell;

use crate::configs::NET_STACK_RESOURCES_SIZE;

static NET_STACK_RESOURCES_STATION: StaticCell<StackResources<NET_STACK_RESOURCES_SIZE>> =
    StaticCell::new();
static NET_STACK_RESOURCES_ACCESS_POINT: StaticCell<StackResources<NET_STACK_RESOURCES_SIZE>> =
    StaticCell::new();

pub fn init_network_stack_station<'a>(
    station: esp_radio::wifi::Interface,
) -> (
    embassy_net::Stack<'a>,
    embassy_net::Runner<'a, esp_radio::wifi::Interface>,
) {
    let rng = Rng::new();
    let stack_resources_station = NET_STACK_RESOURCES_STATION
        .uninit()
        .write(StackResources::<NET_STACK_RESOURCES_SIZE>::new());

    let embassy_net_config = embassy_net::Config::dhcpv4(Default::default());
    let embassy_net_seed = (rng.random() as u64) << 32 | rng.random() as u64;
    embassy_net::new(
        station,
        embassy_net_config,
        stack_resources_station,
        embassy_net_seed,
    )
}

pub fn init_network_stack_access_point<'a>(
    access_point: esp_radio::wifi::Interface,
) -> (
    embassy_net::Stack<'a>,
    embassy_net::Runner<'a, esp_radio::wifi::Interface>,
) {
    let rng = Rng::new();
    let stack_resources_access_point = NET_STACK_RESOURCES_ACCESS_POINT
        .uninit()
        .write(StackResources::<NET_STACK_RESOURCES_SIZE>::new());

    let embassy_net_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Addr::new(192, 168, 4, 1), 24),
        gateway: Some(Ipv4Addr::new(192, 168, 4, 1)),
        dns_servers: [Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(8, 8, 4, 4)].into(),
    });

    let embassy_net_seed = (rng.random() as u64) << 32 | rng.random() as u64;
    embassy_net::new(
        access_point,
        embassy_net_config,
        stack_resources_access_point,
        embassy_net_seed,
    )
}

#[embassy_executor::task(pool_size = 2)]
pub async fn net_runner_service(
    mut runner: embassy_net::Runner<'static, esp_radio::wifi::Interface>,
) {
    runner.run().await;
}

#[embassy_executor::task(pool_size = 2)]
pub async fn net_monitor_service(network_stack: embassy_net::Stack<'static>, role: &'static str) {
    loop {
        network_stack.wait_config_up().await;
        if let Some(config) = network_stack.config_v4() {
            log::info!("[NET] {} IP: {}", role, config.address);
        }
        network_stack.wait_config_down().await;
        log::warn!("[NET] {} Link/config lost", role);
    }
}
