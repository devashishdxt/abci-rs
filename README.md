# abci-rs

A Rust crate for creating ABCI applications.

## ABCI Overview

ABCI is the interface between Tendermint (a state-machine replication engine) and your application (the actual state
machine). It consists of a set of methods, where each method has a corresponding `Request` and `Response` message type.
Tendermint calls the ABCI methods on the ABCI application by sending the `Request` messages and receiving the `Response`
messages in return.

ABCI methods are split across 3 separate ABCI connections:

- `Consensus` Connection: `InitChain`, `BeginBlock`, `DeliverTx`, `EndBlock`, `Commit`
- `Mempool` Connection: `CheckTx`
- `Info` Connection: `Info`, `SetOption`, `Query`

Additionally, there is a `Flush` method that is called on every connection, and an `Echo` method that is just for
debugging.

To know more about ABCI protocol specifications, go to official ABCI [documentation](https://tendermint.com/docs/spec/abci/).

## Usage

Add `abci-rs` in your `Cargo.toml`'s `dependencies` section:

```toml
[dependencies]
abci-rs = "0.3"
```

Each ABCI application has to implement three core traits corresponding to all three ABCI connections, `Consensus`,
`Mempool` and `Info`.

> Note: Implementations of these traits are expected to be `Send + Sync` and methods take immutable reference of `self`.
So, internal mutability must be handled using thread safe (`Arc`, `Mutex`, etc.) constructs.

After implementing all three above mentioned `trait`s, you can create a `Server` object and use `Server::run()` to start
ABCI application.

`Server::run()` is an `async` function and returns a `Future`. So, you'll need an executor to drive `Future` returned
from `Server::run()`. `async-std` and `tokio` are two popular options. In `counter` example, we use `async-std`'s
executor.

To know more, go to `examples/` to see a sample ABCI application.

### Features

- `uds`: Enables support for running ABCI server over Unix Domain Socket (UDS)
  - Supported on **Unix** only.
  - **Disabled** by default.

## Supported Versions

- Tendermint v0.32.0
- ABCI v0.16.0

## License

Licensed under either of

- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (http://opensource.org/licenses/MIT)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as 
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
