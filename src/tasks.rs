use std::sync::Arc;

#[cfg(feature = "use-async-std")]
use async_channel::unbounded;
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
    sync::{mpsc::unbounded_channel, Mutex},
};
use tracing::{debug, error, instrument};

use crate::{
    handler::*,
    state::ConsensusStateValidator,
    stream_split::StreamSplit,
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
pub fn spawn_mempool_task<D, P, M>(stream: D, peer_addr: Option<P>, mempool: Arc<M>)
where
    D: Read + Write + StreamSplit + Send + Unpin + 'static,
    P: std::fmt::Debug + Sync + Send + 'static,
    M: Mempool + 'static,
{
    let (mut reader, mut writer) = stream.split_stream();

    #[cfg(feature = "use-async-std")]
    let (sender, receiver) = unbounded();

    #[cfg(feature = "use-tokio")]
    let (sender, mut receiver) = unbounded_channel();

    let peer_addr = Arc::new(peer_addr);
    let peer_address = peer_addr.clone();

    spawn(async move {
        #[cfg(feature = "use-async-std")]
        while let Ok(handle) = receiver.recv().await {
            let response = handle.await;

            if let Err(err) = encode(response, &mut writer).await {
                error!(message = "Error while writing to stream", %err, ?peer_addr);
            }
        }

        #[cfg(feature = "use-tokio")]
        while let Some(handle) = receiver.recv().await {
            let response = handle.await;

            match response {
                Ok(response) => {
                    if let Err(err) = encode(response, &mut writer).await {
                        error!(message = "Error while writing to stream", %err, ?peer_addr);
                    }
                }
                Err(err) => error!(message = "Mempool request execution not completed", ?err),
            }
        }
    });

    spawn(async move {
        while let Ok(request) = decode(&mut reader).await {
            match request {
                None => {
                    debug!(message = "Received empty request", peer_addr = ?peer_address.clone())
                }
                Some(request) => {
                    let peer_addr = peer_address.clone();
                    let mempool = mempool.clone();

                    let handle = spawn(async move {
                        let request: Request = request;

                        match request.value {
                            None => {
                                debug!(
                                    message = "Received empty value in request",
                                    ?peer_addr,
                                    ?request
                                );
                                Response::default()
                            }
                            Some(request_value) => {
                                handle_mempool_request(mempool, request_value).await
                            }
                        }
                    });

                    #[cfg(feature = "use-async-std")]
                    sender.send(handle).await.expect("Channel receiver dropper");

                    #[cfg(feature = "use-tokio")]
                    sender.send(handle).expect("Channel receiver dropped");
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
