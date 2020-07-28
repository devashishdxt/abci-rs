#![allow(missing_docs)]
//! Types used in ABCI
pub(crate) mod abci {
    #[allow(clippy::large_enum_variant)]
    pub mod types {
        include!("proto/tendermint.abci.types.rs");
    }
}

mod crypto {
    pub mod merkle {
        include!("proto/tendermint.crypto.merkle.rs");
    }
}

mod libs {
    pub mod kv {
        include!("proto/tendermint.libs.kv.rs");
    }
}

pub use prost_types::{Duration, Timestamp};

pub use self::abci::types::{
    BlockId, BlockParams, ConsensusParams, Event, Evidence, EvidenceParams, Header, LastCommitInfo,
    PartSetHeader, PubKey, RequestBeginBlock, RequestCheckTx, RequestCommit, RequestDeliverTx,
    RequestEcho, RequestEndBlock, RequestFlush, RequestInfo, RequestInitChain, RequestQuery,
    RequestSetOption, ResponseBeginBlock, ResponseCheckTx, ResponseCommit, ResponseDeliverTx,
    ResponseEcho, ResponseEndBlock, ResponseFlush, ResponseInfo, ResponseInitChain, ResponseQuery,
    ResponseSetOption, Validator, ValidatorParams, ValidatorUpdate, Version, VoteInfo,
};
pub use self::crypto::merkle::*;
pub use self::libs::kv::*;

use std::{
    convert::TryFrom,
    io::{Error, ErrorKind, Result},
};

#[cfg(feature = "use-async-std")]
use async_std::{
    io::{Read, Write},
    prelude::*,
};
#[cfg(feature = "use-tokio")]
use tokio::io::{AsyncRead as Read, AsyncReadExt, AsyncWrite as Write, AsyncWriteExt};

use integer_encoding::{VarIntAsyncReader, VarIntAsyncWriter};
use prost::Message;

/// Decodes a `Request` from stream
pub(crate) async fn decode<M: Message + Default, R: Read + Unpin + Send>(
    mut reader: R,
) -> Result<Option<M>> {
    let length: i64 = reader.read_varint_async().await?;

    if length == 0 {
        return Ok(None);
    }

    let mut bytes = vec![0; length as usize];
    reader.take(length as u64).read(&mut bytes).await?;

    <M>::decode(bytes.as_slice())
        .map(Some)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
}

/// Encodes a `Response` to stream
pub(crate) async fn encode<M: Message, W: Write + Unpin + Send>(
    message: M,
    mut writer: W,
) -> Result<()> {
    writer
        .write_varint_async(
            i64::try_from(message.encoded_len()).expect("Cannot convert from `i64` to `usize`"),
        )
        .await?;

    let mut bytes = vec![];

    message
        .encode(&mut bytes)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    writer.write_all(&bytes).await
}
