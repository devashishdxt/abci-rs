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

pub type Result<T> = std::result::Result<T, Error>;
