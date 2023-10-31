use crate::types::ReadHalfBuf;
use tokio::io::{AsyncRead, AsyncWrite, BufReader, WriteHalf};

pub enum ConnectionType {
    HTTP,
    HTTPS,
}

/// Represents a HTTP/HTTPS connection with separated read and write streams.  
pub struct Connection<T>
where
    T: AsyncRead + AsyncWrite,
{
    pub read_buffer: ReadHalfBuf<T>,
    pub writer: WriteHalf<T>,
    pub conn_type: ConnectionType,
}

impl<T> Connection<T>
where
    T: AsyncRead + AsyncWrite,
{
    pub fn new(stream: T, conn_type: ConnectionType) -> Connection<T> {
        let (read_half, write_half) = tokio::io::split(stream);
        let read_buffer = BufReader::new(read_half);

        Connection {
            read_buffer: read_buffer,
            writer: write_half,
            conn_type: conn_type,
        }
    }
}
