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
    use std::io::{Error, ErrorKind};

    use bytes::{BufMut, BytesMut};
    use integer_encoding::{VarInt, VarIntWriter};
    use protobuf::{parse_from_bytes, Message};
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
            if src.is_empty() {
                return Ok(None);
            }

            let (length, consumed) = i64::decode_var(&src[..]);

            if length as usize + consumed > src.len() {
                return Ok(None);
            }

            src.split_to(consumed);
            let request = parse_from_bytes(&src[..length as usize])?;
            src.split_to(length as usize);

            Ok(Some(request))
        }
    }
}
