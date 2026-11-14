use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use picoserve::ResponseSent;
use picoserve::response::Response;
use picoserve::routing::PathRouterService;

use crate::vfs::VirtualFileSystem;

mod helpers;
mod vfs;
mod wifi;

// Defines
pub struct WebServerHandler {
    vfs: Mutex<NoopRawMutex, VirtualFileSystem>,
}

// Helpers macros
macro_rules! helper_static_file {
    ($request:ident,
     $response_writer:ident,
     $path:literal,
     $mime:literal
     $(, $header_key:expr => $header_value:expr)* $(,)?
     ) => {{
        let connection = $request.body_connection.finalize().await?;

        static BODY: &[u8] =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/webui/dist/", $path));

        $response_writer
            .write_response(
                connection,
                Response::ok(BODY)
                    .with_content_type($mime)
                    .with_header("Access-Control-Allow-Origin", "*")
                    $(.with_header($header_key, $header_value))*
            )
            .await
    }};
}

// Impl
impl WebServerHandler {
    pub fn new(vfs: VirtualFileSystem) -> Self {
        Self {
            vfs: Mutex::new(vfs),
        }
    }
}

impl PathRouterService for WebServerHandler {
    async fn call_path_router_service<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        _state: &(),
        _current_path_parameters: (),
        _path: picoserve::request::Path<'_>,
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        match request.parts.method() {
            "GET" => self.get_handler(request, response_writer).await,
            "POST" => self.post_handler(request, response_writer).await,
            _ => self.other_handler(request, response_writer).await,
        }
    }
}

// Method specific
impl WebServerHandler {
    async fn get_handler<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        match request.parts.path().encoded() {
            // APIs
            "/api/v1/status" => Self::status(request, response_writer).await,

            // VFS
            "/api/v1/vfs/exists" => self.vfs_exists(request, response_writer).await,
            "/api/v1/vfs/read-dir" => self.vfs_read_dir(request, response_writer).await,
            "/api/v1/vfs/create-dir" => self.vfs_create_dir(request, response_writer).await,
            "/api/v1/vfs/remove" => self.vfs_remove(request, response_writer).await,
            "/api/v1/vfs/remove-all" => self.vfs_remove_all(request, response_writer).await,
            "/api/v1/vfs/rename" => self.vfs_rename(request, response_writer).await,
            "/api/v1/vfs/download-file" => self.vfs_download_file(request, response_writer).await,
            "/api/v1/vfs/total-space" => self.vfs_total_space(request, response_writer).await,
            "/api/v1/vfs/available-space" => {
                self.vfs_available_space(request, response_writer).await
            }

            // Wifi
            "/api/v1/wifi/scan" => Self::wifi_scan(request, response_writer).await,
            "/api/v1/wifi/set-station" => Self::set_station(request, response_writer).await,
            "/api/v1/wifi/set-access-point" => {
                Self::set_access_point(request, response_writer).await
            }

            // Reduce server load
            "/favicon.ico" => Self::not_found(request, response_writer).await,

            // Not founds
            _ => self.get_default_routing(request, response_writer).await,
        }
    }

    async fn post_handler<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        match request.parts.path().encoded() {
            // APIs
            "/api/v1/status" => Self::status(request, response_writer).await,

            // VFS
            "/api/v1/vfs/exists" => self.vfs_exists(request, response_writer).await,
            "/api/v1/vfs/read-dir" => self.vfs_read_dir(request, response_writer).await,
            "/api/v1/vfs/create-dir" => self.vfs_create_dir(request, response_writer).await,
            "/api/v1/vfs/remove" => self.vfs_remove(request, response_writer).await,
            "/api/v1/vfs/remove-all" => self.vfs_remove_all(request, response_writer).await,
            "/api/v1/vfs/rename" => self.vfs_rename(request, response_writer).await,
            "/api/v1/vfs/upload-file-raw" => {
                self.vfs_upload_file_raw(request, response_writer).await
            }
            "/api/v1/vfs/upload-file" => self.vfs_upload_file(request, response_writer).await,
            "/api/v1/vfs/download-file" => self.vfs_download_file(request, response_writer).await,
            "/api/v1/vfs/total-space" => self.vfs_total_space(request, response_writer).await,
            "/api/v1/vfs/available-space" => {
                self.vfs_available_space(request, response_writer).await
            }

            // Wifi
            "/api/v1/wifi/scan" => Self::wifi_scan(request, response_writer).await,

            // Not founds
            _ => Self::not_found(request, response_writer).await,
        }
    }

    async fn other_handler<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_error_response(request, response_writer, "Method Not Allowed", 405).await
    }

    pub async fn status<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_ok_response(request, response_writer).await
    }

    pub async fn not_found<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_error_response(request, response_writer, "Not Found", 404).await
    }

    pub async fn get_default_routing<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        if let Ok(path) = crate::models::path::Path::new(request.parts.path().encoded()) {
            let mut vfs = self.vfs.lock().await;
            if let Ok(metadata) = vfs.metadata(path).await
                && !metadata.is_dir
            {
                return self.vfs_download_file(request, response_writer).await;
            }
        };

        helper_static_file!(
            request,
            response_writer,
            "index.html.gz",
            "text/html",
            "Content-Encoding" => "gzip",
        )
    }
}
