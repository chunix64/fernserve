use alloc::string::ToString;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;

use alloc::string::String;
use picoserve::ResponseSent;
use picoserve::response::{Response, StatusCode};
use picoserve::routing::PathRouterService;

use crate::services::webserver::WebServerError;
use crate::services::webserver::parser::WebServerParser;
use crate::vfs::VirtualFileSystem;

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
            "/api/v1/vfs/exist" => self.vfs_exist(request, response_writer).await,
            "/api/v1/vfs/read-dir" => self.vfs_read_dir(request, response_writer).await,
            "/api/v1/vfs/create-dir" => self.vfs_create_dir(request, response_writer).await,
            "/api/v1/vfs/remove" => self.vfs_remove(request, response_writer).await,
            "/api/v1/vfs/remove-all" => self.vfs_remove_all(request, response_writer).await,
            "/api/v1/vfs/rename" => self.vfs_rename(request, response_writer).await,

            // Not founds
            _ => {
                helper_static_file!(
                    request,
                    response_writer,
                    "index.html.gz",
                    "text/html",
                    "Content-Encoding" => "gzip",
                )
            }
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
            "/api/v1/vfs/exist" => self.vfs_exist(request, response_writer).await,
            "/api/v1/vfs/read-dir" => self.vfs_read_dir(request, response_writer).await,
            "/api/v1/vfs/create-dir" => self.vfs_create_dir(request, response_writer).await,
            "/api/v1/vfs/remove" => self.vfs_remove(request, response_writer).await,
            "/api/v1/vfs/remove-all" => self.vfs_remove_all(request, response_writer).await,
            "/api/v1/vfs/rename" => self.vfs_rename(request, response_writer).await,
            "/api/v1/vfs/upload-file" => self.vfs_upload_file(request, response_writer).await,

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
}

// Path specific
impl WebServerHandler {
    async fn status<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_ok_response(request, response_writer).await
    }

    async fn not_found<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_json_response(request, response_writer, r###"{"error": "not_found"}"###).await
    }

    async fn vfs_exist<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = match Self::helper_parse_key(&mut request, "path").await {
            Ok(path) => WebServerParser::vfs_exist(&mut *self.vfs.lock().await, &path).await,
            Err(error) => Err(error),
        };

        match result {
            Ok(json) => Self::helper_json_response(request, response_writer, &json).await,
            Err(error) => {
                Self::helper_webserver_error_response(request, response_writer, error).await
            }
        }
    }

    async fn vfs_read_dir<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = match Self::helper_parse_key(&mut request, "path").await {
            Ok(path) => WebServerParser::vfs_read_dir(&mut *self.vfs.lock().await, &path).await,
            Err(error) => Err(error),
        };

        match result {
            Ok(json) => Self::helper_json_response(request, response_writer, &json).await,
            Err(error) => {
                Self::helper_webserver_error_response(request, response_writer, error).await
            }
        }
    }

    async fn vfs_create_dir<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = match Self::helper_parse_key(&mut request, "path").await {
            Ok(path) => WebServerParser::vfs_create_dir(&mut *self.vfs.lock().await, &path).await,
            Err(error) => Err(error),
        };

        Self::helper_result_response(request, response_writer, result).await
    }

    async fn vfs_remove<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = match Self::helper_parse_key(&mut request, "path").await {
            Ok(path) => WebServerParser::vfs_remove(&mut *self.vfs.lock().await, &path).await,
            Err(error) => Err(error),
        };

        Self::helper_result_response(request, response_writer, result).await
    }

    async fn vfs_remove_all<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = match Self::helper_parse_key(&mut request, "path").await {
            Ok(path) => WebServerParser::vfs_remove_all(&mut *self.vfs.lock().await, &path).await,
            Err(error) => Err(error),
        };

        Self::helper_result_response(request, response_writer, result).await
    }

    async fn vfs_rename<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = async {
            let src_path = Self::helper_parse_key(&mut request, "src_path").await?;
            let dest_path = Self::helper_parse_key(&mut request, "dest_path").await?;
            WebServerParser::vfs_rename(&mut *self.vfs.lock().await, &src_path, &dest_path).await
        }
        .await;

        Self::helper_result_response(request, response_writer, result).await
    }

    async fn vfs_upload_file<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = match Self::helper_parse_param_key(&mut request, "path").await {
            Ok(path) => {
                WebServerParser::vfs_upload_file(
                    &mut *self.vfs.lock().await,
                    &path,
                    request.body_connection.body().reader(),
                )
                .await
            }
            Err(error) => Err(error),
        };

        Self::helper_result_response(request, response_writer, result).await
    }
}

// Helpers
impl WebServerHandler {
    async fn helper_parse_key<R: embedded_io_async::Read>(
        request: &mut picoserve::request::Request<'_, R>,
        key: &str,
    ) -> Result<String, WebServerError> {
        match request.parts.method() {
            "POST" => Self::helper_parse_json_key(request, key).await,
            _ => Self::helper_parse_param_key(request, key).await,
        }
    }

    async fn helper_parse_json_key<R: embedded_io_async::Read>(
        request: &mut picoserve::request::Request<'_, R>,
        key: &str,
    ) -> Result<String, WebServerError> {
        let body = request
            .body_connection
            .body()
            .read_all()
            .await
            .map_err(|_| WebServerError::ParseJsonKey)?;
        WebServerParser::parse_json_key(&String::from_utf8_lossy(body), key)
            .map_err(|_| WebServerError::ParseJsonKey)
    }

    async fn helper_parse_param_key<R: embedded_io_async::Read>(
        request: &mut picoserve::request::Request<'_, R>,
        key: &str,
    ) -> Result<String, WebServerError> {
        WebServerParser::parse_query_key(
            request
                .parts
                .query()
                .ok_or(WebServerError::ParseParamKey)?
                .try_into_string::<2048>()
                .map_err(|_| WebServerError::ParseParamKey)?
                .as_str(),
            key,
        )
        .map_err(|_| WebServerError::ParseParamKey)?
        .ok_or(WebServerError::ParseParamKey)
    }

    async fn helper_result_response<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
        T,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
        result: Result<T, WebServerError>,
    ) -> Result<ResponseSent, W::Error> {
        match result {
            Ok(_) => Self::helper_ok_response(request, response_writer).await,
            Err(error) => {
                Self::helper_webserver_error_response(request, response_writer, error).await
            }
        }
    }

    async fn helper_ok_response<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_json_response(request, response_writer, r###"{"status": "ok"}"###).await
    }

    async fn helper_webserver_error_response<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
        error: WebServerError,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_error_response(request, response_writer, &error.to_string(), 503).await
    }

    async fn helper_error_response<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
        error: &str,
        status_code: u16,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_text_response(
            request,
            response_writer,
            &WebServerParser::parse_error_json(status_code, error),
            "application/json",
            StatusCode::new(status_code),
        )
        .await
    }

    async fn helper_json_response<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
        json: &str,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_text_response(
            request,
            response_writer,
            json,
            "application/json",
            StatusCode::OK,
        )
        .await
    }

    async fn helper_text_response<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
        text: &str,
        mime: &'static str,
        status: StatusCode,
    ) -> Result<ResponseSent, W::Error> {
        let connection = request.body_connection.finalize().await?;

        response_writer
            .write_response(
                connection,
                Response::new(status, text)
                    .with_content_type(mime)
                    .with_header("Access-Control-Allow-Origin", "*"),
            )
            .await
    }
}
