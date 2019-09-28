use std::time::Duration;

use protobuf::well_known_types::Timestamp;

use crate::proto::abci::{
    RequestInitChain, ResponseInitChain, ValidatorUpdate as ProtoValidatorUpdate,
};
use crate::types::{ConsensusParams, ValidatorUpdate};

#[derive(Debug, Default)]
pub struct InitChainRequest {
    /// Genesis time (duration since epoch)
    pub time: Option<Duration>,
    /// ID of blockchain
    pub chain_id: String,
    /// Initial consensus-critical parameters
    pub consensus_params: Option<ConsensusParams>,
    /// Initial genesis validators
    pub validators: Vec<ValidatorUpdate>,
    /// Serialized initial application state (amino-encoded JSON bytes)
    pub app_state_bytes: Vec<u8>,
}

impl From<InitChainRequest> for RequestInitChain {
    fn from(init_chain_request: InitChainRequest) -> RequestInitChain {
        let mut request_init_chain = RequestInitChain::new();
        request_init_chain.time = init_chain_request
            .time
            .map(|time| {
                let mut timestamp = Timestamp::new();
                timestamp.seconds = time.as_secs() as i64;
                timestamp.nanos = time.subsec_nanos() as i32;
                timestamp
            })
            .into();
        request_init_chain.chain_id = init_chain_request.chain_id;
        request_init_chain.consensus_params =
            init_chain_request.consensus_params.map(Into::into).into();
        request_init_chain.validators = init_chain_request
            .validators
            .into_iter()
            .map(Into::into)
            .collect::<Vec<ProtoValidatorUpdate>>()
            .into();

        request_init_chain
    }
}

#[derive(Debug, Default)]
pub struct InitChainResponse {
    /// Initial consensus-critical parameters
    pub consensus_params: Option<ConsensusParams>,
    /// Initial validator set (if non empty)
    pub validators: Vec<ValidatorUpdate>,
}

impl From<InitChainResponse> for ResponseInitChain {
    fn from(init_chain_response: InitChainResponse) -> ResponseInitChain {
        let mut response_init_chain = ResponseInitChain::new();
        response_init_chain.consensus_params =
            init_chain_response.consensus_params.map(Into::into).into();
        response_init_chain.validators = init_chain_response
            .validators
            .into_iter()
            .map(Into::into)
            .collect::<Vec<ProtoValidatorUpdate>>()
            .into();
        response_init_chain
    }
}
