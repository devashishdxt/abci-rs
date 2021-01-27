use std::{io::Result, sync::Arc};

#[cfg(all(unix, feature = "use-async-std"))]
use async_std::os::unix::net::UnixListener;
#[cfg(feature = "use-async-std")]
use async_std::{
    io::{Read, Write},
    net::TcpListener,
    sync::Mutex,
    task::spawn,
};
#[cfg(all(unix, feature = "use-smol"))]
use smol::net::unix::UnixListener;
#[cfg(feature = "use-smol")]
use smol::{
    io::{AsyncRead as Read, AsyncWrite as Write},
    lock::Mutex,
    net::TcpListener,
    spawn,
};
use tendermint_proto::abci::{Request, Response};
#[cfg(all(unix, feature = "use-tokio"))]
use tokio::net::UnixListener;
#[cfg(feature = "use-tokio")]
use tokio::{
    io::{AsyncRead as Read, AsyncWrite as Write},
    net::TcpListener,
    spawn,
    sync::Mutex,
};
use tracing::{debug, error, info, instrument};

use crate::{
    address::Address,
    async_api::{Consensus, Info, Mempool, Snapshot},
    handler::*,
    state::ConsensusStateValidator,
    stream_split::StreamSplit,
    tasks::*,
    utils::{get_stream_pair, ConnectionType, StreamReader, StreamWriter},
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

/// ABCI Server
pub struct Server<C, M, I, S>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: Snapshot + 'static,
{
    /// Wrapping inner type in `Arc` so that it becomes clonable and can be shared between multiple
    /// async tasks
    inner: Arc<Inner<C, M, I, S>>,
}

impl<C, M, I, S> Server<C, M, I, S>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: Snapshot + 'static,
{
    /// Creates a new instance of [`Server`](self::Server)
    pub fn new(consensus: C, mempool: M, info: I, snapshot: S) -> Self {
        Self {
            inner: Arc::new(Inner::new(consensus, mempool, info, snapshot)),
        }
    }

    /// Starts ABCI server
    ///
    /// # Note
    ///
    /// This is an `async` function and returns a `Future`. So, you'll need an executor to drive the `Future` returned
    /// from this function. `async-std` and `tokio` are two popular options.
    pub async fn run<T>(&self, addr: T) -> Result<()>
    where
        T: Into<Address>,
    {
        let addr = addr.into();

        match addr {
            Address::Tcp(addr) => {
                let listener = TcpListener::bind(addr).await?;
                info!(message = "Started ABCI server at", %addr);

                loop {
                    let (stream, peer_addr) = listener.accept().await?;
                    self.handle_connection(stream, peer_addr.to_string());
                }
            }
            #[cfg(unix)]
            Address::Uds(path) => {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "use-async-std")] {
                        let listener = UnixListener::bind(&path).await?;
                    } else if #[cfg(any(feature = "use-smol", feature = "use-tokio"))] {
                        let listener = UnixListener::bind(&path)?;
                    } else {
                        unreachable!()
                    }
                }

                info!(message = "Started ABCI server at", path = %path.display());

                loop {
                    let (stream, peer_addr) = listener.accept().await?;
                    self.handle_connection(stream, format!("{:?}", peer_addr));
                }
            }
            #[cfg(test)]
            Address::Mock(mut listener) => {
                while let Ok(stream) = listener.accept().await {
                    self.handle_connection(stream, "test_peer".to_string());
                }

                Ok(())
            }
        }
    }

    #[instrument(skip(self, stream))]
    pub(crate) fn handle_connection<D>(&self, stream: D, peer_addr: String)
    where
        D: StreamSplit,
    {
        info!("New peer connection");

        let inner = self.inner.clone();
        let (stream_reader, stream_writer) = get_stream_pair(stream);

        spawn!(async move {
            inner
                .handle_connection(stream_reader, stream_writer, peer_addr)
                .await
        });
    }
}

