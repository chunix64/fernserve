use nanofish::SmallEmbassyHttpServer;
use serde::Serialize;

use crate::{services::webserver::handler::WebServerHandler, vfs::VirtualFileSystem};

pub mod handler;
mod parser;

#[embassy_executor::task]
pub async fn webserver_service(network_stack: embassy_net::Stack<'static>, vfs: VirtualFileSystem) {
    let mut server = SmallEmbassyHttpServer::new(80);
    let handler = WebServerHandler::new(vfs);

    server.serve(network_stack, handler).await;
}

#[derive(Serialize)]
pub enum WebServerError {
    Parser,
    Path,
}
