use alloc::borrow::ToOwned;

use picoserve::ResponseSent;
use picoserve::response::chunked::ChunkedResponse;

use crate::services::webserver::chunks::ChunkReader;
use crate::services::webserver::parser::WebServerParser;

use super::WebServerHandler;

impl WebServerHandler {
    pub(super) async fn vfs_exists<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = match Self::helper_parse_key(&mut request, "path").await {
            Ok(path) => WebServerParser::vfs_exists(&mut *self.vfs.lock().await, &path).await,
            Err(error) => Err(error),
        };

        match result {
            Ok(json) => Self::helper_json_response(request, response_writer, &json).await,
            Err(error) => {
                Self::helper_webserver_error_response(request, response_writer, error).await
            }
        }
    }

    pub(super) async fn vfs_available_space<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let path = Self::helper_parse_key(&mut request, "path")
            .await
            .unwrap_or("/".to_owned());

        let result = WebServerParser::vfs_available_space(&mut *self.vfs.lock().await, &path).await;

        match result {
            Ok(json) => Self::helper_json_response(request, response_writer, &json).await,
            Err(error) => {
                Self::helper_webserver_error_response(request, response_writer, error).await
            }
        }
    }

    pub(super) async fn vfs_total_space<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let path = Self::helper_parse_key(&mut request, "path")
            .await
            .unwrap_or("/".to_owned());

        let result = WebServerParser::vfs_total_space(&mut *self.vfs.lock().await, &path).await;

        match result {
            Ok(json) => Self::helper_json_response(request, response_writer, &json).await,
            Err(error) => {
                Self::helper_webserver_error_response(request, response_writer, error).await
            }
        }
    }

    pub(super) async fn vfs_read_dir<
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

    pub(super) async fn vfs_create_dir<
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

    pub(super) async fn vfs_remove<
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

    pub(super) async fn vfs_remove_all<
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

    pub(super) async fn vfs_rename<
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

    pub(super) async fn vfs_upload_file_raw<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = match Self::helper_parse_param_key(&mut request, "path").await {
            Ok(path) => {
                WebServerParser::vfs_upload_file_raw(
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

    pub(super) async fn vfs_upload_file<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let result = async {
            let path = Self::helper_parse_param_key(&mut request, "path").await?;
            let chunk_index: usize = Self::helper_parse_param_key(&mut request, "chunk_index")
                .await?
                .parse()?;
            let total_chunks: usize = Self::helper_parse_param_key(&mut request, "total_chunks")
                .await?
                .parse()?;

            WebServerParser::vfs_upload_file_chunk(
                &mut *self.vfs.lock().await,
                &path,
                chunk_index,
                total_chunks,
                request.body_connection.body().reader(),
            )
            .await
        }
        .await;

        match result {
            Ok(json) => Self::helper_json_response(request, response_writer, &json).await,
            Err(error) => {
                Self::helper_webserver_error_response(request, response_writer, error).await
            }
        }
    }

    pub(super) async fn vfs_download_file<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        mut request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let path = match Self::helper_parse_key(&mut request, "path").await {
            Ok(path) => path,
            Err(error) => {
                return Self::helper_webserver_error_response(request, response_writer, error)
                    .await;
            }
        };

        let path = match crate::models::path::Path::new(&path) {
            Ok(path) => path,
            Err(error) => {
                return Self::helper_webserver_error_response(
                    request,
                    response_writer,
                    error.into(),
                )
                .await;
            }
        };

        let mut vfs = self.vfs.lock().await;

        if let Ok(true) = vfs.exists(path).await {
            let file = match vfs.open(path).await {
                Ok(file) => file,
                Err(error) => {
                    return Self::helper_webserver_error_response(
                        request,
                        response_writer,
                        error.into(),
                    )
                    .await;
                }
            };

            let connection = request.body_connection.finalize().await?;

            let chunked_response =
                ChunkedResponse::new(ChunkReader::new(file, "application/octet-stream"))
                    .into_response()
                    .with_header("Content-Disposition", "inline");

            if let Ok(metadata) = vfs.metadata(path).await {
                return response_writer
                    .write_response(
                        connection,
                        chunked_response.with_header("Content-Length", metadata.len),
                    )
                    .await;
            } else {
                return response_writer
                    .write_response(connection, chunked_response)
                    .await;
            }
        }

        Self::helper_webserver_error_response(
            request,
            response_writer,
            crate::models::errors::VirtualFileSystemError::InvalidPath.into(),
        )
        .await
    }
}
