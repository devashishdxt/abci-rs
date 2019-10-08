use crate::error::Result;
use crate::types::*;

pub trait Application: Send + Sync {
    /// Echo a string to test abci client/server implementation.
    fn echo(&self, message: String) -> String {
        message
    }

    /// Signals that messages queued on the client should be flushed to the server.
    fn flush(&self) {}

    /// Return information about the application state.
    fn info(&self, info_request: InfoRequest) -> InfoResponse {
        Default::default()
    }

    /// Set non-consensus critical application specific options.
    fn set_option(&self, set_option_request: SetOptionRequest) -> Result<SetOptionResponse> {
        Ok(Default::default())
    }

    /// Called once upon genesis. Usually used to establish initial (genesis) state.
    fn init_chain(&self, init_chain_request: InitChainRequest) -> InitChainResponse {
        Default::default()
    }

    /// Query for data from the application at current or past height.
    fn query(&self, query_request: QueryRequest) -> Result<QueryResponse> {
        Ok(Default::default())
    }

    /// Signals the beginning of a new block. Called prior to any `deliver_tx`s.
    fn begin_block(&self, begin_block_request: BeginBlockRequest) -> BeginBlockResponse;

    /// Guardian of the mempool: every node runs CheckTx before letting a transaction into its local mempool.
    /// Technically optional - not involved in processing blocks
    fn check_tx(&self, check_tx_request: CheckTxRequest) -> Result<CheckTxResponse> {
        Ok(Default::default())
    }

    /// Execute the transaction in full. The workhorse of the application.
    fn deliver_tx(&self, deliver_tx_request: DeliverTxRequest) -> Result<DeliverTxResponse>;

    /// Signals the end of a block. Called after all transactions, prior to each Commit.
    fn end_block(&self, end_block_request: EndBlockRequest) -> EndBlockResponse;

    /// Persist the application state.
    fn commit(&self) -> CommitResponse;
}
