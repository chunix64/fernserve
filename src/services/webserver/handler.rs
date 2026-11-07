use alloc::borrow::Cow;
use heapless::String;
use nanofish::{
    BodyLength, HttpHeader, HttpMethod, HttpRequest, ResponseWriter, StatusCode, StreamingHandler,
    mime_types,
};

use crate::{
    services::webserver::{WebServerError, parser::WebServerParser},
    vfs::VirtualFileSystem,
};

// Defines
pub struct WebServerHandler {
    vfs: VirtualFileSystem,
    buffer: String<2048>,
}

use super::WEBSERVER_BUFFER_SIZE;

// Helper macros
macro_rules! helper_static_file {
    ($response:ident, $path:literal, $mime:expr $(, $header:expr)* $(,)?) => {{
        static BODY: &[u8] =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/webui/dist/", $path));

        $response
            .send_head(
                StatusCode::Ok,
                &[
                    HttpHeader::new("Access-Control-Allow-Headers", "*"),
                    HttpHeader::content_type($mime),
                    $($header,)*
                ],
                BodyLength::Known(BODY.len()),
            )
            .await?;

        for chunk in BODY.chunks(WEBSERVER_BUFFER_SIZE) {
            $response.write(chunk).await?;
        }

        Ok(())
    }};
}

// Impl
impl WebServerHandler {
    pub fn new(vfs: VirtualFileSystem) -> Self {
        Self {
            vfs,
            buffer: String::new(),
        }
    }
}

impl StreamingHandler for WebServerHandler {
    async fn handle<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        &mut self,
        request: &HttpRequest<'_>,
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        log::debug!("[DEBUG] http: {:?}", request);
        match request.method {
            HttpMethod::GET => self.get_handler(request, response).await,
            HttpMethod::POST => self.post_handler(request, response).await,
            _ => self.other_handler(response).await,
        }
    }
}

