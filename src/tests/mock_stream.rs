use std::{
    future::Future,
    io::{ErrorKind, Result},
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender},
        Mutex,
    },
};

macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            Poll::Ready(t) => t,
            Poll::Pending => return Poll::Pending,
        }
    };
}

/// Mock IO listener
#[derive(Debug)]
pub struct MockListener {
    receiver: Receiver<MockStream>,
}

impl MockListener {
    /// Creates a new mock listener with stream sender
    pub fn new() -> (Self, Sender<MockStream>) {
        let (sender, receiver) = unbounded_channel();
        (Self { receiver }, sender)
    }
}

impl Deref for MockListener {
    type Target = Receiver<MockStream>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl DerefMut for MockListener {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.receiver
    }
}

/// Mock IO stream
#[derive(Debug, Clone)]
pub struct MockStream {
    receiver: Arc<Mutex<Receiver<Vec<u8>>>>,
    sender: Arc<Sender<Vec<u8>>>,
    remaining: Arc<Mutex<Vec<u8>>>,
}

impl MockStream {
    /// Creates a pair of connected mock streams
    pub fn pair() -> (MockStream, MockStream) {
        let (sender_1, receiver_1) = unbounded_channel();
        let (sender_2, receiver_2) = unbounded_channel();

        let stream_1 = Self {
            receiver: Arc::new(Mutex::new(receiver_1)),
            sender: Arc::new(sender_2),
            remaining: Default::default(),
        };

        let stream_2 = Self {
            receiver: Arc::new(Mutex::new(receiver_2)),
            sender: Arc::new(sender_1),
            remaining: Default::default(),
        };

        (stream_1, stream_2)
    }
}

impl AsyncRead for MockStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        let available_space = buf.remaining();

        let mut remaining_lock = Box::pin(self.remaining.lock());
        let mut remaining = ready!(remaining_lock.as_mut().poll(cx));

        if remaining.is_empty() {
            let mut receiver_lock = Box::pin(self.receiver.lock());
            let mut receiver = ready!(receiver_lock.as_mut().poll(cx));
            let mut bytes_future = Box::pin(receiver.recv());
            let bytes = ready!(bytes_future.as_mut().poll(cx));

            match bytes {
                None => return Poll::Pending,
                Some(bytes) => {
                    *remaining = bytes;
                }
            }
        }

        if remaining.len() > available_space {
            buf.put_slice(&remaining[..available_space]);
            *remaining = remaining[available_space..].to_vec();
            Poll::Ready(Ok(()))
        } else {
            buf.put_slice(&remaining);
            *remaining = Default::default();
            Poll::Ready(Ok(()))
        }
    }
}

impl AsyncWrite for MockStream {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let result = self.sender.send(buf.to_vec());
        match result {
            Ok(_) => Poll::Ready(Ok(buf.len())),
            Err(_) => Poll::Ready(Err(ErrorKind::Other.into())),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }
}
