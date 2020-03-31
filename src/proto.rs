pub mod abci;
pub mod merkle;
pub mod types;

use std::io::{Error, ErrorKind, Result};

#[cfg(feature = "use-async-std")]
use async_std::{
    io::{Read, Write},
    prelude::*,
};
#[cfg(feature = "use-tokio")]
use tokio::io::{AsyncRead as Read, AsyncReadExt, AsyncWrite as Write, AsyncWriteExt};

use integer_encoding::{VarIntAsyncReader, VarIntAsyncWriter};
use protobuf::{parse_from_bytes, Message};

use self::abci::{Request, Response};

/// Decodes a `Request` from stream
pub async fn decode<R: Read + Unpin + Send>(mut reader: R) -> Result<Option<Request>> {
    let length: i64 = reader.read_varint_async().await?;

    if length == 0 {
        return Ok(None);
    }

    let mut bytes = vec![0; length as usize];
    reader.take(length as u64).read(&mut bytes).await?;

    parse_from_bytes(&bytes)
        .map(Some)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
}

/// Encodes a `Response` to stream
pub async fn encode<W: Write + Unpin + Send>(message: Response, mut writer: W) -> Result<()> {
    writer
        .write_varint_async(i64::from(message.compute_size()))
        .await?;

    let bytes = message
        .write_to_bytes()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    writer.write_all(&bytes).await
}
