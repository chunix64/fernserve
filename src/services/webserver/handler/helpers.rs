use alloc::string::String;
use alloc::string::ToString;

use anyhow::{Result, anyhow};
use picoserve::ResponseSent;
use picoserve::response::{Response, StatusCode};

use super::WebServerHandler;
use crate::services::webserver::parser::WebServerParser;

impl WebServerHandler {
    pub(super) async fn helper_parse_key<R: embedded_io_async::Read>(
        request: &mut picoserve::request::Request<'_, R>,
        key: &str,
    ) -> Result<String> {
        match request.parts.method() {
            "POST" => Self::helper_parse_json_key(request, key).await,
            _ => Self::helper_parse_param_key(request, key).await,
        }
    }

    pub(super) async fn helper_parse_json_key<R: embedded_io_async::Read>(
        request: &mut picoserve::request::Request<'_, R>,
        key: &str,
    ) -> Result<String> {
        let body = request.body_connection.body().read_all().await?;

        WebServerParser::parse_json_key(&String::from_utf8_lossy(body), key)
    }

    pub(super) async fn helper_parse_param_key<R: embedded_io_async::Read>(
        request: &mut picoserve::request::Request<'_, R>,
        key: &str,
    ) -> Result<String> {
        let query = request
            .parts
            .query()
            .ok_or(anyhow!("Request parse query failed"))?
            .try_into_string::<2048>()?;

        WebServerParser::parse_param_key(&query, key)?.ok_or(anyhow!("Parse param key failed"))
    }

    pub(super) async fn helper_result_response<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
        T,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
        result: Result<T>,
    ) -> Result<ResponseSent, W::Error> {
        match result {
            Ok(_) => Self::helper_ok_response(request, response_writer).await,
            Err(error) => {
                Self::helper_webserver_error_response(request, response_writer, error).await
            }
        }
    }

    pub(super) async fn helper_ok_response<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_json_response(request, response_writer, r###"{"status": 200}"###).await
    }

    pub(super) async fn helper_webserver_error_response<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
        error: anyhow::Error,
    ) -> Result<ResponseSent, W::Error> {
        Self::helper_error_response(request, response_writer, &error.to_string(), 503).await
    }

    pub(super) async fn helper_error_response<
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

    pub(super) async fn helper_json_response<
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

    pub(super) async fn helper_text_response<
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
