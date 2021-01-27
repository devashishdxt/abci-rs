# abci-rs

[![Continuous Integration](https://github.com/devashishdxt/abci-rs/workflows/Continuous%20Integration/badge.svg)](https://github.com/devashishdxt/abci-rs/actions?query=workflow%3A%22Continuous+Integration%22)
[![Crates.io](https://img.shields.io/crates/v/abci-rs)](https://crates.io/crates/abci-rs)
[![Documentation](https://docs.rs/abci-rs/badge.svg)](https://docs.rs/abci-rs)
[![License](https://img.shields.io/crates/l/abci-rs)](https://github.com/devashishdxt/abci-rs/blob/master/LICENSE-MIT)

A Rust crate for creating ABCI applications.

## ABCI Overview

ABCI is the interface between Tendermint (a state-machine replication engine) and your application (the actual state
machine). It consists of a set of methods, where each method has a corresponding `Request` and `Response` message
type. Tendermint calls the ABCI methods on the ABCI application by sending the `Request` messages and receiving the
`Response` messages in return.

ABCI methods are split across 4 separate ABCI connections:

- `Consensus` Connection: `InitChain`, `BeginBlock`, `DeliverTx`, `EndBlock`, `Commit`
- `Mempool` Connection: `CheckTx`
- `Info` Connection: `Info`, `SetOption`, `Query`
- `Snapshot` Connection: `ListSnapshots`, `LoadSnapshotChunk`, `OfferSnapshot`, `ApplySnapshotChunk`

Additionally, there is a `Flush` method that is called on every connection, and an `Echo` method that is just for
debugging.

To know more about ABCI protocol specifications, go to official ABCI [documentation](https://tendermint.com/docs/spec/abci/).

## Usage

Add `abci-rs` in your `Cargo.toml`'s `dependencies` section:

```toml
[dependencies]
abci-rs = "0.11"
```

Each ABCI application has to implement four core traits corresponding to all four ABCI connections, `Consensus`,
`Mempool`, `Info` and `Snapshot`.

> Note: Implementations of these traits are expected to be `Send + Sync` and methods take immutable reference of
`self`. So, internal mutability must be handled using thread safe (`Arc`, `Mutex`, etc.) constructs.

### Synchronous and asynchronous APIs

`abci-rs` supports both, synchronous and asynchronous APIs (using `sync-api` and `async-api` cargo features). At
least one of these features should be enabled. By default, both, `sync-api` and `async-api`, features are enabled.

### Async runtimes

`abci-rs` also supports multiple async runtimes. These different runtimes can be enabled by using cargo features
`use-async-std`, `use-smol` or `use-tokio`. Only one runtime can be enabled at a time. Compilation will fail more
than one runtime is enabled of none of them are enabled. By default, `use-tokio` feature is enabled.

### Examples

Example ABCI applications can be found in `examples/sync-counter.rs` (using `sync_api`) and `examples/async-counter.rs`
(using `async_api`).

## Minimum Supported Versions

- Tendermint: [`v0.34.3`](https://github.com/tendermint/tendermint/releases/tag/v0.34.3)

## Documentation

- [`master`](https://devashishdxt.github.io/abci-rs/abci/)
- [`release`](https://docs.rs/abci-rs/)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
