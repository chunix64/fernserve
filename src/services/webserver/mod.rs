use embassy_executor::Spawner;
use embassy_time::Duration;
use picoserve::routing::ServicePathRouter;
use picoserve::{AppBuilder, AppRouter, Timeouts, make_static};

use crate::configs::{HTTP_BUFFER_SIZE, WEB_SERVER_PORT, WEB_TASK_POOL_SIZE};
use crate::{services::webserver::handler::WebServerHandler, vfs::VirtualFileSystem};

mod chunks;
mod handler;
mod parser;

static WEBSERVER_CONFIG: picoserve::Config = picoserve::Config::new(Timeouts {
    start_read_request: Duration::from_secs(5),
    persistent_start_read_request: Duration::from_secs(5),
    read_request: Duration::from_secs(30),
    write: Duration::from_secs(30),
});

#[embassy_executor::task]
pub async fn webserver_service(
    spawner: Spawner,
    network_stack: embassy_net::Stack<'static>,
    vfs: VirtualFileSystem,
) {
    let app_props = AppProps::new(vfs);
    let app = make_static!(AppRouter<AppProps>, app_props.build_app());

    for task_id in 0..WEB_TASK_POOL_SIZE {
        spawner.spawn(web_task(task_id, network_stack, app).unwrap());
    }
}

// Inner logic
struct AppProps {
    handler: WebServerHandler,
}

impl AppProps {
    pub fn new(vfs: VirtualFileSystem) -> Self {
        Self {
            handler: WebServerHandler::new(vfs),
        }
    }
}

impl AppBuilder for AppProps {
    type PathRouter = ServicePathRouter<WebServerHandler>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::from_service(self.handler)
    }
}

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    task_id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static picoserve::AppRouter<AppProps>,
) {
    let port = WEB_SERVER_PORT;
    let mut tcp_rx_buffer: [u8; HTTP_BUFFER_SIZE / 2] = [0; HTTP_BUFFER_SIZE / 2];
    let mut tcp_tx_buffer: [u8; HTTP_BUFFER_SIZE / 2] = [0; HTTP_BUFFER_SIZE / 2];
    let mut http_buffer: [u8; HTTP_BUFFER_SIZE] = [0; HTTP_BUFFER_SIZE];

    picoserve::Server::new(app, &WEBSERVER_CONFIG, &mut http_buffer)
        .listen_and_serve(
            task_id,
            stack,
            port as u16,
            &mut tcp_rx_buffer,
            &mut tcp_tx_buffer,
        )
        .await
        .into_never()
}
