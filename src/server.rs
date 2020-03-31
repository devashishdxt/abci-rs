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
#[cfg(all(unix, feature = "use-tokio"))]
use tokio::net::UnixListener;
#[cfg(feature = "use-tokio")]
use tokio::{
    io::{AsyncRead as Read, AsyncWrite as Write},
    net::TcpListener,
    spawn,
    stream::StreamExt,
    sync::Mutex,
};
use tracing::{debug, error, info, instrument};

use crate::{
    proto::{abci::*, decode, encode},
    Consensus, Info, Mempool,
};

/// ABCI Server
pub struct Server<C, M, I>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
{
    pub(crate) consensus: Arc<C>,
    pub(crate) mempool: Arc<M>,
    pub(crate) info: Arc<I>,
    pub(crate) consensus_state: Arc<Mutex<ConsensusState>>,
}

impl<C, M, I> Server<C, M, I>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
{
    /// Creates a new instance of [`Server`](struct.Server.html)
    ///
    /// # Note
    ///
    /// This is an `async` function and returns a `Future`. So, you'll need an executor to drive the `Future` returned
    /// from this function. `async-std` and `tokio` are two popular options.
    ///
    /// # Arguments
    ///
    /// - `is_initialized` should be `true` if `InitChain` was previously executed. `false` otherwise.
    pub async fn new(consensus: C, mempool: M, info: I, is_initialized: bool) -> Result<Self> {
        let consensus_state = if is_initialized {
            ConsensusState::Initialized
        } else {
            ConsensusState::NotInitialized
        };

        Ok(Self {
            consensus: Arc::new(consensus),
            mempool: Arc::new(mempool),
            info: Arc::new(info),
            consensus_state: Arc::new(Mutex::new(consensus_state)),
        })
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
                #[cfg(feature = "use-async-std")]
                let listener = TcpListener::bind(addr).await?;

                #[cfg(feature = "use-tokio")]
                let mut listener = TcpListener::bind(addr).await?;

                info!(message = "Started ABCI server at", %addr);

                let mut incoming = listener.incoming();

                while let Some(stream) = incoming.next().await {
                    let stream = stream?;
                    let peer_addr = stream.peer_addr().ok();
                    self.handle_connection(stream, peer_addr).await;
                }
            }
            #[cfg(unix)]
            Address::Uds(path) => {
                #[cfg(feature = "use-async-std")]
                let listener = UnixListener::bind(&path).await?;

                #[cfg(feature = "use-tokio")]
                let mut listener = UnixListener::bind(&path)?;

                info!(message = "Started ABCI server at", path = %path.display());

                let mut incoming = listener.incoming();

                while let Some(stream) = incoming.next().await {
                    let stream = stream?;
                    let peer_addr = stream.peer_addr().ok();
                    self.handle_connection(stream, peer_addr).await;
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self, stream))]
    async fn handle_connection<S, P>(&self, mut stream: S, peer_addr: Option<P>)
    where
        S: Read + Write + Send + Unpin + 'static,
        P: std::fmt::Debug + Send + 'static,
    {
        info!("New peer connection");

        let consensus = self.consensus.clone();
        let mempool = self.mempool.clone();
        let info = self.info.clone();
        let consensus_state = self.consensus_state.clone();

        spawn(async move {
            while let Ok(request) = decode(&mut stream).await {
                match request {
                    Some(request) => {
                        let response = process(
                            consensus.clone(),
                            mempool.clone(),
                            info.clone(),
                            consensus_state.clone(),
                            request,
                        )
                        .await;

                        if let Err(err) = encode(response, &mut stream).await {
                            error!(message = "Error while writing to stream", %err, ?peer_addr);
                        }
                    }
                    None => debug!(message = "Received empty request", ?peer_addr),
                }
            }

            error!(
                message = "Error while receiving ABCI request from socket",
                ?peer_addr
            );
        });
    }
}

#[instrument(skip(consensus, mempool, info, consensus_state))]
async fn process<C, M, I>(
    consensus: Arc<C>,
    mempool: Arc<M>,
    info: Arc<I>,
    consensus_state: Arc<Mutex<ConsensusState>>,
    request: Request,
) -> Response
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
{
    let value = match request.value.unwrap() {
        Request_oneof_value::echo(request) => {
            let mut response = ResponseEcho::new();
            response.message = info.echo(request.message).await;
            Response_oneof_value::echo(response)
        }
        Request_oneof_value::flush(_) => {
            consensus.flush().await;
            Response_oneof_value::flush(ResponseFlush::new())
        }
        Request_oneof_value::info(request) => {
            Response_oneof_value::info(info.info(request.into()).await.into())
        }
        Request_oneof_value::set_option(request) => {
            Response_oneof_value::set_option(info.set_option(request.into()).await.into())
        }
        Request_oneof_value::init_chain(request) => {
            consensus_state
                .lock()
                .await
                .validate(ConsensusState::InitChain);
            Response_oneof_value::init_chain(consensus.init_chain(request.into()).await.into())
        }
        Request_oneof_value::query(request) => {
            Response_oneof_value::query(info.query(request.into()).await.into())
        }
        Request_oneof_value::begin_block(request) => {
            consensus_state
                .lock()
                .await
                .validate(ConsensusState::BeginBlock);
            Response_oneof_value::begin_block(consensus.begin_block(request.into()).await.into())
        }
        Request_oneof_value::check_tx(request) => {
            Response_oneof_value::check_tx(mempool.check_tx(request.into()).await.into())
        }
        Request_oneof_value::deliver_tx(request) => {
            consensus_state
                .lock()
                .await
                .validate(ConsensusState::DeliverTx);
            Response_oneof_value::deliver_tx(consensus.deliver_tx(request.into()).await.into())
        }
        Request_oneof_value::end_block(request) => {
            consensus_state
                .lock()
                .await
                .validate(ConsensusState::EndBlock);
            Response_oneof_value::end_block(consensus.end_block(request.into()).await.into())
        }
        Request_oneof_value::commit(_) => {
            consensus_state
                .lock()
                .await
                .validate(ConsensusState::Commit);
            Response_oneof_value::commit(consensus.commit().await.into())
        }
    };

    let mut response = Response::new();
    response.value = Some(value);

    debug!(message = "Sending response", ?response);

    response
}

#[derive(Debug, Clone, Copy)]
pub enum ConsensusState {
    NotInitialized,
    Initialized,
    InitChain,
    BeginBlock,
    DeliverTx,
    EndBlock,
    Commit,
}

impl Default for ConsensusState {
    #[inline]
    fn default() -> Self {
        ConsensusState::NotInitialized
    }
}

impl ConsensusState {
    pub fn validate(&mut self, next: ConsensusState) {
        let is_valid = match (*self, next) {
            (ConsensusState::NotInitialized, ConsensusState::InitChain) => true,
            (ConsensusState::Initialized, ConsensusState::BeginBlock) => true,
            (ConsensusState::InitChain, ConsensusState::BeginBlock) => true,
            (ConsensusState::BeginBlock, ConsensusState::DeliverTx) => true,
            (ConsensusState::BeginBlock, ConsensusState::EndBlock) => true,
            (ConsensusState::DeliverTx, ConsensusState::DeliverTx) => true,
            (ConsensusState::DeliverTx, ConsensusState::EndBlock) => true,
            (ConsensusState::EndBlock, ConsensusState::Commit) => true,
            (ConsensusState::Commit, ConsensusState::BeginBlock) => true,
            _ => false,
        };

        if is_valid {
            *self = next;
        } else {
            panic!("{:?} cannot be called after {:?}", next, self);
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
