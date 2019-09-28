use crate::error::Result;
use crate::proto::abci::{RequestSetOption, ResponseSetOption};

#[derive(Debug, Default)]
pub struct SetOptionRequest {
    /// Key to set
    pub key: String,
    /// Value to set for key
    pub value: String,
}

impl From<SetOptionRequest> for RequestSetOption {
    fn from(set_option_request: SetOptionRequest) -> RequestSetOption {
        let mut request_set_option = RequestSetOption::new();
        request_set_option.key = set_option_request.key;
        request_set_option.value = set_option_request.value;
        request_set_option
    }
}

#[derive(Debug, Default)]
pub struct SetOptionResponse {
    /// Output of application's logger (may be non-deterministic)
    pub log: String,
    /// Additional information (may be non-deterministic)
    pub info: String,
}

impl From<Result<SetOptionResponse>> for ResponseSetOption {
    fn from(set_option_response: Result<SetOptionResponse>) -> ResponseSetOption {
        let mut response_set_option = ResponseSetOption::new();

        match set_option_response {
            Ok(set_option_response) => {
                response_set_option.log = set_option_response.log;
                response_set_option.info = set_option_response.info;
            }
            Err(error) => {
                response_set_option.code = error.code;
                response_set_option.log = error.log;
                response_set_option.info = error.info;
            }
        }

        response_set_option
    }
}
