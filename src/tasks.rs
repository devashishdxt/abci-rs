use std::sync::Arc;

#[cfg(feature = "use-async-std")]
use async_std::{
    io::{Read, Write},
    sync::Mutex,
    task::spawn,
};
use tendermint_proto::abci::{Request, Response};
#[cfg(feature = "use-tokio")]
use tokio::{
    io::{AsyncRead as Read, AsyncWrite as Write},
    spawn,
    sync::Mutex,
};
use tracing::{debug, error, instrument};

use crate::{
    handler::*,
    state::ConsensusStateValidator,
    types::{decode, encode},
    Consensus, Info, Mempool, Snapshot,
};

#[instrument(skip(stream, consensus))]
pub fn spawn_consensus_task<D, P, C>(
    mut stream: D,
    peer_addr: Option<P>,
    consensus: Arc<C>,
    validator: Arc<Mutex<ConsensusStateValidator>>,
) where
    D: Read + Write + Send + Unpin + 'static,
    P: std::fmt::Debug + Send + 'static,
    C: Consensus + 'static,
{
    spawn(async move {
        while let Ok(request) = decode(&mut stream).await {
            match request {
                None => debug!(message = "Received empty request", ?peer_addr),
                Some(request) => {
                    let request: Request = request;

                    let response = match request.value {
                        None => {
                            debug!(
                                message = "Received empty value in request",
                                ?peer_addr,
                                ?request
                            );
                            Response::default()
                        }
                        Some(request_value) => {
                            handle_consensus_request(
                                consensus.clone(),
                                validator.clone(),
                                request_value,
                            )
                            .await
                        }
                    };

                    if let Err(err) = encode(response, &mut stream).await {
                        error!(message = "Error while writing to stream", %err, ?peer_addr);
                    }
                }
            }
        }
    });
}

#[instrument(skip(stream, mempool))]
pub fn spawn_mempool_task<D, P, M>(mut stream: D, peer_addr: Option<P>, mempool: Arc<M>)
where
    D: Read + Write + Send + Unpin + 'static,
    P: std::fmt::Debug + Send + 'static,
    M: Mempool + 'static,
{
    spawn(async move {
        while let Ok(request) = decode(&mut stream).await {
            match request {
                None => debug!(message = "Received empty request", ?peer_addr),
                Some(request) => {
                    let request: Request = request;

                    let response = match request.value {
                        None => {
                            debug!(
                                message = "Received empty value in request",
                                ?peer_addr,
                                ?request
                            );
                            Response::default()
                        }
                        Some(request_value) => {
                            handle_mempool_request(mempool.clone(), request_value).await
                        }
                    };

                    if let Err(err) = encode(response, &mut stream).await {
                        error!(message = "Error while writing to stream", %err, ?peer_addr);
                    }
                }
            }
        }
    });
}

#[instrument(skip(stream, info))]
pub fn spawn_info_task<D, P, I>(
    mut stream: D,
    peer_addr: Option<P>,
    info: Arc<I>,
    validator: Arc<Mutex<ConsensusStateValidator>>,
) where
    D: Read + Write + Send + Unpin + 'static,
    P: std::fmt::Debug + Send + 'static,
    I: Info + 'static,
{
    spawn(async move {
        while let Ok(request) = decode(&mut stream).await {
            match request {
                None => debug!(message = "Received empty request", ?peer_addr),
                Some(request) => {
                    let request: Request = request;

                    let response = match request.value {
                        None => {
                            debug!(
                                message = "Received empty value in request",
                                ?peer_addr,
                                ?request
                            );
                            Response::default()
                        }
                        Some(request_value) => {
                            handle_info_request(info.clone(), validator.clone(), request_value)
                                .await
                        }
                    };

                    if let Err(err) = encode(response, &mut stream).await {
                        error!(message = "Error while writing to stream", %err, ?peer_addr);
                    }
                }
            }
        }
    });
}

#[instrument(skip(stream, snapshot))]
pub fn spawn_snapshot_task<D, P, S>(mut stream: D, peer_addr: Option<P>, snapshot: Arc<S>)
where
    D: Read + Write + Send + Unpin + 'static,
    P: std::fmt::Debug + Send + 'static,
    S: Snapshot + 'static,
{
    spawn(async move {
        while let Ok(request) = decode(&mut stream).await {
            match request {
                None => debug!(message = "Received empty request", ?peer_addr),
                Some(request) => {
                    let request: Request = request;

                    let response = match request.value {
                        None => {
                            debug!(
                                message = "Received empty value in request",
                                ?peer_addr,
                                ?request
                            );
                            Response::default()
                        }
                        Some(request_value) => {
                            handle_snapshot_request(snapshot.clone(), request_value).await
                        }
                    };

                    if let Err(err) = encode(response, &mut stream).await {
                        error!(message = "Error while writing to stream", %err, ?peer_addr);
                    }
                }
            }
        }
    });
}