impl WebServerHandler {
    // Methods
    async fn get_handler<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        &mut self,
        request: &HttpRequest<'_>,
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        match request.route_path() {
            // static files
            // TODO: include_dir! macro
            "/" => helper_static_file!(
                response,
                "index.html.gz",
                mime_types::HTML,
                HttpHeader::new("Content-Encoding", "gzip")
            ),

            // APIs
            "/api/v1/status" => Self::status(response).await,
            "/api/v1/vfs/read-dir" => self.vfs_read_dir(request, response).await,
            "/api/v1/vfs/create-dir" => self.vfs_create_dir(request, response).await,
            "/api/v1/vfs/remove" => self.vfs_remove(request, response).await,
            "/api/v1/vfs/rename" => self.vfs_rename(request, response).await,

            // Not founds
            _ => Self::not_found(response).await,
        }
    }

    async fn post_handler<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        &mut self,
        request: &HttpRequest<'_>,
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        match request.path {
            "/api/v1/status" => Self::status(response).await,
            "/api/v1/vfs/read-dir" => self.vfs_read_dir(request, response).await,
            "/api/v1/vfs/create-dir" => self.vfs_create_dir(request, response).await,
            "/api/v1/vfs/remove" => self.vfs_remove(request, response).await,
            "/api/v1/vfs/rename" => self.vfs_rename(request, response).await,
            _ => Self::not_found(response).await,
        }
    }

    async fn other_handler<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        &mut self,
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        response
            .send_head(
                StatusCode::MethodNotAllowed,
                &[
                    HttpHeader::new("Access-Control-Allow-Headers", "*"),
                    HttpHeader::content_type(mime_types::TEXT),
                ],
                BodyLength::Known(0),
            )
            .await?;

        Ok(())
    }

    // Path specifics
    async fn status<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        Self::helper_ok_response(response).await
    }

    async fn not_found<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        Self::helper_json_response(response, r###"{"error": "not_found"}"###).await
    }

    async fn vfs_read_dir<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        &mut self,
        request: &HttpRequest<'_>,
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        let path: Cow<'_, str> = Self::helper_parse_json_key(request, "path")?;

        match WebServerParser::vfs_read_dir(&mut self.vfs, &path) {
            Ok(json) => Self::helper_json_response(response, &json).await,
            Err(error) => self.helper_error_response(response, error).await,
        }
    }

    async fn vfs_create_dir<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        &mut self,
        request: &HttpRequest<'_>,
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        let path: Cow<'_, str> = Self::helper_parse_json_key(request, "path")?;

        match WebServerParser::vfs_create_dir(&mut self.vfs, &path) {
            Ok(_) => Self::helper_ok_response(response).await,
            Err(error) => self.helper_error_response(response, error).await,
        }
    }

    async fn vfs_remove<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        &mut self,
        request: &HttpRequest<'_>,
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        let path: Cow<'_, str> = Self::helper_parse_json_key(request, "path")?;

        match WebServerParser::vfs_remove(&mut self.vfs, &path) {
            Ok(_) => Self::helper_ok_response(response).await,
            Err(error) => self.helper_error_response(response, error).await,
        }
    }

    async fn vfs_rename<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        &mut self,
        request: &HttpRequest<'_>,
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        let src_path: Cow<'_, str> = Self::helper_parse_json_key(request, "src_path")?;
        let dest_path: Cow<'_, str> = Self::helper_parse_json_key(request, "dest_path")?;

        match WebServerParser::vfs_rename(&mut self.vfs, &src_path, &dest_path) {
            Ok(_) => Self::helper_ok_response(response).await,
            Err(error) => self.helper_error_response(response, error).await,
        }
    }

    // Helpers
    fn helper_parse_json_key<'a>(
        request: &'a HttpRequest<'a>,
        key: &str,
    ) -> Result<Cow<'a, str>, nanofish::Error> {
        match request.method {
            HttpMethod::POST => {
                let body = request
                    .body_str()
                    .map_err(|_| nanofish::Error::InvalidUrl)?;
                Ok(Cow::Owned(
                    WebServerParser::parse_json_key(body, key)
                        .map_err(|_| nanofish::Error::InvalidResponse("invalid path body"))?,
                ))
            }
            _ => Ok(Cow::Borrowed(
                request.query(key).ok_or(nanofish::Error::InvalidUrl)?,
            )),
        }
    }

    async fn helper_ok_response<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
    ) -> Result<(), nanofish::Error> {
        Self::helper_json_response(response, r###"{"status": "ok"}"###).await
    }

    async fn helper_json_response<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
        json: &str,
    ) -> Result<(), nanofish::Error> {
        response
            .send_head(
                StatusCode::Ok,
                &[
                    HttpHeader::new("Access-Control-Allow-Headers", "*"),
                    HttpHeader::content_type(mime_types::JSON),
                ],
                BodyLength::Known(json.len()),
            )
            .await?;

        for chunk in json.as_bytes().chunks(WEBSERVER_BUFFER_SIZE) {
            response.write(chunk).await?;
        }

        Ok(())
    }

    async fn helper_error_response<S: embedded_io_async::Write, const HEAD_SIZE: usize>(
        &mut self,
        response: &mut ResponseWriter<'_, S, HEAD_SIZE>,
        error: WebServerError,
    ) -> Result<(), nanofish::Error> {
        self.buffer = String::try_from(WebServerParser::parse_error(error).as_str())
            .map_err(|_| nanofish::error::Error::BufferOverflow)?;

        response
            .send_head(
                StatusCode::NotFound,
                &[
                    HttpHeader::new("Access-Control-Allow-Headers", "*"),
                    HttpHeader::content_type(mime_types::TEXT),
                ],
                BodyLength::Known(self.buffer.len()),
            )
            .await?;

        for chunk in self.buffer.as_bytes().chunks(WEBSERVER_BUFFER_SIZE) {
            response.write(chunk).await?;
        }

        Ok(())
    }
}
