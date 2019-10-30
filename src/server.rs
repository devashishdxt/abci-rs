#[cfg(all(unix, feature = "uds"))]
use std::path::PathBuf;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use crate::{proto::abci::*, Consensus, Info, Mempool};

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
    #[inline]
    pub fn new(consensus: C, mempool: M, info: I) -> Self {
        Self {
            consensus: Arc::new(consensus),
            mempool: Arc::new(mempool),
            info: Arc::new(info),
            consensus_state: Arc::new(Mutex::new(ConsensusState::default())),
        }
    }
}

pub fn process<C, M, I>(
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
            Response_oneof_value::init_chain(consensus.init_chain(request.into()).into())
        }
        Request_oneof_value::query(request) => {
            Response_oneof_value::query(info.query(request.into()).into())
        }
        Request_oneof_value::begin_block(request) => {
            consensus_state
                .lock()
                .unwrap()
                .validate(ConsensusState::BeginBlock);
            Response_oneof_value::begin_block(consensus.begin_block(request.into()).into())
        }
        Request_oneof_value::check_tx(request) => {
            Response_oneof_value::check_tx(mempool.check_tx(request.into()).into())
        }
        Request_oneof_value::deliver_tx(request) => {
            consensus_state
                .lock()
                .unwrap()
                .validate(ConsensusState::DeliverTx);
            Response_oneof_value::deliver_tx(consensus.deliver_tx(request.into()).into())
        }
        Request_oneof_value::end_block(request) => {
            consensus_state
                .lock()
                .unwrap()
                .validate(ConsensusState::EndBlock);
            Response_oneof_value::end_block(consensus.end_block(request.into()).into())
        }
        Request_oneof_value::commit(_) => {
            consensus_state
                .lock()
                .unwrap()
                .validate(ConsensusState::Commit);
            Response_oneof_value::commit(consensus.commit().into())
        }
    };

    let mut response = Response::new();
    response.value = Some(value);

    response
}

#[derive(Debug, Clone, Copy)]
pub enum ConsensusState {
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
    pub fn validate(&mut self, mut next: ConsensusState) {
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
