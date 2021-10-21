use tendermint_proto::abci::{request::Value as RequestValue, Request};

use crate::types::*;

pub fn info() -> Request {
    Request {
        value: Some(RequestValue::Info(RequestInfo::default())),
    }
}

pub fn init_chain() -> Request {
    Request {
        value: Some(RequestValue::InitChain(Default::default())),
    }
}

pub fn begin_block(block_height: i64, app_hash: Vec<u8>) -> Request {
    let header = Header {
        height: block_height,
        app_hash,
        ..Default::default()
    };

    let begin_block_request = RequestBeginBlock {
        header: Some(header),
        ..Default::default()
    };

    Request {
        value: Some(RequestValue::BeginBlock(begin_block_request)),
    }
}

pub fn check_tx(counter: u64, recheck: bool) -> Request {
    let mut check_tx_request = RequestCheckTx {
        tx: counter.to_be_bytes().to_vec(),
        ..Default::default()
    };

    if recheck {
        check_tx_request.set_type(CheckTxType::Recheck);
    }

    Request {
        value: Some(RequestValue::CheckTx(check_tx_request)),
    }
}

pub fn deliver_tx(counter: u64) -> Request {
    let deliver_tx_request = RequestDeliverTx {
        tx: counter.to_be_bytes().to_vec(),
    };

    Request {
        value: Some(RequestValue::DeliverTx(deliver_tx_request)),
    }
}

pub fn end_block(block_height: i64) -> Request {
    let end_block_request = RequestEndBlock {
        height: block_height,
    };

    Request {
        value: Some(RequestValue::EndBlock(end_block_request)),
    }
}

pub fn commit() -> Request {
    Request {
        value: Some(RequestValue::Commit(RequestCommit::default())),
    }
}
