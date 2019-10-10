use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{channel, Receiver, Sender},
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
    consensus: C,
    mempool: M,
    info: I,
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
            consensus,
            mempool,
            info,
        }
    }

    /// Start ABCI server
    pub fn run(self, address: SocketAddr) {
        let consensus = self.consensus;
        let mempool = self.mempool;
        let info = self.info;

        let (consensus_sender, consensus_receiver) = channel::<(Request, TcpStream)>();
        let (mempool_sender, mempool_receiver) = channel::<(Request, TcpStream)>();
        let (info_sender, info_receiver) = channel::<(Request, TcpStream)>();

        consensus_thread(consensus, consensus_receiver);
        mempool_thread(mempool, mempool_receiver);
        info_thread(info, info_receiver);

        run_io_loop(address, consensus_sender, mempool_sender, info_sender);
    }
}

fn run_io_loop(
    address: SocketAddr,
    consensus_sender: Sender<(Request, TcpStream)>,
    mempool_sender: Sender<(Request, TcpStream)>,
    info_sender: Sender<(Request, TcpStream)>,
) {
    loop {
        let listener = TcpListener::bind(address).unwrap();
        log::info!("Started ABCI server on {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => match decode(&stream) {
                    Ok(request) => {
                        log::trace!("Received request: {:?}", request);

                        if let Some(request_type) = <Option<RequestType>>::from(&request) {
                            if let Err(err) = match request_type {
                                RequestType::Consensus => consensus_sender.send((request, stream)),
                                RequestType::Mempool => mempool_sender.send((request, stream)),
                                RequestType::Info => info_sender.send((request, stream)),
                            } {
                                panic!(
                                    "Channel for {:?} requests is broken: {}",
                                    request_type, err
                                );
                            }
                        } else {
                            log::warn!("Empty request received");
                        }
                    }
                    Err(err) => log::error!("Unable to read request from socket: {}", err),
                },
                Err(err) => log::error!("Connection error: {}", err),
            }
        }
    }
}

fn consensus_thread<C: Consensus + 'static>(
    consensus: C,
    consensus_receiver: Receiver<(Request, TcpStream)>,
) {
    thread::spawn(move || {
        let mut consensus_state = ConsensusState::default();

        for (request, stream) in consensus_receiver.into_iter() {
            let value = match request.value.unwrap() {
                Request_oneof_value::flush(_) => {
                    consensus.flush();
                    Response_oneof_value::flush(ResponseFlush::new())
                }
                Request_oneof_value::init_chain(request) => {
                    consensus_state.validate(ConsensusState::InitChain);
                    Response_oneof_value::init_chain(consensus.init_chain(request.into()).into())
                }
                Request_oneof_value::begin_block(request) => {
                    consensus_state.validate(ConsensusState::BeginBlock);
                    Response_oneof_value::begin_block(consensus.begin_block(request.into()).into())
                }
                Request_oneof_value::deliver_tx(request) => {
                    consensus_state.validate(ConsensusState::DeliverTx);
                    Response_oneof_value::deliver_tx(consensus.deliver_tx(request.into()).into())
                }
                Request_oneof_value::end_block(request) => {
                    consensus_state.validate(ConsensusState::EndBlock);
                    Response_oneof_value::end_block(consensus.end_block(request.into()).into())
                }
                Request_oneof_value::commit(_) => {
                    consensus_state.validate(ConsensusState::Commit);
                    Response_oneof_value::commit(consensus.commit().into())
                }
                _ => unreachable!("Non-consensus request cannot be handled by consensus thread"),
            };

            respond(stream, value);
        }
    });
}

fn mempool_thread<M: Mempool + 'static>(
    mempool: M,
    mempool_receiver: Receiver<(Request, TcpStream)>,
) {
    thread::spawn(move || {
        for (request, stream) in mempool_receiver.into_iter() {
            let value = match request.value.unwrap() {
                Request_oneof_value::check_tx(request) => {
                    Response_oneof_value::check_tx(mempool.check_tx(request.into()).into())
                }
                _ => unreachable!("Non-mempool request cannot be handled by mempool thread"),
            };

            respond(stream, value);
        }
    });
}

fn info_thread<I: Info + 'static>(info: I, info_receiver: Receiver<(Request, TcpStream)>) {
    thread::spawn(move || {
        for (request, stream) in info_receiver.into_iter() {
            let value = match request.value.unwrap() {
                Request_oneof_value::echo(request) => {
                    let mut response = ResponseEcho::new();
                    response.message = info.echo(request.message);
                    Response_oneof_value::echo(response)
                }
                Request_oneof_value::info(request) => {
                    Response_oneof_value::info(info.info(request.into()).into())
                }
                Request_oneof_value::set_option(request) => {
                    Response_oneof_value::set_option(info.set_option(request.into()).into())
                }
                Request_oneof_value::query(request) => {
                    Response_oneof_value::query(info.query(request.into()).into())
                }
                _ => unreachable!("Non-info request cannot be handled by info thread"),
            };

            respond(stream, value);
        }
    });
}

fn respond(stream: TcpStream, value: Response_oneof_value) {
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
    fn is_valid_next(self, next: ConsensusState) -> bool {
        match (self, next) {
            (ConsensusState::InitChain, ConsensusState::InitChain) => true,
            (ConsensusState::InitChain, ConsensusState::BeginBlock) => true,
            (ConsensusState::BeginBlock, ConsensusState::DeliverTx) => true,
            (ConsensusState::DeliverTx, ConsensusState::DeliverTx) => true,
            (ConsensusState::DeliverTx, ConsensusState::EndBlock) => true,
            (ConsensusState::EndBlock, ConsensusState::Commit) => true,
            (ConsensusState::Commit, ConsensusState::BeginBlock) => true,
            _ => false,
        }
    }

    fn validate(&mut self, mut next: ConsensusState) {
        if self.is_valid_next(next) {
            std::mem::swap(self, &mut next);
        } else {
            panic!("{:?} cannot be called after {:?}", next, self);
        }
    }
}
