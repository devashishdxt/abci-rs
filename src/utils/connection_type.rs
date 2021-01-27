use tendermint_proto::abci::request::Value as RequestValue;

/// Different types of connections created by tendermint
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConnectionType {
    Unknown,
    Consensus,
    Mempool,
    Info,
    Snapshot,
}

impl Default for ConnectionType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<&RequestValue> for ConnectionType {
    fn from(request_value: &RequestValue) -> Self {
        match request_value {
            RequestValue::Echo(_) | RequestValue::Flush(_) => Self::Unknown,
            RequestValue::InitChain(_)
            | RequestValue::BeginBlock(_)
            | RequestValue::DeliverTx(_)
            | RequestValue::EndBlock(_)
            | RequestValue::Commit(_) => Self::Consensus,
            RequestValue::CheckTx(_) => Self::Mempool,
            RequestValue::Info(_) | RequestValue::SetOption(_) | RequestValue::Query(_) => {
                Self::Info
            }
            RequestValue::ListSnapshots(_)
            | RequestValue::OfferSnapshot(_)
            | RequestValue::LoadSnapshotChunk(_)
            | RequestValue::ApplySnapshotChunk(_) => Self::Snapshot,
        }
    }
}
