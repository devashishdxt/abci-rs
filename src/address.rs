use std::net::SocketAddr;
#[cfg(unix)]
use std::path::PathBuf;

#[cfg(test)]
use mock_io::tokio::MockListener;

/// Address of ABCI Server
#[derive(Debug)]
pub enum Address {
    /// TCP Address
    Tcp(SocketAddr),
    /// UDS Address
    #[cfg(unix)]
    #[cfg_attr(feature = "doc", doc(cfg(unix)))]
    Uds(PathBuf),
    /// Mock Address
    #[cfg(test)]
    Mock(MockListener),
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Self {
        Self::Tcp(addr)
    }
}

#[cfg(unix)]
impl From<PathBuf> for Address {
    fn from(path: PathBuf) -> Self {
        Self::Uds(path)
    }
}

#[cfg(test)]
impl From<MockListener> for Address {
    fn from(listener: MockListener) -> Self {
        Self::Mock(listener)
    }
}
