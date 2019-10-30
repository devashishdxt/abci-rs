use std::sync::{Arc, Mutex};

#[cfg(all(unix, feature = "uds"))]
use tokio::net::unix::UnixListener;
use tokio::{codec::Framed, io, net::tcp::TcpListener, prelude::*};

use crate::{
    proto::AbciCodec,
    server::{process, ConsensusState},
    Address, Consensus, Info, Mempool, Server,
};

/// Starts server in asynchronous mode
///
/// # Note
///
/// Only works with `tokio` executor
pub async fn run_async<C, M, I, T>(server: &Server<C, M, I>, addr: T) -> io::Result<()>
where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    T: Into<Address>,
{
    let addr = addr.into();

    match addr {
        Address::Tcp(addr) => {
            let mut listener = TcpListener::bind(addr).await?;
            log::info!("Started ABCI server at {}", addr);

            loop {
                let (stream, _) = listener.accept().await?;
                handle_connection(
                    server.consensus.clone(),
                    server.mempool.clone(),
                    server.info.clone(),
                    server.consensus_state.clone(),
                    stream,
                )
                .await;
            }
        }
        #[cfg(all(unix, feature = "uds"))]
        Address::Uds(path) => {
            let mut listener = UnixListener::bind(&path)?;
            log::info!("Started ABCI server at {}", path.display());

            loop {
                let (stream, _) = listener.accept().await?;
                handle_connection(
                    server.consensus.clone(),
                    server.mempool.clone(),
                    server.info.clone(),
                    server.consensus_state.clone(),
                    stream,
                )
                .await;
            }
        }
    }
}

async fn handle_connection<C, M, I, S>(
    consensus: Arc<C>,
    mempool: Arc<M>,
    info: Arc<I>,
    consensus_state: Arc<Mutex<ConsensusState>>,
    stream: S,
) where
    C: Consensus + 'static,
    M: Mempool + 'static,
    I: Info + 'static,
    S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    tokio::spawn(async move {
        let mut framed = Framed::new(stream, AbciCodec);

        while let Some(request) = framed.next().await {
            match request {
                Ok(request) => {
                    let response = process(
                        consensus.clone(),
                        mempool.clone(),
                        info.clone(),
                        consensus_state.clone(),
                        request,
                    );

                    if let Err(err) = framed.send(response).await {
                        log::error!("Error while writing to stream: {}", err);
                    }
                }
                Err(e) => panic!("Error while receiving ABCI request from socket: {}", e),
            }
        }
    });
}