/// Inner type that contains all the trait implementations
struct Inner<C, M, I, S>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: Snapshot + 'static,
{
    consensus: Arc<C>,
    mempool: Arc<M>,
    info: Arc<I>,
    snapshot: Arc<S>,
    validator: Arc<Mutex<ConsensusStateValidator>>,
}

impl<C, M, I, S> Inner<C, M, I, S>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: Snapshot + 'static,
{
    pub fn new(consensus: C, mempool: M, info: I, snapshot: S) -> Self {
        Self {
            consensus: Arc::new(consensus),
            mempool: Arc::new(mempool),
            info: Arc::new(info),
            snapshot: Arc::new(snapshot),
            validator: Default::default(),
        }
    }

    #[instrument(skip(self, stream_reader, stream_writer))]
    async fn handle_connection<R, W>(
        self: Arc<Self>,
        mut stream_reader: StreamReader<R>,
        mut stream_writer: StreamWriter<W>,
        peer_addr: String,
    ) where
        R: Read + Unpin + Send + 'static,
        W: Write + Unpin + Send + 'static,
    {
        info!(message = "In handle_connection");

        loop {
            match stream_reader.read().await {
                Ok(request) => match request {
                    Some(request) => {
                        let (response, connection_type) = self.process(request).await;

                        if let Err(err) = stream_writer.write(response).await {
                            error!(message = "Error while writing to stream", %err);
                        }

                        if !matches!(connection_type, ConnectionType::Unknown) {
                            self.spawn_connection(
                                stream_reader,
                                stream_writer,
                                peer_addr,
                                connection_type,
                            );
                            break;
                        }
                    }
                    None => debug!(message = "Received empty request"),
                },
                Err(err) => {
                    error!(message = "Error while receiving ABCI request from socket", %err);
                    break;
                }
            }
        }
    }

    #[instrument(skip(self, stream_reader, stream_writer))]
    fn spawn_connection<R, W>(
        &self,
        stream_reader: StreamReader<R>,
        stream_writer: StreamWriter<W>,
        peer_addr: String,
        connection_type: ConnectionType,
    ) where
        R: Read + Unpin + Send + 'static,
        W: Write + Unpin + Send + 'static,
    {
        debug!("Spawning a new connection task");

        match connection_type {
            ConnectionType::Unknown => unreachable!(
                "Connection type cannot be unknown when spawning a task for a connection type"
            ),
            ConnectionType::Consensus => spawn_consensus_task(
                stream_reader,
                stream_writer,
                peer_addr,
                self.consensus.clone(),
                self.validator.clone(),
            ),
            ConnectionType::Mempool => spawn_mempool_task(
                stream_reader,
                stream_writer,
                peer_addr,
                self.mempool.clone(),
            ),
            ConnectionType::Info => spawn_info_task(
                stream_reader,
                stream_writer,
                peer_addr,
                self.info.clone(),
                self.validator.clone(),
            ),
            ConnectionType::Snapshot => spawn_snapshot_task(
                stream_reader,
                stream_writer,
                peer_addr,
                self.snapshot.clone(),
            ),
        }
    }

    #[instrument(skip(self))]
    async fn process(&self, request: Request) -> (Response, ConnectionType) {
        match request.value {
            None => {
                debug!(message = "Received empty value in request", ?request);

                (Response::default(), ConnectionType::default())
            }
            Some(request_value) => {
                let connection_type = ConnectionType::from(&request_value);

                let response = match connection_type {
                    ConnectionType::Unknown => handle_unknown_request(request_value),
                    ConnectionType::Consensus => {
                        handle_consensus_request(
                            self.consensus.as_ref(),
                            self.validator.clone(),
                            request_value,
                        )
                        .await
                    }
                    ConnectionType::Mempool => {
                        handle_mempool_request(self.mempool.as_ref(), request_value).await
                    }
                    ConnectionType::Info => {
                        handle_info_request(
                            self.info.as_ref(),
                            self.validator.clone(),
                            request_value,
                        )
                        .await
                    }
                    ConnectionType::Snapshot => {
                        handle_snapshot_request(self.snapshot.as_ref(), request_value).await
                    }
                };

                (response, connection_type)
            }
        }
    }
}
