use embedded_io_async::Read;
use picoserve::response::chunked::{ChunkWriter, Chunks, ChunksWritten};

use crate::configs::HTTP_BUFFER_SIZE;

pub struct ChunkReader<R> {
    pub reader: R,
    pub content_type: &'static str,
}

impl<R> ChunkReader<R> {
    pub fn new(reader: R, content_type: &'static str) -> Self {
        Self {
            reader,
            content_type,
        }
    }
}

impl<R: Read> Chunks for ChunkReader<R> {
    fn content_type(&self) -> &'static str {
        self.content_type
    }

    async fn write_chunks<W: picoserve::io::Write>(
        mut self,
        mut chunk_writer: ChunkWriter<W>,
    ) -> Result<ChunksWritten, W::Error> {
        let mut buffer = [0u8; HTTP_BUFFER_SIZE];
        loop {
            match self.reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(read_bytes) => {
                    chunk_writer.write_chunk(&buffer[..read_bytes]).await?;
                }
                Err(error) => {
                    log::warn!("[CHUNK READER] failed to read chunks, {:?}", error);
                    break;
                }
            }
        }

        chunk_writer.finalize().await
    }
}
