#![deny(missing_docs, unsafe_code, unstable_features)]
//! A Rust crate for creating ABCI applications.
//!
//! ## ABCI Overview
//!
//! ABCI is the interface between Tendermint (a state-machine replication engine) and your application (the actual state
//! machine). It consists of a set of methods, where each method has a corresponding `Request` and `Response` message type.
//! Tendermint calls the ABCI methods on the ABCI application by sending the `Request` messages and receiving the `Response`
//! messages in return.
//!
//! ABCI methods are split across 3 separate ABCI connections:
//!
//! - `Consensus` Connection: `InitChain`, `BeginBlock`, `DeliverTx`, `EndBlock`, `Commit`
//! - `Mempool` Connection: `CheckTx`
//! - `Info` Connection: `Info`, `SetOption`, `Query`
//!
//! Additionally, there is a `Flush` method that is called on every connection, and an `Echo` method that is just for
//! debugging.
//!
//! To know more about ABCI protocol specifications, go to official ABCI [documentation](https://tendermint.com/docs/spec/abci/).
//!
//! ## Usage
//!
//! Add `abci-rs` in your `Cargo.toml`'s `dependencies` section:
//!
//! ```toml
//! [dependencies]
//! abci-rs = "0.1"
//! ```
//!
//! Each ABCI application has to implement three core traits corresponding to all three ABCI connections, `Consensus`,
//! `Mempool` and `Info`.
//!
//! > Note: Implementations of these traits are expected to be `Send + Sync` and methods take immutable reference of `self`.
//! So, internal mutability must be handled using thread safe (`Arc`, `Mutex`, etc.) constructs.
//!
//! After implementing all three above mentioned `trait`s, you can create a `Server` object and use `server.start()`
//! function to start ABCI application.
//!
//! To know more, go to `examples/counter.rs` to see a sample ABCI application.
//!
//! ### Features
//!
//! - `uds`: Enables support for running ABCI server over Unix Domain Socket (UDS)
//!   - Supported on **Unix** only.
//!   - **Disabled** by default.
//! - `sync`: Enables ABCI Server with synchronous IO
//!   - **Enabled** by default.
//!
//! ## Supported Versions
//!
//! - Tendermint v0.32.0
//! - ABCI v0.16.0
mod application;
mod error;
mod proto;
#[cfg(feature = "sync")]
mod sync_server;

pub mod types;

pub use self::application::{Consensus, Info, Mempool};
pub use self::error::{Error, Result};
#[cfg(feature = "sync")]
pub use self::sync_server::SyncServer;
#[cfg(feature = "utils")]
pub use self::utils::Address;

#[cfg(feature = "utils")]
mod utils {
    use std::net::SocketAddr;
    #[cfg(all(unix, feature = "uds"))]
    use std::path::PathBuf;

    /// Address of ABCI Server
    pub enum Address {
        /// TCP Address
        Tcp(SocketAddr),
        /// UDS Address
        ///
        /// ### Platform support
        ///
        /// This is supported on **Unix** only.
        #[cfg(all(unix, feature = "uds"))]
        Uds(PathBuf),
    }

    impl From<SocketAddr> for Address {
        fn from(addr: SocketAddr) -> Self {
            Self::Tcp(addr)
        }
    }

    #[cfg(all(unix, feature = "uds"))]
    impl From<PathBuf> for Address {
        fn from(path: PathBuf) -> Self {
            Self::Uds(path)
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum ConsensusState {
        InitChain,
        BeginBlock,
        DeliverTx,
        EndBlock,
        Commit,
    }

    impl Default for ConsensusState {
        #[inline]
        fn default() -> Self {
            ConsensusState::InitChain
        }
    }

    impl ConsensusState {
        pub fn validate(&mut self, mut next: ConsensusState) {
            let is_valid = match (&self, next) {
                (ConsensusState::InitChain, ConsensusState::InitChain) => true,
                (ConsensusState::InitChain, ConsensusState::BeginBlock) => true,
                (ConsensusState::BeginBlock, ConsensusState::DeliverTx) => true,
                (ConsensusState::BeginBlock, ConsensusState::EndBlock) => true,
                (ConsensusState::DeliverTx, ConsensusState::DeliverTx) => true,
                (ConsensusState::DeliverTx, ConsensusState::EndBlock) => true,
                (ConsensusState::EndBlock, ConsensusState::Commit) => true,
                (ConsensusState::Commit, ConsensusState::BeginBlock) => true,
                _ => false,
            };

            if is_valid {
                std::mem::swap(self, &mut next);
            } else {
                panic!("{:?} cannot be called after {:?}", next, self);
            }
        }
    }
}
