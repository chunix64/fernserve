use core::net::Ipv4Addr;

use leasehund::{DhcpConfigBuilder, DhcpServer};

use crate::configs::{DHCP_MAX_CLIENT, DHCP_MAX_DNS_SERVER, DHCP_MAX_POOLS, DHCP_MAX_STATIC};

#[embassy_executor::task]
pub async fn dhcp_server_service(ap_network_stack: embassy_net::Stack<'static>) {
    let config = DhcpConfigBuilder::<DHCP_MAX_DNS_SERVER>::new()
        .server_ip(Ipv4Addr::new(192, 168, 4, 1))
        .subnet_mask(Ipv4Addr::new(255, 255, 255, 0))
        .router(Ipv4Addr::new(192, 168, 4, 1))
        .add_dns_server(Ipv4Addr::new(8, 8, 8, 8))
        .add_dns_server(Ipv4Addr::new(8, 8, 4, 4))
        .ip_pool(
            Ipv4Addr::new(192, 168, 4, 100),
            Ipv4Addr::new(192, 168, 4, 200),
        )
        .lease_time(86400) // 1 day
        .build();

    let mut server: DhcpServer<
        DHCP_MAX_CLIENT,
        DHCP_MAX_DNS_SERVER,
        DHCP_MAX_STATIC,
        DHCP_MAX_POOLS,
    > = DhcpServer::with_config(config);

    log::error!(
        "[DHCP] Infallible server crashed: {:?}",
        server.run(ap_network_stack).await
    );
}
