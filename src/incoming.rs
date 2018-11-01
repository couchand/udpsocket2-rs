use std::io;
use std::iter::repeat;
use std::sync::{Arc, Mutex};

use futures::{Stream, Poll, Async};
use tokio::net::UdpSocket;

use crate::UdpDatagram;

const MAX_MESSAGE_LENGTH: usize = 1024;

/// A stream of incoming UDP datagrams.
#[derive(Debug)]
pub struct Incoming {
    socket: Arc<Mutex<UdpSocket>>,
    buffer: Vec<u8>,
}

impl Incoming {
    pub(crate) fn new(socket: Arc<Mutex<UdpSocket>>) -> Incoming {
        let mut buffer = Vec::with_capacity(MAX_MESSAGE_LENGTH);
        buffer.extend(repeat(0u8).take(MAX_MESSAGE_LENGTH));
        Incoming { socket, buffer }
    }
}

impl Stream for Incoming {
    type Item = UdpDatagram;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let (length, peer) = try_ready!{
            self.socket.lock().unwrap().poll_recv_from(&mut self.buffer)
        };

        let mut data = Vec::with_capacity(length);
        data.extend_from_slice(&self.buffer[..length]);

        Ok(Async::Ready(Some(
            UdpDatagram { peer, data }
        )))
    }
}
