use std::{net::SocketAddr, sync::mpsc::channel};

use crate::{Consensus, Info, Mempool};

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
    C: Consensus,
    M: Mempool,
    I: Info,
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
        // let (consensus_sender, consensus_receiver) = channel();
        // let (mempool_sender, mempool_receiver) = channel();
        // let (info_sender, info_receiver) = channel();

        unimplemented!()
    }
}
