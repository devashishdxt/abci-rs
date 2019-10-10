pub mod abci;
pub mod merkle;
pub mod types;

use std::io::{Error, ErrorKind, Read, Result, Write};

use integer_encoding::{VarIntReader, VarIntWriter};
use protobuf::{parse_from_reader, Message};

use self::abci::{Request, Response};

pub fn decode<R: Read>(mut reader: R) -> Result<Request> {
    reader.read_varint::<i64>()?;
    parse_from_reader::<Request>(&mut reader).map_err(|e| Error::new(ErrorKind::InvalidData, e))
}

pub fn encode<W: Write>(message: Response, mut writer: W) -> Result<()> {
    writer.write_varint(i64::from(message.compute_size()))?;
    message
        .write_to_writer(&mut writer)
        .map_err(|e| Error::new(ErrorKind::Other, e))
}
