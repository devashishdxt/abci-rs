/// ABCI Error
pub struct Error {
    /// Error code
    pub code: u32,
    /// Output of application's logger (may be non-deterministic)
    pub log: String,
    /// Additional information (may be non-deterministic)
    pub info: String,
}
