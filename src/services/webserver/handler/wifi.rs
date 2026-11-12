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
}
