//! Types used in ABCI
pub use prost_types::{Duration, Timestamp};
pub use tendermint_proto::{
    abci::{
        response_apply_snapshot_chunk::Result as ApplySnapshotChunkResult,
        response_offer_snapshot::Result as OfferSnapshotResult, BlockParams, CheckTxType,
        ConsensusParams, Event, EventAttribute, Evidence, LastCommitInfo,
        RequestApplySnapshotChunk, RequestBeginBlock, RequestCheckTx, RequestCommit,
        RequestDeliverTx, RequestEcho, RequestEndBlock, RequestFlush, RequestInfo,
        RequestInitChain, RequestListSnapshots, RequestLoadSnapshotChunk, RequestOfferSnapshot,
        RequestQuery, RequestSetOption, ResponseApplySnapshotChunk, ResponseBeginBlock,
        ResponseCheckTx, ResponseCommit, ResponseDeliverTx, ResponseEcho, ResponseEndBlock,
        ResponseFlush, ResponseInfo, ResponseInitChain, ResponseListSnapshots,
        ResponseLoadSnapshotChunk, ResponseOfferSnapshot, ResponseQuery, ResponseSetOption,
        Snapshot, Validator, ValidatorUpdate, VoteInfo,
    },
    crypto::{public_key::Sum, ProofOp, ProofOps, PublicKey},
    types::{BlockId, EvidenceParams, Header, PartSetHeader, ValidatorParams, VersionParams},
    version::Consensus,
};

use std::{
    convert::TryFrom,
    io::{Error, ErrorKind, Result},
};

use bytes::{Buf, BufMut};
use integer_encoding::VarInt;
use prost::Message;

/// Returns decoded message and number of bytes read from buffer
pub(crate) fn decode<M, B>(buf: &mut B) -> Result<Option<M>>
where
    M: Message + Default,
    B: Buf,
{
    if buf.remaining() == 0 {
        // Buffer is empty
        return Ok(None);
    }

    let (len, advance) =
        i64::decode_var(buf.chunk()).ok_or_else(|| Error::from(ErrorKind::InvalidData))?;

    assert!(len >= 0, "Length of protobuf message must not be negative");

    if len == 0 {
        // Received empty request
        buf.advance(advance);
        return Ok(None);
    }

    let len = usize::try_from(len).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    if buf.remaining() < (advance + len) {
        // We haven't received all the data yet
        return Ok(None);
    }

    buf.advance(advance);
    let bytes = buf.copy_to_bytes(len);
    let message = M::decode(bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    Ok(Some(message))
}

pub(crate) fn encode<M, B>(message: M, buf: &mut B) -> Result<()>
where
    M: Message,
    B: BufMut,
{
    let len = i64::try_from(message.encoded_len()).map_err(|e| Error::new(ErrorKind::Other, e))?;
    let len_bytes = len.encode_var_vec();

    buf.put(len_bytes.as_ref());

    message
        .encode(buf)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use bytes::{Buf, BytesMut};
    use tendermint_proto::abci::{request::Value, Request, RequestFlush, RequestInfo};

    use super::{decode, encode};

    #[test]
    fn check_decoding() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[
            30, 26, 13, 10, 7, 118, 48, 46, 51, 52, 46, 51, 16, 11, 24, 8, 4, 18, 0,
        ]);

        let request = decode::<Request, _>(&mut buf);
        assert!(request.is_ok());
        let request = request.unwrap();
        assert!(request.is_some());
        let request = request.unwrap();
        assert_eq!(
            request,
            Request {
                value: Some(Value::Info(RequestInfo {
                    version: "v0.34.3".to_string(),
                    block_version: 11,
                    p2p_version: 8
                }))
            }
        );

        let request = decode::<Request, _>(&mut buf);
        assert!(request.is_ok());
        let request = request.unwrap();
        assert!(request.is_some());
        let request = request.unwrap();
        assert_eq!(
            request,
            Request {
                value: Some(Value::Flush(RequestFlush {}))
            }
        );

        let request = decode::<Request, _>(&mut buf);
        assert!(request.is_ok());
        let request = request.unwrap();
        assert!(request.is_none());

        assert_eq!(0, buf.remaining());
    }

    #[test]
    fn check_encoding() {
        let mut buf = BytesMut::new();

        let request = Request {
            value: Some(Value::Flush(RequestFlush {})),
        };
        encode(request, &mut buf).unwrap();
        assert_eq!([4, 18, 0], buf.chunk());

        buf.clear();

        let request = Request {
            value: Some(Value::Info(RequestInfo {
                version: "v0.34.3".to_string(),
                block_version: 11,
                p2p_version: 8,
            })),
        };

        encode(request, &mut buf).unwrap();
        assert_eq!(
            [30, 26, 13, 10, 7, 118, 48, 46, 51, 52, 46, 51, 16, 11, 24, 8],
            buf.chunk()
        );
    }
}
