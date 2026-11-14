use picoserve::ResponseSent;

use crate::services::webserver::parser::WebServerParser;

use super::WebServerHandler;

impl WebServerHandler {
    pub(super) async fn wifi_scan<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        match WebServerParser::scan_wifi().await {
            Ok(json) => Self::helper_json_response(request, response_writer, &json).await,
            Err(error) => {
                Self::helper_webserver_error_response(request, response_writer, error).await
            }
        }
    }

    pub(super) async fn set_station<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = async {
            let ssid = Self::helper_parse_key(&mut request, "ssid").await?;
            let password = Self::helper_parse_key(&mut request, "password").await?;

            WebServerParser::set_station(&ssid, &password).await
        }
        .await;

        Self::helper_result_response(request, response_writer, result).await
    }

    pub(super) async fn set_access_point<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = async {
            let ssid = Self::helper_parse_key(&mut request, "ssid").await?;
            let password = Self::helper_parse_key(&mut request, "password").await?;

            WebServerParser::set_access_point(&ssid, &password).await
        }
        .await;

        Self::helper_result_response(request, response_writer, result).await
    }
}
