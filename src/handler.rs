use std::sync::Arc;

#[cfg(feature = "use-async-std")]
use async_std::sync::Mutex;
use tendermint_proto::abci::{
    request::Value as RequestValue, response::Value as ResponseValue, Response, ResponseException,
};
#[cfg(feature = "use-tokio")]
use tokio::sync::Mutex;
use tracing::{debug, instrument};

use crate::{
    state::ConsensusStateValidator, types::ResponseEcho, Consensus, Info, Mempool, Snapshot,
};

#[instrument]
pub fn handle_unknown_request(request_value: RequestValue) -> Response {
    let response_value = match request_value {
        RequestValue::Echo(request) => ResponseValue::Echo(ResponseEcho {
            message: request.message,
        }),
        RequestValue::Flush(_) => ResponseValue::Flush(Default::default()),
        _ => unreachable!("handle_unknown_request cannot handle known requests"),
    };

    let mut response = Response::default();
    response.value = Some(response_value);

    response
}

#[instrument(skip(consensus))]
pub async fn handle_consensus_request<C: Consensus>(
    consensus: Arc<C>,
    validator: Arc<Mutex<ConsensusStateValidator>>,
    request_value: RequestValue,
) -> Response {
    let response_value = match request_value {
        RequestValue::Echo(request) => ResponseValue::Echo(consensus.echo(request).await),
        RequestValue::Flush(request) => ResponseValue::Flush(consensus.flush(request).await),
        RequestValue::InitChain(request) => match validator.lock().await.on_init_chain_request() {
            Ok(_) => ResponseValue::InitChain(consensus.init_chain(request).await),
            Err(error) => ResponseValue::Exception(ResponseException { error }),
        },
        RequestValue::BeginBlock(request) => {
            match validator.lock().await.on_begin_block_request(&request) {
                Ok(_) => ResponseValue::BeginBlock(consensus.begin_block(request).await),
                Err(error) => ResponseValue::Exception(ResponseException { error }),
            }
        }
        RequestValue::DeliverTx(request) => match validator.lock().await.on_deliver_tx_request() {
            Ok(_) => ResponseValue::DeliverTx(consensus.deliver_tx(request).await),
            Err(error) => ResponseValue::Exception(ResponseException { error }),
        },
        RequestValue::EndBlock(request) => {
            match validator.lock().await.on_end_block_request(&request) {
                Ok(_) => ResponseValue::EndBlock(consensus.end_block(request).await),
                Err(error) => ResponseValue::Exception(ResponseException { error }),
            }
        }
        RequestValue::Commit(request) => {
            let mut validator_locked = validator.lock().await;

            match validator_locked.on_commit_request() {
                Ok(_) => {
                    let response = consensus.commit(request).await;

                    match validator_locked.on_commit_response(&response) {
                        Ok(_) => ResponseValue::Commit(response),
                        Err(error) => ResponseValue::Exception(ResponseException { error }),
                    }
                }
                Err(error) => ResponseValue::Exception(ResponseException { error }),
            }
        }
        _ => ResponseValue::Exception(ResponseException {
            error: "Non-consensus request on consensus connection".to_string(),
        }),
    };

    let mut response = Response::default();
    response.value = Some(response_value);

    debug!(message = "Sending response", ?response);

    response
}

#[instrument(skip(mempool))]
pub async fn handle_mempool_request<M: Mempool>(
    mempool: Arc<M>,
    request_value: RequestValue,
) -> Response {
    let response_value = match request_value {
        RequestValue::Echo(request) => ResponseValue::Echo(mempool.echo(request).await),
        RequestValue::Flush(request) => ResponseValue::Flush(mempool.flush(request).await),
        RequestValue::CheckTx(request) => ResponseValue::CheckTx(mempool.check_tx(request).await),
        _ => ResponseValue::Exception(ResponseException {
            error: "Non-mempool request on mempool connection".to_string(),
        }),
    };

    let mut response = Response::default();
    response.value = Some(response_value);

    debug!(message = "Sending response", ?response);

    response
}

#[instrument(skip(info))]
pub async fn handle_info_request<I: Info>(
    info: Arc<I>,
    validator: Arc<Mutex<ConsensusStateValidator>>,
    request_value: RequestValue,
) -> Response {
    let response_value = match request_value {
        RequestValue::Echo(request) => ResponseValue::Echo(info.echo(request).await),
        RequestValue::Flush(request) => ResponseValue::Flush(info.flush(request).await),
        RequestValue::Info(request) => {
            let info_response = info.info(request).await;
            validator.lock().await.on_info_response(&info_response);
            ResponseValue::Info(info_response)
        }
        RequestValue::SetOption(request) => {
            ResponseValue::SetOption(info.set_option(request).await)
        }
        RequestValue::Query(request) => ResponseValue::Query(info.query(request).await),
        _ => ResponseValue::Exception(ResponseException {
            error: "Non-info request on info connection".to_string(),
        }),
    };

    let mut response = Response::default();
    response.value = Some(response_value);

    debug!(message = "Sending response", ?response);

    response
}

#[instrument(skip(snapshot))]
pub async fn handle_snapshot_request<S: Snapshot>(
    snapshot: Arc<S>,
    request_value: RequestValue,
) -> Response {
    let response_value = match request_value {
        RequestValue::Echo(request) => ResponseValue::Echo(snapshot.echo(request).await),
        RequestValue::Flush(request) => ResponseValue::Flush(snapshot.flush(request).await),
        RequestValue::ListSnapshots(request) => {
            ResponseValue::ListSnapshots(snapshot.list_snapshots(request).await)
        }
        RequestValue::OfferSnapshot(request) => {
            ResponseValue::OfferSnapshot(snapshot.offer_snapshot(request).await)
        }
        RequestValue::LoadSnapshotChunk(request) => {
            ResponseValue::LoadSnapshotChunk(snapshot.load_snapshot_chunk(request).await)
        }
        RequestValue::ApplySnapshotChunk(request) => {
            ResponseValue::ApplySnapshotChunk(snapshot.apply_snapshot_chunk(request).await)
        }
        _ => ResponseValue::Exception(ResponseException {
            error: "Non-snapshot request on snapshot connection".to_string(),
        }),
    };

    let mut response = Response::default();
    response.value = Some(response_value);

    debug!(message = "Sending response", ?response);

    response
}
