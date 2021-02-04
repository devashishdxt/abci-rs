#[cfg(all(unix, feature = "use-async-std"))]
use async_std::os::unix::net::UnixStream;
#[cfg(feature = "use-async-std")]
use async_std::{
    io::{Read, Write},
    net::TcpStream,
};
#[cfg(test)]
use mock_io::tokio::{MockStream, ReadHalf as MockReadHalf, WriteHalf as MockWriteHalf};
#[cfg(all(unix, feature = "use-smol"))]
use smol::net::unix::UnixStream;
#[cfg(feature = "use-smol")]
use smol::{
    io::{AsyncRead as Read, AsyncWrite as Write},
    net::TcpStream,
};
#[cfg(all(unix, feature = "use-tokio"))]
use tokio::net::{
    unix::{OwnedReadHalf as UnixReadHalf, OwnedWriteHalf as UnixWriteHalf},
    UnixStream,
};
#[cfg(feature = "use-tokio")]
use tokio::{
    io::{AsyncRead as Read, AsyncWrite as Write},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub trait StreamSplit {
    type Reader: Read + Send + Unpin + 'static;
    type Writer: Write + Send + Unpin + 'static;

    /// Splits the stream into reader and writer halves
    fn split_stream(self) -> (Self::Reader, Self::Writer);
}

#[cfg(any(feature = "use-async-std", feature = "use-smol"))]
impl StreamSplit for TcpStream {
    type Reader = Self;
    type Writer = Self;

    fn split_stream(self) -> (Self::Reader, Self::Writer) {
        (self.clone(), self)
    }
}

#[cfg(feature = "use-tokio")]
impl StreamSplit for TcpStream {
    type Reader = OwnedReadHalf;
    type Writer = OwnedWriteHalf;

    fn split_stream(self) -> (Self::Reader, Self::Writer) {
        self.into_split()
    }
}

#[cfg(all(unix, any(feature = "use-async-std", feature = "use-smol")))]
impl StreamSplit for UnixStream {
    type Reader = Self;
    type Writer = Self;

    fn split_stream(self) -> (Self::Reader, Self::Writer) {
        (self.clone(), self)
    }
}

#[cfg(all(unix, feature = "use-tokio"))]
impl StreamSplit for UnixStream {
    type Reader = UnixReadHalf;
    type Writer = UnixWriteHalf;

    fn split_stream(self) -> (Self::Reader, Self::Writer) {
        self.into_split()
    }
}

#[cfg(test)]
impl StreamSplit for MockStream {
    type Reader = MockReadHalf;
    type Writer = MockWriteHalf;

    fn split_stream(self) -> (Self::Reader, Self::Writer) {
        self.split()
    }
}
