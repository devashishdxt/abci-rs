use crate::error::Result;
use crate::types::*;

/// Trait for initialization and for queries from the user.
pub trait Info: Send + Sync {
    /// Echo a string to test abci client/server implementation.
    fn echo(&self, message: String) -> String {
        message
    }

    /// Return information about the application state.
    fn info(&self, _info_request: InfoRequest) -> InfoResponse {
        Default::default()
    }

    /// Set non-consensus critical application specific options.
    fn set_option(&self, _set_option_request: SetOptionRequest) -> Result<SetOptionResponse> {
        Ok(Default::default())
    }

    /// Query for data from the application at current or past height.
    fn query(&self, _query_request: QueryRequest) -> Result<QueryResponse> {
        Ok(Default::default())
    }
}

/// Trait for managing consensus of blockchain.
///
/// # Details
///
/// [_Consensus_] should maintain a `consensus_state` - the working state for block execution. It should be updated by
/// the calls to [`begin_block`], [`deliver_tx`], and [`end_block`] during block execution and committed to disk as the
/// **latest committed state** during [`commit`].
///
/// Updates made to the `consensus_state` by each method call must be readable by each subsequent method - ie. the
/// updates are linearizable.
///
/// [_Consensus_]: trait.Consensus.html#details
/// [`begin_block`]: trait.Consensus.html#tymethod.begin_block
/// [`deliver_tx`]: trait.Consensus.html#tymethod.deliver_tx
/// [`end_block`]: trait.Consensus.html#tymethod.end_block
/// [`commit`]: trait.Consensus.html#tymethod.commit
pub trait Consensus: Send + Sync {
    /// Called once upon genesis. Usually used to establish initial (genesis) state.
    fn init_chain(&self, init_chain_request: InitChainRequest) -> InitChainResponse;

    /// Signals the beginning of a new block. Called prior to any [`deliver_tx`](trait.Consensus.html#tymethod.deliver_tx) s.
    fn begin_block(&self, begin_block_request: BeginBlockRequest) -> BeginBlockResponse;

    /// Execute the transaction in full. The workhorse of the application.
    fn deliver_tx(&self, deliver_tx_request: DeliverTxRequest) -> Result<DeliverTxResponse>;

    /// Signals the end of a block. Called after all transactions, prior to each [`commit`](trait.Commit.html#tymethod.commit).
    fn end_block(&self, end_block_request: EndBlockRequest) -> EndBlockResponse;

    /// Persist the application state.
    ///
    /// # Details
    ///
    /// Application state should only be persisted to disk during [`commit`].
    ///
    /// Before [`commit`] is called, Tendermint locks and flushes the mempool so that no new messages will be received
    /// on the mempool connection. This provides an opportunity to safely update all three states ([_Consensus_],
    /// [_Mempool_] and [_Info_]) to the **latest committed state** at once.
    ///
    /// When [`commit`] completes, it unlocks the mempool.
    ///
    /// # Warning
    ///
    /// If the ABCI application logic processing the [`commit`] message sends a `/broadcast_tx_sync` or
    /// `/broadcast_tx_commit` and waits for the response before proceeding, it will deadlock. Executing those
    /// `broadcast_tx` calls involves acquiring a lock that is held during the [`commit`] call, so it's not possible. If
    /// you make the call to the `broadcast_tx` endpoints concurrently, that's no problem, it just can't be part of the
    /// sequential logic of the [`commit`] function.
    ///
    /// [`commit`]: trait.Commit.html#tymethod.commit
    /// [_Consensus_]: trait.Consensus.html#details
    /// [_Mempool_]: trait.Mempool.html#details
    /// [_Info_]: trait.Info.html
    fn commit(&self) -> CommitResponse;
}

/// Trait for managing tendermint's mempool.
///
/// # Details
///
/// [_Mempool_] should maintain a `mempool_state` to sequentially process pending transactions in the mempool that have
/// not yet been committed. It should be initialized to the latest committed state at the end of every [`commit`].
///
/// The `mempool_state` may be updated concurrently with the `consensus_state`, as messages may be sent concurrently on
/// [_Consensus_] and [_Mempool_] connections. However, before calling [`commit`], Tendermint will lock and flush the
/// mempool connection, ensuring that all existing [`check_tx`] are responded to and no new ones can begin.
///
/// After [`commit`], [`check_tx`] is run again on all transactions that remain in the node's local mempool after
/// filtering those included in the block. To prevent the mempool from rechecking all transactions every time a block is
/// committed, set the configuration option `mempool.recheck=false`.
///
/// Finally, the mempool will unlock and new transactions can be processed through [`check_tx`] again.
///
/// Note that [`check_tx`] doesn't have to check everything that affects transaction validity; the expensive things can
/// be skipped. In fact, [`check_tx`] doesn't have to check anything; it might say that any transaction is a valid
/// transaction. Unlike [`deliver_tx`], [`check_tx`] is just there as a sort of weak filter to keep invalid transactions
/// out of the blockchain. It's weak, because a Byzantine node doesn't care about [`check_tx`]; it can propose a block
/// full of invalid transactions if it wants.
///
/// [_Mempool_]: trait.Mempool.html#details
/// [`commit`]: trait.Consensus.html#tymethod.commit
/// [_Consensus_]: trait.Consensus.html#details
/// [`deliver_tx`]: trait.Consensus.html#tymethod.deliver_tx
/// [`check_tx`]: trait.Mempool.html#method.check_tx
pub trait Mempool: Send + Sync {
    /// Guardian of the mempool: every node runs CheckTx before letting a transaction into its local mempool.
    /// Technically optional - not involved in processing blocks
    fn check_tx(&self, _check_tx_request: CheckTxRequest) -> Result<CheckTxResponse> {
        Ok(Default::default())
    }
}
