#![allow(missing_docs)]
//! Types used in ABCI
mod begin_block;
mod check_tx;
mod commit;
mod deliver_tx;
mod end_block;
mod info;
mod init_chain;
mod misc;
mod query;
mod set_option;

pub use self::begin_block::*;
pub use self::check_tx::*;
pub use self::commit::*;
pub use self::deliver_tx::*;
pub use self::end_block::*;
pub use self::info::*;
pub use self::init_chain::*;
pub use self::misc::*;
pub use self::query::*;
pub use self::set_option::*;

/// ABCI Error
pub struct Error {
    /// Error code
    pub code: u32,
    /// Namespace for error code
    pub codespace: String,
    /// Output of application's logger (may be non-deterministic)
    pub log: String,
    /// Additional information (may be non-deterministic)
    pub info: String,
}

/// ABCI Result
pub type Result<T> = std::result::Result<T, Error>;
