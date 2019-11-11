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
//! abci-rs = "0.2"
//! ```
//!
//! Each ABCI application has to implement three core traits corresponding to all three ABCI connections, `Consensus`,
//! `Mempool` and `Info`.
//!
//! > Note: Implementations of these traits are expected to be `Send + Sync` and methods take immutable reference of `self`.
//! So, internal mutability must be handled using thread safe (`Arc`, `Mutex`, etc.) constructs.
//!
//! After implementing all three above mentioned `trait`s, you can create a `Server` object and use `run_sync()` or
//! `run_async()` function to start ABCI application.
//!
//! To know more, go to `examples/` to see a sample ABCI applications.
//!
//! ### Features
//!
//! - `async`: Enables ABCI Server with asynchronous IO
//!   - Only supports **`tokio`** executor.
//!   - **Disabled** by default.
//! - `uds`: Enables support for running ABCI server over Unix Domain Socket (UDS)
//!   - Supported on **Unix** only.
//!   - **Disabled** by default.
//!
//! ## Supported Versions
//!
//! - Tendermint v0.32.0
//! - ABCI v0.16.0
mod application;
#[cfg(feature = "async")]
mod async_runner;
mod proto;
mod server;
mod sync_runner;

pub mod types;

pub use self::application::{Consensus, Info, Mempool};
#[cfg(feature = "async")]
pub use self::async_runner::run_async;
pub use self::server::{Address, Server};
pub use self::sync_runner::run_sync;
