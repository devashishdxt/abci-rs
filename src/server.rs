#[cfg(unix)]
use std::path::PathBuf;
use std::{io::Result, net::SocketAddr, sync::Arc};

#[cfg(all(unix, feature = "use-async-std"))]
use async_std::os::unix::net::UnixListener;
#[cfg(feature = "use-async-std")]
use async_std::{
    io::{Read, Write},
    net::TcpListener,
    prelude::*,
    sync::Mutex,
    task::spawn,
};
#[cfg(test)]
use mock_io::tokio::MockListener;
use tendermint_proto::abci::{request::Value as RequestValue, Request, Response};
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
    handler::*,
    state::ConsensusStateValidator,
    stream_split::StreamSplit,
    tasks::*,
    types::{decode, encode},
    Consensus, Info, Mempool, Snapshot,
};

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
    /// Creates a new instance of [`Server`](struct.Server.html)
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

                #[cfg(feature = "use-async-std")]
                {
                    let mut incoming = listener.incoming();

                    while let Some(stream) = incoming.next().await {
                        let stream = stream?;
                        let peer_addr = stream.peer_addr().ok();
                        self.handle_connection(stream, peer_addr);
                    }

                    Ok(())
                }

                #[cfg(feature = "use-tokio")]
                {
                    loop {
                        let (stream, peer_addr) = listener.accept().await?;
                        self.handle_connection(stream, Some(peer_addr));
                    }
                }
            }
            #[cfg(unix)]
            Address::Uds(path) => {
                #[cfg(feature = "use-async-std")]
                let listener = UnixListener::bind(&path).await?;

                #[cfg(feature = "use-tokio")]
                let listener = UnixListener::bind(&path)?;

                info!(message = "Started ABCI server at", path = %path.display());

                #[cfg(feature = "use-async-std")]
                {
                    let mut incoming = listener.incoming();

                    while let Some(stream) = incoming.next().await {
                        let stream = stream?;
                        let peer_addr = stream.peer_addr().ok();
                        self.handle_connection(stream, peer_addr);
                    }

                    Ok(())
                }

                #[cfg(feature = "use-tokio")]
                {
                    loop {
                        let (stream, peer_addr) = listener.accept().await?;
                        self.handle_connection(stream, Some(peer_addr));
                    }
                }
            }
            #[cfg(test)]
            Address::Mock(mut listener) => {
                while let Ok(stream) = listener.accept().await {
                    self.handle_connection(stream, Some("test_peer"));
                }

                Ok(())
            }
        }
    }

    #[instrument(skip(self, stream))]
    pub(crate) fn handle_connection<D, P>(&self, mut stream: D, peer_addr: Option<P>)
    where
        D: Read + Write + StreamSplit + Send + Unpin + 'static,
        P: std::fmt::Debug + Send + Sync + 'static,
    {
        info!("New peer connection");

        let inner = self.inner.clone();

        spawn(async move {
            loop {
                match decode(&mut stream).await {
                    Ok(request) => match request {
                        Some(request) => {
                            let (response, connection_type) = inner.process(request).await;

                            if let Err(err) = encode(response, &mut stream).await {
                                error!(message = "Error while writing to stream", %err, ?peer_addr);
                            }

                            if !connection_type.is_unknown() {
                                inner.spawn_connection(stream, peer_addr, connection_type);
                                break;
                            }
                        }
                        None => debug!(message = "Received empty request", ?peer_addr),
                    },
                    Err(err) => {
                        error!(
                            message = "Error while receiving ABCI request from socket",
                            ?peer_addr, %err
                        );
                        break;
                    }
                }
            }
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

    #[instrument(skip(self, stream))]
    fn spawn_connection<D, P>(
        &self,
        stream: D,
        peer_addr: Option<P>,
        connection_type: ConnectionType,
    ) where
        D: Read + Write + StreamSplit + Send + Unpin + 'static,
        P: std::fmt::Debug + Sync + Send + 'static,
    {
        debug!("Spawning a new connection task");

        match connection_type {
            ConnectionType::Unknown => unreachable!(
                "Connection type cannot be unknown when spawning a task for a connection type"
            ),
            ConnectionType::Consensus => spawn_consensus_task(
                stream,
                peer_addr,
                self.consensus.clone(),
                self.validator.clone(),
            ),
            ConnectionType::Mempool => spawn_mempool_task(stream, peer_addr, self.mempool.clone()),
            ConnectionType::Info => {
                spawn_info_task(stream, peer_addr, self.info.clone(), self.validator.clone())
            }
            ConnectionType::Snapshot => {
                spawn_snapshot_task(stream, peer_addr, self.snapshot.clone())
            }
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
                            self.consensus.clone(),
                            self.validator.clone(),
                            request_value,
                        )
                        .await
                    }
                    ConnectionType::Mempool => {
                        handle_mempool_request(self.mempool.clone(), request_value).await
                    }
                    ConnectionType::Info => {
                        handle_info_request(
                            self.info.clone(),
                            self.validator.clone(),
                            request_value,
                        )
                        .await
                    }
                    ConnectionType::Snapshot => {
                        handle_snapshot_request(self.snapshot.clone(), request_value).await
                    }
                };

                (response, connection_type)
            }
        }
    }
}

/// Address of ABCI Server
#[derive(Debug)]
pub enum Address {
    /// TCP Address
    Tcp(SocketAddr),
    /// UDS Address
    #[cfg(unix)]
    #[cfg_attr(feature = "doc", doc(cfg(unix)))]
    Uds(PathBuf),
    /// Mock Address
    #[cfg(test)]
    Mock(MockListener),
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Self {
        Self::Tcp(addr)
    }
}

#[cfg(unix)]
impl From<PathBuf> for Address {
    fn from(path: PathBuf) -> Self {
        Self::Uds(path)
    }
}

#[cfg(test)]
impl From<MockListener> for Address {
    fn from(listener: MockListener) -> Self {
        Self::Mock(listener)
    }
}

/// Different types of connections created by tendermint
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ConnectionType {
    Unknown,
    Consensus,
    Mempool,
    Info,
    Snapshot,
}

impl ConnectionType {
    /// Returns true of connection type is unknown
    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl Default for ConnectionType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<&RequestValue> for ConnectionType {
    fn from(request_value: &RequestValue) -> Self {
        match request_value {
            RequestValue::Echo(_) | RequestValue::Flush(_) => Self::Unknown,
            RequestValue::InitChain(_)
            | RequestValue::BeginBlock(_)
            | RequestValue::DeliverTx(_)
            | RequestValue::EndBlock(_)
            | RequestValue::Commit(_) => Self::Consensus,
            RequestValue::CheckTx(_) => Self::Mempool,
            RequestValue::Info(_) | RequestValue::SetOption(_) | RequestValue::Query(_) => {
                Self::Info
            }
            RequestValue::ListSnapshots(_)
            | RequestValue::OfferSnapshot(_)
            | RequestValue::LoadSnapshotChunk(_)
            | RequestValue::ApplySnapshotChunk(_) => Self::Snapshot,
        }
    }
}
