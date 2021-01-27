use std::sync::Arc;

#[cfg(feature = "use-async-std")]
use async_std::{
    channel::{unbounded as unbounded_channel, Receiver, Sender},
    io::{Read, Write},
    sync::Mutex,
    task::{spawn, JoinHandle},
};
#[cfg(feature = "use-smol")]
use smol::{
    channel::{unbounded as unbounded_channel, Receiver, Sender},
    io::{AsyncRead as Read, AsyncWrite as Write},
    lock::Mutex,
    spawn, Task as JoinHandle,
};
use tendermint_proto::abci::{Request, Response};
#[cfg(feature = "use-tokio")]
use tokio::{
    io::{AsyncRead as Read, AsyncWrite as Write},
    spawn,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::{debug, error, info, instrument};

use crate::{
    async_api::{Consensus, Info, Mempool, Snapshot},
    handler::*,
    state::ConsensusStateValidator,
    utils::{StreamReader, StreamWriter},
};

macro_rules! spawn {
    ($expr: expr) => {
        cfg_if::cfg_if! {
            if #[cfg(any(feature = "use-async-std", feature = "use-tokio"))] {
                spawn($expr)
            } else if #[cfg(feature = "use-smol")] {
                spawn($expr)
                    .detach()
            } else {
                unreachable!()
            }
        }
    };
}

#[instrument(skip(stream_reader, stream_writer, consensus))]
pub fn spawn_consensus_task<R, W, C>(
    stream_reader: StreamReader<R>,
    stream_writer: StreamWriter<W>,
    peer_addr: String,
    consensus: Arc<C>,
    validator: Arc<Mutex<ConsensusStateValidator>>,
) where
    R: Read + Unpin + Send + 'static,
    W: Write + Unpin + Send + 'static,
    C: Consensus + 'static,
{
    info!(message = "Spawning consensus task");

    spawn!(async move {
        consensus_task(
            stream_reader,
            stream_writer,
            peer_addr,
            consensus.as_ref(),
            validator,
        )
        .await
    });
}

#[instrument(skip(stream_reader, stream_writer, consensus))]
async fn consensus_task<R, W, C>(
    mut stream_reader: StreamReader<R>,
    mut stream_writer: StreamWriter<W>,
    peer_addr: String,
    consensus: &C,
    validator: Arc<Mutex<ConsensusStateValidator>>,
) where
    R: Read + Unpin,
    W: Write + Unpin,
    C: Consensus,
{
    while let Ok(request) = stream_reader.read().await {
        match request {
            None => debug!(message = "Received empty request"),
            Some(request) => {
                let request: Request = request;

                let response = match request.value {
                    None => {
                        debug!(message = "Received empty value in request", ?request);
                        Response::default()
                    }
                    Some(request_value) => {
                        handle_consensus_request(consensus, validator.clone(), request_value).await
                    }
                };

                if let Err(err) = stream_writer.write(response).await {
                    error!(message = "Error while writing to stream", %err);
                }
            }
        }
    }
}

#[instrument(skip(stream_reader, stream_writer, mempool))]
pub fn spawn_mempool_task<R, W, M>(
    stream_reader: StreamReader<R>,
    stream_writer: StreamWriter<W>,
    peer_addr: String,
    mempool: Arc<M>,
) where
    R: Read + Unpin + Send + 'static,
    W: Write + Unpin + Send + 'static,
    M: Mempool + 'static,
{
    info!(message = "Spawning mempool tasks");

    let (handle_sender, handle_receiver) = unbounded_channel();
    let peer_addr_clone = peer_addr.clone();

    spawn!(
        async move { mempool_writer_task(stream_writer, peer_addr_clone, handle_receiver).await }
    );

    spawn!(
        async move { mempool_reader_task(stream_reader, peer_addr, mempool, handle_sender).await }
    );
}

#[instrument(skip(stream_writer, handle_receiver))]
async fn mempool_writer_task<W>(
    mut stream_writer: StreamWriter<W>,
    peer_addr: String,
    handle_receiver: Receiver<JoinHandle<Response>>,
) where
    W: Write + Unpin,
{
    cfg_if::cfg_if! {
        if #[cfg(any(feature = "use-async-std", feature = "use-smol"))] {
            while let Ok(handle) = handle_receiver.recv().await {
                let response = handle.await;

                if let Err(err) = stream_writer.write(response).await {
                    error!(message = "Error while writing to stream", %err);
                }
            }
        } else if #[cfg(feature = "use-tokio")] {
            let mut handle_receiver = handle_receiver;

            while let Some(handle) = handle_receiver.recv().await {
                let response = handle.await;

                match response {
                    Ok(response) => {
                        if let Err(err) = stream_writer.write(response).await {
                            error!(message = "Error while writing to stream", %err);
                        }
                    }
                    Err(err) => error!(message = "Mempool request execution not completed", ?err),
                }
            }
        } else {
            unreachable!()
        }
    }
}

