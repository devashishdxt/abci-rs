use std::sync::Arc;

#[cfg(feature = "use-async-std")]
use async_std::task::spawn_blocking;
use async_trait::async_trait;
#[cfg(feature = "use-smol")]
use smol::unblock as spawn_blocking;
#[cfg(feature = "use-tokio")]
use tokio::task::spawn_blocking;

use crate::{
    async_api::{
        Consensus as AsyncConsensus, Info as AsyncInfo, Mempool as AsyncMempool,
        Snapshot as AsyncSnapshot,
    },
    sync_api::{Consensus, Info, Mempool, Snapshot},
    types::*,
};

macro_rules! spawn_blocking {
    ($expr: expr) => {{
        cfg_if::cfg_if! {
            if #[cfg(any(feature = "use-async-std", feature = "use-smol"))] {
                spawn_blocking($expr).await
            } else if #[cfg(feature = "use-tokio")] {
                spawn_blocking($expr).await.expect("Failed to execute blocking task")
            }
        }
    }};
}

pub struct AsyncConsensusImpl<C>
where
    C: Consensus + Send + Sync + 'static,
{
    inner: Arc<C>,
}

impl<C> AsyncConsensusImpl<C>
where
    C: Consensus + Send + Sync,
{
    pub fn new(inner: C) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

#[async_trait]
impl<C> AsyncConsensus for AsyncConsensusImpl<C>
where
    C: Consensus + Send + Sync + 'static,
{
    async fn echo(&self, echo_request: RequestEcho) -> ResponseEcho {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.echo(echo_request))
    }

    async fn init_chain(&self, init_chain_request: RequestInitChain) -> ResponseInitChain {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.init_chain(init_chain_request))
    }

    async fn begin_block(&self, begin_block_request: RequestBeginBlock) -> ResponseBeginBlock {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.begin_block(begin_block_request))
    }

    async fn deliver_tx(&self, deliver_tx_request: RequestDeliverTx) -> ResponseDeliverTx {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.deliver_tx(deliver_tx_request))
    }

    async fn end_block(&self, end_block_request: RequestEndBlock) -> ResponseEndBlock {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.end_block(end_block_request))
    }

    async fn commit(&self, commit_request: RequestCommit) -> ResponseCommit {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.commit(commit_request))
    }

    async fn flush(&self, flush_request: RequestFlush) -> ResponseFlush {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.flush(flush_request))
    }
}

pub struct AsyncInfoImpl<I>
where
    I: Info + Send + Sync + 'static,
{
    inner: Arc<I>,
}

impl<I> AsyncInfoImpl<I>
where
    I: Info + Send + Sync,
{
    pub fn new(inner: I) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

#[async_trait]
impl<I> AsyncInfo for AsyncInfoImpl<I>
where
    I: Info + Send + Sync + 'static,
{
    async fn echo(&self, echo_request: RequestEcho) -> ResponseEcho {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.echo(echo_request))
    }

    async fn info(&self, info_request: RequestInfo) -> ResponseInfo {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.info(info_request))
    }

    async fn set_option(&self, set_option_request: RequestSetOption) -> ResponseSetOption {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.set_option(set_option_request))
    }

    async fn query(&self, query_request: RequestQuery) -> ResponseQuery {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.query(query_request))
    }

    async fn flush(&self, flush_request: RequestFlush) -> ResponseFlush {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.flush(flush_request))
    }
}

pub struct AsyncMempoolImpl<M>
where
    M: Mempool + Send + Sync + 'static,
{
    inner: Arc<M>,
}

impl<M> AsyncMempoolImpl<M>
where
    M: Mempool + Send + Sync,
{
    pub fn new(inner: M) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

#[async_trait]
impl<M> AsyncMempool for AsyncMempoolImpl<M>
where
    M: Mempool + Send + Sync + 'static,
{
    async fn echo(&self, echo_request: RequestEcho) -> ResponseEcho {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.echo(echo_request))
    }

    async fn check_tx(&self, check_tx_request: RequestCheckTx) -> ResponseCheckTx {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.check_tx(check_tx_request))
    }

    async fn flush(&self, flush_request: RequestFlush) -> ResponseFlush {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.flush(flush_request))
    }
}

pub struct AsyncSnapshotImpl<S>
where
    S: Snapshot + Send + Sync + 'static,
{
    inner: Arc<S>,
}

impl<S> AsyncSnapshotImpl<S>
where
    S: Snapshot + Send + Sync,
{
    pub fn new(inner: S) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

#[async_trait]
impl<S> AsyncSnapshot for AsyncSnapshotImpl<S>
where
    S: Snapshot + Send + Sync + 'static,
{
    async fn echo(&self, echo_request: RequestEcho) -> ResponseEcho {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.echo(echo_request))
    }

    async fn list_snapshots(
        &self,
        list_snapshots_request: RequestListSnapshots,
    ) -> ResponseListSnapshots {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.list_snapshots(list_snapshots_request))
    }

    async fn offer_snapshot(
        &self,
        offer_snapshot_request: RequestOfferSnapshot,
    ) -> ResponseOfferSnapshot {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.offer_snapshot(offer_snapshot_request))
    }

    async fn load_snapshot_chunk(
        &self,
        load_snapshot_chunk_request: RequestLoadSnapshotChunk,
    ) -> ResponseLoadSnapshotChunk {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.load_snapshot_chunk(load_snapshot_chunk_request))
    }

    async fn apply_snapshot_chunk(
        &self,
        apply_snapshot_chunk_request: RequestApplySnapshotChunk,
    ) -> ResponseApplySnapshotChunk {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.apply_snapshot_chunk(apply_snapshot_chunk_request))
    }

    async fn flush(&self, flush_request: RequestFlush) -> ResponseFlush {
        let inner = self.inner.clone();
        spawn_blocking!(move || inner.flush(flush_request))
    }
}
