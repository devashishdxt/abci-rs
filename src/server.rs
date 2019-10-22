use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener},
    sync::{Arc, Mutex},
    thread,
};
#[cfg(all(unix, feature = "uds"))]
use std::{os::unix::net::UnixListener, path::PathBuf};

use crate::{
    proto::{abci::*, decode, encode},
    Consensus, Info, Mempool,
};

/// ABCI Server
pub struct Server<C, M, I>
where
    C: Consensus,
    M: Mempool,
    I: Info,
{
    consensus: Arc<C>,
    mempool: Arc<M>,
    info: Arc<I>,
    consensus_state: Arc<Mutex<ConsensusState>>,
}

impl<C, M, I> Server<C, M, I>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
{
    /// Creates a new instance of [`Server`](struct.Server.html)
    #[inline]
    pub fn new(consensus: C, mempool: M, info: I) -> Self {
        Server {
            consensus: Arc::new(consensus),
            mempool: Arc::new(mempool),
            info: Arc::new(info),
            consensus_state: Arc::new(Mutex::new(ConsensusState::default())),
        }
    }

    /// Starts ABCI Server
    pub fn start<T: Into<Address>>(&self, addr: T) -> io::Result<()> {
        let addr = addr.into();

        match addr {
            Address::Tcp(addr) => {
                let listener = TcpListener::bind(addr)?;
                log::info!("Started ABCI server at {}", addr);

                for stream in listener.incoming() {
                    self.handle_connection(stream?);
                }
            }
            #[cfg(all(unix, feature = "uds"))]
            Address::Uds(path) => {
                let listener = UnixListener::bind(&path)?;
                log::info!("Started ABCI server at {}", path.display());

                for stream in listener.incoming() {
                    self.handle_connection(stream?);
                }
            }
        }

        Ok(())
    }

    fn handle_connection<S>(&self, mut stream: S)
    where
        S: Read + Write + Send + 'static,
    {
        let consensus = self.consensus.clone();
        let mempool = self.mempool.clone();
        let info = self.info.clone();
        let consensus_state = self.consensus_state.clone();

        thread::spawn(move || loop {
            match decode(&mut stream) {
                Ok(Some(request)) => {
                    log::trace!("Received request: {:?}", request);

                    let value = match request.value.unwrap() {
                        Request_oneof_value::echo(request) => {
                            let mut response = ResponseEcho::new();
                            response.message = info.echo(request.message);
                            Response_oneof_value::echo(response)
                        }
                        Request_oneof_value::flush(_) => {
                            consensus.flush();
                            Response_oneof_value::flush(ResponseFlush::new())
                        }
                        Request_oneof_value::info(request) => {
                            Response_oneof_value::info(info.info(request.into()).into())
                        }
                        Request_oneof_value::set_option(request) => {
                            Response_oneof_value::set_option(info.set_option(request.into()).into())
                        }
                        Request_oneof_value::init_chain(request) => {
                            consensus_state
                                .lock()
                                .unwrap()
                                .validate(ConsensusState::InitChain);
                            Response_oneof_value::init_chain(
                                consensus.init_chain(request.into()).into(),
                            )
                        }
                        Request_oneof_value::query(request) => {
                            Response_oneof_value::query(info.query(request.into()).into())
                        }
                        Request_oneof_value::begin_block(request) => {
                            consensus_state
                                .lock()
                                .unwrap()
                                .validate(ConsensusState::BeginBlock);
                            Response_oneof_value::begin_block(
                                consensus.begin_block(request.into()).into(),
                            )
                        }
                        Request_oneof_value::check_tx(request) => {
                            Response_oneof_value::check_tx(mempool.check_tx(request.into()).into())
                        }
                        Request_oneof_value::deliver_tx(request) => {
                            consensus_state
                                .lock()
                                .unwrap()
                                .validate(ConsensusState::DeliverTx);
                            Response_oneof_value::deliver_tx(
                                consensus.deliver_tx(request.into()).into(),
                            )
                        }
                        Request_oneof_value::end_block(request) => {
                            consensus_state
                                .lock()
                                .unwrap()
                                .validate(ConsensusState::EndBlock);
                            Response_oneof_value::end_block(
                                consensus.end_block(request.into()).into(),
                            )
                        }
                        Request_oneof_value::commit(_) => {
                            consensus_state
                                .lock()
                                .unwrap()
                                .validate(ConsensusState::Commit);
                            Response_oneof_value::commit(consensus.commit().into())
                        }
                    };

                    respond(&mut stream, value);
                }
                Ok(None) => log::trace!("Received empty request"),
                Err(e) => panic!("Error while receiving ABCI request from socket: {}", e),
            }
        });
    }
}

fn respond<W: Write>(writer: W, value: Response_oneof_value) {
    let mut response = Response::new();
    response.value = Some(value);

    log::trace!("Sending response: {:?}", response);

    if let Err(err) = encode(response, writer) {
        log::error!("Error while writing to stream: {}", err);
    }
}

/// Address of ABCI Server
pub enum Address {
    /// TCP Address
    Tcp(SocketAddr),
    /// UDS Address
    ///
    /// ### Platform support
    ///
    /// This is supported on **Unix** only.
    #[cfg(all(unix, feature = "uds"))]
    Uds(PathBuf),
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Self {
        Self::Tcp(addr)
    }
}

#[cfg(all(unix, feature = "uds"))]
impl From<PathBuf> for Address {
    fn from(path: PathBuf) -> Self {
        Self::Uds(path)
    }
}

#[derive(Debug, Clone, Copy)]
enum ConsensusState {
    InitChain,
    BeginBlock,
    DeliverTx,
    EndBlock,
    Commit,
}

impl Default for ConsensusState {
    #[inline]
    fn default() -> Self {
        ConsensusState::InitChain
    }
}

impl ConsensusState {
    fn validate(&mut self, mut next: ConsensusState) {
        let is_valid = match (&self, next) {
            (ConsensusState::InitChain, ConsensusState::InitChain) => true,
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
            std::mem::swap(self, &mut next);
        } else {
            panic!("{:?} cannot be called after {:?}", next, self);
        }
    }
}
