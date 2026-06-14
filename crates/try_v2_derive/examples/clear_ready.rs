#![feature(never_type)]
#![allow(unused, clippy::disallowed_names)]
use std::{io, pin::Pin, task::{Context, Poll, ready}};

struct MySocket;
trait Stream {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}

enum Ready {
    Read,
    Write,
    ReadWrite
}

impl Ready {
    fn is_read(&self) -> bool {
        todo!()
    }
}

/// Async polling for a socket
trait PollableSocket
where
    Self: Sized,
{
    /// Clear the readiness state of the underlying socket.
    ///
    /// **This MUST be called after any failed readiness poll.**
    ///
    /// Implementations should attempt to clear the relevant readiness marker of the underlying
    /// socket and then return:
    /// - `Poll::Pending` if successful
    /// - `Poll::Ready(error)` on error, to avoid repeated polling without handling the error
    fn clear_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<!>>;

    /// Check whether the socket is ready.
    ///
    /// ## Note
    ///
    /// You **MUST** call self.clear_ready() in the following cases:
    ///
    /// - If this fails it may leave the socket in an undefined readiness state.
    /// - If you do not make use of the readiness it will remain blocked in that state.
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Ready>>;
}

impl PollableSocket for MySocket {
    fn clear_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<!>> {
        todo!()
    }

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Ready>> {
        todo!()
    }
}

impl Stream for MySocket {
    type Item = io::Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match ready!(self.as_mut().poll_ready(cx)) {
            Ok(readiness) if readiness.is_read() => todo!("read and stream"),
            _ => self.clear_ready(cx).map_ok(|x| x).map(Some), // <- .map_ok(|x| x) to coerce ! to String
        }
    }
}

fn main() {}
