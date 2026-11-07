use nanofish::EmbassyHttpServer;
use serde::Serialize;

use crate::{services::webserver::handler::WebServerHandler, vfs::VirtualFileSystem};

pub mod handler;
mod parser;

const WEBSERVER_BUFFER_SIZE: usize = 8192;

#[embassy_executor::task]
pub async fn webserver_service(network_stack: embassy_net::Stack<'static>, vfs: VirtualFileSystem) {
    let mut server = EmbassyHttpServer::<
        WEBSERVER_BUFFER_SIZE,
        WEBSERVER_BUFFER_SIZE,
        WEBSERVER_BUFFER_SIZE,
        WEBSERVER_BUFFER_SIZE,
    >::new(80);
    let handler = WebServerHandler::new(vfs);

    server.serve_streaming(network_stack, handler).await;
}

#[derive(Serialize)]
pub enum WebServerError {
    Parser,
    Path,
}
