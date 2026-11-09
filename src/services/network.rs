use crate::configs::NET_STACK_RESOURCES_SIZE;
use embassy_net::StackResources;
use esp_hal::rng::Rng;
use static_cell::StaticCell;

static NET_STACK_RESOURCES: StaticCell<StackResources<NET_STACK_RESOURCES_SIZE>> =
    StaticCell::new();

pub fn init_network_stack<'a>(
    station: esp_radio::wifi::Interface,
) -> (
    embassy_net::Stack<'a>,
    embassy_net::Runner<'a, esp_radio::wifi::Interface>,
) {
    let rng = Rng::new();
    let stack_resources = NET_STACK_RESOURCES
        .uninit()
        .write(StackResources::<NET_STACK_RESOURCES_SIZE>::new());

    let embassy_net_config = embassy_net::Config::dhcpv4(Default::default());
    let embassy_net_seed = (rng.random() as u64) << 32 | rng.random() as u64;
    embassy_net::new(
        station,
        embassy_net_config,
        stack_resources,
        embassy_net_seed,
    )
}

#[embassy_executor::task]
pub async fn net_runner_service(
    mut runner: embassy_net::Runner<'static, esp_radio::wifi::Interface>,
) {
    runner.run().await;
}

#[embassy_executor::task]
pub async fn net_monitor_service(network_stack: embassy_net::Stack<'static>) {
    network_stack.wait_config_up().await;
    if let Some(config) = network_stack.config_v4() {
        log::info!("[NET] IP: {}", config.address);
    }
}
