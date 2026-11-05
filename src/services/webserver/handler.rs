use alloc::borrow::Cow;
use heapless::{String, Vec};
use nanofish::{HttpHandler, HttpMethod, HttpRequest, HttpResponse, ResponseBody, StatusCode};

use crate::{
    services::webserver::{WebServerError, parser::WebServerParser},
    vfs::VirtualFileSystem,
};

pub struct WebServerHandler {
    vfs: VirtualFileSystem,
    buffer: String<2048>,
}

impl WebServerHandler {
    pub fn new(vfs: VirtualFileSystem) -> Self {
        Self {
            vfs,
            buffer: String::new(),
        }
    }
}

impl HttpHandler for WebServerHandler {
    async fn handle_request(
        &mut self,
        request: &nanofish::HttpRequest<'_>,
    ) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        log::debug!("[DEBUG] http: {:?}", request);
        match request.method {
            HttpMethod::GET => self.get_handler(request),
            HttpMethod::POST => self.post_handler(request),
            _ => Ok(HttpResponse {
                status_code: StatusCode::MethodNotAllowed,
                headers: Vec::new(),
                body: ResponseBody::Empty,
            }),
        }
    }
}

impl WebServerHandler {
    // Methods
    fn get_handler(
        &mut self,
        request: &HttpRequest,
    ) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        match request.route_path() {
            "/" => self.homepage(),
            "/api/v1/status" => Self::status(),
            "/api/v1/vfs/read_dir" => self.vfs_read_dir(request),
            "/api/vfs/create_dir" => self.vfs_create_dir(request),
            "/api/vfs/remove" => self.vfs_remove(request),
            "/api/vfs/rename" => self.vfs_rename(request),
            _ => Self::not_found(),
        }
    }

    fn post_handler(
        &mut self,
        request: &HttpRequest,
    ) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        match request.path {
            "/api/status" => Self::status(),
            "/api/vfs/read_dir" => self.vfs_read_dir(request),
            "/api/vfs/create_dir" => self.vfs_create_dir(request),
            "/api/vfs/remove" => self.vfs_remove(request),
            "/api/vfs/rename" => self.vfs_rename(request),
            _ => Self::not_found(),
        }
    }

    // Path specifics
    fn homepage(&mut self) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        self.helper_text_response("<h1>Fernserve</h1>")
    }

    fn status() -> Result<nanofish::HttpResponse<'static>, nanofish::Error> {
        Self::helper_ok_response()
    }

    fn not_found() -> Result<nanofish::HttpResponse<'static>, nanofish::Error> {
        Ok(HttpResponse {
            status_code: StatusCode::NotFound,
            headers: Vec::new(),
            body: ResponseBody::Text(r###"{"error": "not_found"}"###),
        })
    }

    fn vfs_read_dir(
        &mut self,
        request: &HttpRequest,
    ) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        let path: Cow<'_, str> = Self::helper_parse_json_key(request, "path")?;

        match WebServerParser::vfs_read_dir(&mut self.vfs, &path) {
            Ok(json) => self.helper_text_response(&json),
            Err(error) => self.helper_error_response(error),
        }
    }

    fn vfs_create_dir(
        &mut self,
        request: &HttpRequest,
    ) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        let path: Cow<'_, str> = Self::helper_parse_json_key(request, "path")?;

        match WebServerParser::vfs_create_dir(&mut self.vfs, &path) {
            Ok(_) => Self::helper_ok_response(),
            Err(error) => self.helper_error_response(error),
        }
    }

    fn vfs_remove(
        &mut self,
        request: &HttpRequest,
    ) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        let path: Cow<'_, str> = Self::helper_parse_json_key(request, "path")?;

        match WebServerParser::vfs_remove(&mut self.vfs, &path) {
            Ok(_) => Self::helper_ok_response(),
            Err(error) => self.helper_error_response(error),
        }
    }

    fn vfs_rename(
        &mut self,
        request: &HttpRequest,
    ) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        let src_path: Cow<'_, str> = Self::helper_parse_json_key(request, "src_path")?;
        let dest_path: Cow<'_, str> = Self::helper_parse_json_key(request, "dest_path")?;

        match WebServerParser::vfs_rename(&mut self.vfs, &src_path, &dest_path) {
            Ok(_) => Self::helper_ok_response(),
            Err(error) => self.helper_error_response(error),
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

    fn helper_ok_response() -> Result<nanofish::HttpResponse<'static>, nanofish::Error> {
        Ok(HttpResponse {
            status_code: StatusCode::Ok,
            headers: Vec::new(),
            body: ResponseBody::Text(r###"{"status": "ok"}"###),
        })
    }

    fn helper_text_response(
        &mut self,
        json: &str,
    ) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        self.buffer = String::try_from(json).map_err(|_| nanofish::error::Error::BufferOverflow)?;

        Ok(HttpResponse {
            status_code: StatusCode::Ok,
            headers: Vec::new(),
            body: ResponseBody::Text(self.buffer.as_str()),
        })
    }

    fn helper_error_response(
        &mut self,
        error: WebServerError,
    ) -> Result<nanofish::HttpResponse<'_>, nanofish::Error> {
        self.buffer = String::try_from(WebServerParser::parse_error(error).as_str())
            .map_err(|_| nanofish::error::Error::BufferOverflow)?;

        Ok(HttpResponse {
            status_code: StatusCode::ServiceUnavailable,
            headers: Vec::new(),
            body: ResponseBody::Text(self.buffer.as_str()),
        })
    }
}
