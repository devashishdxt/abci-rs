pub mod abci;
pub mod merkle;
pub mod types;

use std::io::Result;

use async_std::io::{Read, Write};

use self::abci::{Request, Response};

pub async fn decode<R: Read>(mut _reader: R) -> Result<Option<Request>> {
    unimplemented!()
}

pub async fn encode<W: Write>(_message: Response, mut _writer: W) -> Result<()> {
    unimplemented!()
}
