#![deny(missing_docs, unsafe_code, unstable_features)]
//! Implementation of Tendermint ABCI protocol.
mod application;
mod error;
mod proto;

pub mod types;

pub use application::{Consensus, Info, Mempool};
pub use error::{Error, Result};
