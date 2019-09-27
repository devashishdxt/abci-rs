use crate::types::*;

pub trait Application: Send + Sync {
    fn echo(&self, message: String) -> String;
    fn flush(&self);
    fn info(&self, info_request: InfoRequest) -> InfoResponse;
}
