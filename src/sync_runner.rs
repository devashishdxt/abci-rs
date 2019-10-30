#[cfg(all(unix, feature = "uds"))]
use std::os::unix::net::UnixListener;
use std::{
    io::{self, Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    proto::{abci::*, decode_sync, encode_sync},
    server::{process, ConsensusState},
    Address, Consensus, Info, Mempool, Server,
};

/// Starts server in synchronous mode
pub fn run_sync<C, M, I, T>(server: &Server<C, M, I>, addr: T) -> io::Result<()>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    T: Into<Address>,
{
    let addr = addr.into();

    match addr {
        Address::Tcp(addr) => {
            let listener = TcpListener::bind(addr)?;
            log::info!("Started ABCI server at {}", addr);

            for stream in listener.incoming() {
                handle_connection(
                    server.consensus.clone(),
                    server.mempool.clone(),
                    server.info.clone(),
                    server.consensus_state.clone(),
                    stream?,
                );
            }
        }
        #[cfg(all(unix, feature = "uds"))]
        Address::Uds(path) => {
            let listener = UnixListener::bind(&path)?;
            log::info!("Started ABCI server at {}", path.display());

            for stream in listener.incoming() {
                handle_connection(
                    server.consensus.clone(),
                    server.mempool.clone(),
                    server.info.clone(),
                    server.consensus_state.clone(),
                    stream?,
                );
            }
        }
    }

    Ok(())
}

fn handle_connection<C, M, I, S>(
    consensus: Arc<C>,
    mempool: Arc<M>,
    info: Arc<I>,
    consensus_state: Arc<Mutex<ConsensusState>>,
    mut stream: S,
) where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: Read + Write + Send + 'static,
{
    thread::spawn(move || loop {
        match decode_sync(&mut stream) {
            Ok(Some(request)) => {
                let response = process(
                    consensus.clone(),
                    mempool.clone(),
                    info.clone(),
                    consensus_state.clone(),
                    request,
                );
                respond(&mut stream, response);
            }
            Ok(None) => log::trace!("Received empty request"),
            Err(e) => panic!("Error while receiving ABCI request from socket: {}", e),
        }
    });
}

fn respond<W: Write>(writer: W, response: Response) {
    log::trace!("Sending response: {:?}", response);

    if let Err(err) = encode_sync(response, writer) {
        log::error!("Error while writing to stream: {}", err);
    }
}
