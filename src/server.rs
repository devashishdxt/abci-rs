use std::{net::TcpStream, sync::Arc};

use crate::{
    proto::{abci::*, encode},
    Consensus, Info, Mempool,
};

/// ABCI Server
#[allow(dead_code)]
pub struct Server<C, M, I>
where
    C: Consensus,
    M: Mempool,
    I: Info,
{
    consensus: Arc<C>,
    mempool: Arc<M>,
    info: Arc<I>,
}

impl<C, M, I> Server<C, M, I>
where
    C: Consensus,
    M: Mempool,
    I: Info,
{
    /// Creates a new instance of [`Server`](struct.Server.html)
    #[inline]
    pub fn new(consensus: C, mempool: M, info: I) -> Self {
        Server {
            consensus: Arc::new(consensus),
            mempool: Arc::new(mempool),
            info: Arc::new(info),
        }
    }
}

#[allow(dead_code)]
fn respond(stream: &mut TcpStream, value: Response_oneof_value) {
    let mut response = Response::new();
    response.value = Some(value);

    log::trace!("Sending response: {:?}", response);

    if let Err(err) = encode(response, stream) {
        log::error!("Error while writing to stream: {}", err);
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum RequestType {
    Consensus,
    Mempool,
    Info,
}

impl From<&Request> for Option<RequestType> {
    fn from(request: &Request) -> Option<RequestType> {
        let request = request.value.as_ref()?;

        match request {
            Request_oneof_value::echo(_) => Some(RequestType::Info),
            Request_oneof_value::flush(_) => Some(RequestType::Consensus),
            Request_oneof_value::info(_) => Some(RequestType::Info),
            Request_oneof_value::set_option(_) => Some(RequestType::Info),
            Request_oneof_value::init_chain(_) => Some(RequestType::Consensus),
            Request_oneof_value::query(_) => Some(RequestType::Info),
            Request_oneof_value::begin_block(_) => Some(RequestType::Consensus),
            Request_oneof_value::check_tx(_) => Some(RequestType::Mempool),
            Request_oneof_value::deliver_tx(_) => Some(RequestType::Consensus),
            Request_oneof_value::end_block(_) => Some(RequestType::Consensus),
            Request_oneof_value::commit(_) => Some(RequestType::Consensus),
        }
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
impl ConsensusState {
    fn validate(&mut self, mut next: ConsensusState) {
        let is_valid = match (&self, next) {
            (ConsensusState::InitChain, ConsensusState::InitChain) => true,
            (ConsensusState::InitChain, ConsensusState::BeginBlock) => true,
            (ConsensusState::BeginBlock, ConsensusState::DeliverTx) => true,
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
