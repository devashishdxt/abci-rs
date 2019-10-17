use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    thread,
};

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
        }
    }

    /// Starts ABCI Server
    pub fn start(&self, addr: SocketAddr) -> io::Result<()> {
        let listener = TcpListener::bind(addr)?;
        log::info!("Started ABCI server at {}", addr);

        for stream in listener.incoming() {
            self.handle_connection(stream?);
        }

        Ok(())
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let consensus = self.consensus.clone();
        let mempool = self.mempool.clone();
        let info = self.info.clone();

        thread::spawn(move || {
            let mut consensus_state = ConsensusState::default();

            loop {
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
                                Response_oneof_value::set_option(
                                    info.set_option(request.into()).into(),
                                )
                            }
                            Request_oneof_value::init_chain(request) => {
                                consensus_state.validate(ConsensusState::InitChain);
                                Response_oneof_value::init_chain(
                                    consensus.init_chain(request.into()).into(),
                                )
                            }
                            Request_oneof_value::query(request) => {
                                Response_oneof_value::query(info.query(request.into()).into())
                            }
                            Request_oneof_value::begin_block(request) => {
                                consensus_state.validate(ConsensusState::BeginBlock);
                                Response_oneof_value::begin_block(
                                    consensus.begin_block(request.into()).into(),
                                )
                            }
                            Request_oneof_value::check_tx(request) => {
                                Response_oneof_value::check_tx(
                                    mempool.check_tx(request.into()).into(),
                                )
                            }
                            Request_oneof_value::deliver_tx(request) => {
                                consensus_state.validate(ConsensusState::DeliverTx);
                                Response_oneof_value::deliver_tx(
                                    consensus.deliver_tx(request.into()).into(),
                                )
                            }
                            Request_oneof_value::end_block(request) => {
                                consensus_state.validate(ConsensusState::EndBlock);
                                Response_oneof_value::end_block(
                                    consensus.end_block(request.into()).into(),
                                )
                            }
                            Request_oneof_value::commit(_) => {
                                consensus_state.validate(ConsensusState::Commit);
                                Response_oneof_value::commit(consensus.commit().into())
                            }
                        };

                        respond(&mut stream, value);
                    }
                    Ok(None) => continue,
                    Err(e) => panic!("Error while receiving ABCI request from socket: {}", e),
                }
            }
        });
    }
}

fn respond(stream: &mut TcpStream, value: Response_oneof_value) {
    let mut response = Response::new();
    response.value = Some(value);

    log::trace!("Sending response: {:?}", response);

    if let Err(err) = encode(response, stream) {
        log::error!("Error while writing to stream: {}", err);
    }
}

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
