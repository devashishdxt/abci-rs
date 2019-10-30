pub mod abci;
pub mod merkle;
pub mod types;

#[cfg(feature = "async")]
pub use self::codec_async::AbciCodec;
pub use self::encode_decode_sync::{decode_sync, encode_sync};

mod encode_decode_sync {
    use std::io::{Error, ErrorKind, Read, Result, Write};

    use integer_encoding::{VarIntReader, VarIntWriter};
    use protobuf::{parse_from_reader, Message};

    use super::abci::{Request, Response};

    pub fn decode_sync<R: Read>(mut reader: R) -> Result<Option<Request>> {
        let length = reader.read_varint::<i64>()?;

        if length == 0 {
            return Ok(None);
        }

        parse_from_reader::<Request>(&mut reader.take(length as u64))
            .map(Some)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    pub fn encode_sync<W: Write>(message: Response, mut writer: W) -> Result<()> {
        writer.write_varint(i64::from(message.compute_size()))?;
        message
            .write_to_writer(&mut writer)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

#[cfg(feature = "async")]
mod codec_async {
    use std::io::{Error, ErrorKind, Read};

    use bytes::{Buf, BufMut, BytesMut, IntoBuf};
    use integer_encoding::{VarIntReader, VarIntWriter};
    use protobuf::{parse_from_reader, Message};
    use tokio::codec::{Decoder, Encoder};

    use super::abci::{Request, Response};

    #[derive(Default)]
    pub struct AbciCodec;

    impl Encoder for AbciCodec {
        type Item = Response;
        type Error = Error;

        fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
            let mut writer = dst.writer();
            writer.write_varint(i64::from(item.compute_size()))?;
            item.write_to_writer(&mut writer)
                .map_err(|e| Error::new(ErrorKind::Other, e))
        }
    }

    impl Decoder for AbciCodec {
        type Item = Request;
        type Error = Error;

        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            let mut reader = (&*src).into_buf().reader();

            let length = reader.read_varint::<i64>()?;

            if length == 0 {
                return Ok(None);
            }

            parse_from_reader::<Request>(&mut reader.take(length as u64))
                .map(Some)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))
        }
    }
}
