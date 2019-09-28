use crate::error::Result;
use crate::types::*;

pub trait Application: Send + Sync {
    fn echo(&self, message: String) -> String;
    fn flush(&self);
    fn info(&self, info_request: InfoRequest) -> InfoResponse;
    fn set_option(&self, set_option_request: SetOptionRequest) -> Result<SetOptionResponse>;
    fn init_chain(&self, init_chain_request: InitChainRequest) -> InitChainResponse;
    fn query(&self, query_request: QueryRequest) -> Result<QueryResponse>;
    fn begin_block(&self, begin_block_request: BeginBlockRequest) -> BeginBlockResponse;
    fn check_tx(&self, check_tx_request: CheckTxRequest) -> Result<CheckTxResponse>;
    fn deliver_tx(&self, deliver_tx_request: DeliverTxRequest) -> Result<DeliverTxResponse>;
    fn end_block(&self, end_block_request: EndBlockRequest) -> EndBlockResponse;
    fn commit(&self) -> CommitResponse;
}
