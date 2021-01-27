#![deny(missing_docs, unsafe_code)]
//! A Rust crate for creating ABCI applications.
//!
//! # ABCI Overview
//!
//! ABCI is the interface between Tendermint (a state-machine replication engine) and your application (the actual state
//! machine). It consists of a set of methods, where each method has a corresponding `Request` and `Response` message
//! type. Tendermint calls the ABCI methods on the ABCI application by sending the `Request` messages and receiving the
//! `Response` messages in return.
//!
//! ABCI methods are split across 4 separate ABCI connections:
//!
//! - `Consensus` Connection: `InitChain`, `BeginBlock`, `DeliverTx`, `EndBlock`, `Commit`
//! - `Mempool` Connection: `CheckTx`
//! - `Info` Connection: `Info`, `SetOption`, `Query`
//! - `Snapshot` Connection: `ListSnapshots`, `LoadSnapshotChunk`, `OfferSnapshot`, `ApplySnapshotChunk`
//!
//! Additionally, there is a `Flush` method that is called on every connection, and an `Echo` method that is just for
//! debugging.
//!
//! To know more about ABCI protocol specifications, go to official ABCI [documentation](https://tendermint.com/docs/spec/abci/).
//!
//! # Usage
//!
//! Add `abci-rs` in your `Cargo.toml`'s `dependencies` section:
//!
//! ```toml
//! [dependencies]
//! abci-rs = "0.11"
//! ```
//!
//! Each ABCI application has to implement four core traits corresponding to all four ABCI connections, `Consensus`,
//! `Mempool`, `Info` and `Snapshot`.
//!
//! > Note: Implementations of these traits are expected to be `Send + Sync` and methods take immutable reference of
//! `self`. So, internal mutability must be handled using thread safe (`Arc`, `Mutex`, etc.) constructs.
//!
//! ## Synchronous and asynchronous APIs
//!
//! `abci-rs` supports both, synchronous and asynchronous APIs (using `sync-api` and `async-api` cargo features). At
//! least one of these features should be enabled. By default, both, `sync-api` and `async-api`, features are enabled.
//!
//! ## Async runtimes
//!
//! `abci-rs` also supports multiple async runtimes. These different runtimes can be enabled by using cargo features
//! `use-async-std`, `use-smol` or `use-tokio`. Only one runtime can be enabled at a time. Compilation will fail if more
//! than one runtime is enabled or none of them are enabled. By default, `use-tokio` feature is enabled.
//!
//! ## Examples
//!
//! Example ABCI applications can be found in `examples/sync-counter.rs` (using `sync_api`) and `examples/async-counter.rs`
//! (using `async_api`).
//!
//! # Minimum Supported Versions
//!
//! - Tendermint: [`v0.34.3`](https://github.com/tendermint/tendermint/releases/tag/v0.34.3)
#![cfg_attr(feature = "doc", feature(doc_cfg))]

#[cfg(not(any(feature = "async-api", feature = "sync-api")))]
compile_error!("Either feature `async-api` or `sync-api` must be enabled for this crate");

#[cfg(not(any(feature = "use-async-std", feature = "use-smol", feature = "use-tokio",)))]
compile_error!("One runtime should be enabled: `use-async-std`, `use-smol` or `use-tokio`");

#[cfg(all(feature = "use-async-std", feature = "use-smol"))]
compile_error!("Only one runtime should be enabled: `use-async-std`, `use-smol` or `use-tokio`");
#[cfg(all(feature = "use-async-std", feature = "use-tokio"))]
compile_error!("Only one runtime should be enabled: `use-async-std`, `use-smol` or `use-tokio`");
#[cfg(all(feature = "use-tokio", feature = "use-smol"))]
compile_error!("Only one runtime should be enabled: `use-async-std`, `use-smol` or `use-tokio`");

mod address;
cfg_if::cfg_if! {
    if #[cfg(feature = "async-api")] {
        #[cfg_attr(feature = "doc", doc(cfg(feature = "async-api")))]
        pub mod async_api;
    } else {
        mod async_api;
    }
}
mod handler;
mod state;
mod stream_split;
#[cfg(feature = "sync-api")]
#[cfg_attr(feature = "doc", doc(cfg(feature = "sync-api")))]
pub mod sync_api;
mod tasks;
#[cfg(test)]
mod tests;
pub mod types;
mod utils;

#[cfg(feature = "async-api")]
#[cfg_attr(feature = "doc", doc(cfg(feature = "async-api")))]
pub use async_trait::async_trait;

pub use self::address::Address;
