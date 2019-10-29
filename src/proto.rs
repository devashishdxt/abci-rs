pub mod abci;
pub mod merkle;
pub mod types;

#[cfg(feature = "sync")]
pub use self::encode_decode::{decode, encode};

#[cfg(feature = "sync")]
mod encode_decode {
    use std::io::{Error, ErrorKind, Read, Result, Write};

    use integer_encoding::{VarIntReader, VarIntWriter};
    use protobuf::{parse_from_reader, Message};

    use super::abci::{Request, Response};

    pub fn decode<R: Read>(mut reader: R) -> Result<Option<Request>> {
        let length = reader.read_varint::<i64>()?;

        if length == 0 {
            return Ok(None);
        }

        parse_from_reader::<Request>(&mut reader.take(length as u64))
            .map(Some)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    pub fn encode<W: Write>(message: Response, mut writer: W) -> Result<()> {
        writer.write_varint(i64::from(message.compute_size()))?;
        message
            .write_to_writer(&mut writer)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
