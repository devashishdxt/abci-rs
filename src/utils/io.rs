use std::io::Result;

#[cfg(feature = "use-async-std")]
use async_std::{
    io::{Read, Write},
    prelude::*,
};
use bytes::BytesMut;
use prost::Message;
#[cfg(feature = "use-smol")]
use smol::io::{AsyncRead as Read, AsyncReadExt, AsyncWrite as Write, AsyncWriteExt};
#[cfg(feature = "use-tokio")]
use tokio::io::{AsyncRead as Read, AsyncReadExt, AsyncWrite as Write, AsyncWriteExt};

use crate::{
    stream_split::StreamSplit,
    types::{decode, encode},
};

const DEFAULT_BUFFER_SIZE: usize = 4096;

pub struct StreamReader<S>
where
    S: Read + Unpin,
{
    stream: S,
    read_buf: [u8; DEFAULT_BUFFER_SIZE],
    buf: BytesMut,
}

impl<S> StreamReader<S>
where
    S: Read + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            read_buf: [0; DEFAULT_BUFFER_SIZE],
            buf: BytesMut::new(),
        }
    }

    pub async fn read<M: Message + Default>(&mut self) -> Result<Option<M>> {
        let current_value = decode::<M, _>(&mut self.buf)?;

        if current_value.is_some() {
            return Ok(current_value);
        }

        self.fill_buf().await?;

        let value = decode::<M, _>(&mut self.buf)?;

        Ok(value)
    }

    async fn fill_buf(&mut self) -> Result<usize> {
        let bytes_read = self.stream.read(&mut self.read_buf).await?;

        if bytes_read == 0 {
            return Ok(0);
        }

        self.buf.extend_from_slice(&self.read_buf[0..bytes_read]);
        Ok(bytes_read)
    }
}

pub struct StreamWriter<S>
where
    S: Write + Unpin,
{
    stream: S,
}

impl<S> StreamWriter<S>
where
    S: Write + Unpin,
{
    fn new(stream: S) -> Self {
        Self { stream }
    }

    pub async fn write<M: Message>(&mut self, message: M) -> Result<()> {
        let mut buf = BytesMut::new();
        encode(message, &mut buf)?;

        self.stream.write(&buf).await.map(|_| ())
    }
}

pub fn get_stream_pair<S: StreamSplit>(
    stream: S,
) -> (StreamReader<S::Reader>, StreamWriter<S::Writer>) {
    let (reader, writer) = stream.split_stream();
    (StreamReader::new(reader), StreamWriter::new(writer))
}