#[instrument(skip(stream_reader, mempool, handle_sender))]
async fn mempool_reader_task<R, M>(
    mut stream_reader: StreamReader<R>,
    peer_addr: String,
    mempool: Arc<M>,
    handle_sender: Sender<JoinHandle<Response>>,
) where
    R: Read + Unpin,
    M: Mempool + 'static,
{
    while let Ok(request) = stream_reader.read().await {
        match request {
            None => {
                debug!(message = "Received empty request")
            }
            Some(request) => {
                let peer_addr = peer_addr.clone();
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
                            handle_mempool_request(mempool.as_ref(), request_value).await
                        }
                    }
                });

                cfg_if::cfg_if! {
                    if #[cfg(any(feature = "use-async-std", feature = "use-smol"))] {
                        handle_sender
                            .send(handle)
                            .await
                            .expect("Channel receiver dropped");
                    } else if #[cfg(feature = "use-tokio")] {
                        handle_sender
                            .send(handle)
                            .expect("Channel receiver dropped");
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    }
}

#[instrument(skip(stream_reader, stream_writer, info))]
pub fn spawn_info_task<R, W, I>(
    stream_reader: StreamReader<R>,
    stream_writer: StreamWriter<W>,
    peer_addr: String,
    info: Arc<I>,
    validator: Arc<Mutex<ConsensusStateValidator>>,
) where
    R: Read + Unpin + Send + 'static,
    W: Write + Unpin + Send + 'static,
    I: Info + 'static,
{
    info!(message = "Spawning info task");

    spawn!(async move {
        info_task(
            stream_reader,
            stream_writer,
            peer_addr,
            info.as_ref(),
            validator,
        )
        .await
    });
}

#[instrument(skip(stream_reader, stream_writer, info))]
async fn info_task<R, W, I>(
    mut stream_reader: StreamReader<R>,
    mut stream_writer: StreamWriter<W>,
    peer_addr: String,
    info: &I,
    validator: Arc<Mutex<ConsensusStateValidator>>,
) where
    R: Read + Unpin,
    W: Write + Unpin,
    I: Info,
{
    while let Ok(request) = stream_reader.read().await {
        match request {
            None => debug!(message = "Received empty request"),
            Some(request) => {
                let request: Request = request;

                let response = match request.value {
                    None => {
                        debug!(message = "Received empty value in request", ?request);
                        Response::default()
                    }
                    Some(request_value) => {
                        handle_info_request(info, validator.clone(), request_value).await
                    }
                };

                if let Err(err) = stream_writer.write(response).await {
                    error!(message = "Error while writing to stream", %err);
                }
            }
        }
    }
}

#[instrument(skip(stream_reader, stream_writer, snapshot))]
pub fn spawn_snapshot_task<R, W, S>(
    stream_reader: StreamReader<R>,
    stream_writer: StreamWriter<W>,
    peer_addr: String,
    snapshot: Arc<S>,
) where
    R: Read + Unpin + Send + 'static,
    W: Write + Unpin + Send + 'static,
    S: Snapshot + 'static,
{
    info!(message = "Spawning snapshot task");

    spawn!(async move {
        snapshot_task(stream_reader, stream_writer, peer_addr, snapshot.as_ref()).await
    });
}

#[instrument(skip(stream_reader, stream_writer, snapshot))]
async fn snapshot_task<R, W, S>(
    mut stream_reader: StreamReader<R>,
    mut stream_writer: StreamWriter<W>,
    peer_addr: String,
    snapshot: &S,
) where
    R: Read + Unpin,
    W: Write + Unpin,
    S: Snapshot,
{
    while let Ok(request) = stream_reader.read().await {
        match request {
            None => debug!(message = "Received empty request"),
            Some(request) => {
                let request: Request = request;

                let response = match request.value {
                    None => {
                        debug!(message = "Received empty value in request", ?request);
                        Response::default()
                    }
                    Some(request_value) => handle_snapshot_request(snapshot, request_value).await,
                };

                if let Err(err) = stream_writer.write(response).await {
                    error!(message = "Error while writing to stream", %err);
                }
            }
        }
    }
}
