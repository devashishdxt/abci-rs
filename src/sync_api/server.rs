use std::io::Result;

use crate::{
    async_api::Server as AsyncServer,
    sync_api::{
        async_impls::{AsyncConsensusImpl, AsyncInfoImpl, AsyncMempoolImpl, AsyncSnapshotImpl},
        Consensus, Info, Mempool, Snapshot,
    },
    Address,
};

/// ABCI Server
pub struct Server<C, M, I, S>
where
    C: Consensus + Send + Sync + 'static,
    M: Mempool + Send + Sync + 'static,
    I: Info + Send + Sync + 'static,
    S: Snapshot + Send + Sync + 'static,
{
    async_server: AsyncServer<
        AsyncConsensusImpl<C>,
        AsyncMempoolImpl<M>,
        AsyncInfoImpl<I>,
        AsyncSnapshotImpl<S>,
    >,
}

impl<C, M, I, S> Server<C, M, I, S>
where
    C: Consensus + Send + Sync + 'static,
    M: Mempool + Send + Sync + 'static,
    I: Info + Send + Sync + 'static,
    S: Snapshot + Send + Sync + 'static,
{
    /// Creates a new instance of [`Server`](self::Server)
    pub fn new(consensus: C, mempool: M, info: I, snapshot: S) -> Self {
        Self {
            async_server: AsyncServer::new(
                AsyncConsensusImpl::new(consensus),
                AsyncMempoolImpl::new(mempool),
                AsyncInfoImpl::new(info),
                AsyncSnapshotImpl::new(snapshot),
            ),
        }
    }

    /// Starts ABCI server
    pub fn run<T>(&self, addr: T) -> Result<()>
    where
        T: Into<Address>,
    {
        cfg_if::cfg_if! {
            if #[cfg(feature = "use-async-std")] {
                async_std::task::block_on(async { self.async_server.run(addr).await })
            } else if #[cfg(feature = "use-smol")] {
                smol::block_on(async { self.async_server.run(addr).await })
            } else if #[cfg(feature = "use-tokio")] {
                let runtime = tokio::runtime::Runtime::new()?;
                runtime.block_on(async { self.async_server.run(addr).await })
            } else {
                unreachable!()
            }
        }
    }
}
