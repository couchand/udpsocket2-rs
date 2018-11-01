use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::{Future, Poll, Async};
use tokio::net::UdpSocket;

use crate::UdpDatagram;

/// A future representing a UDP datagram currently being sent.
#[derive(Debug)]
pub struct SendTo<'a> {
    socket: Arc<Mutex<UdpSocket>>,
    buffer: &'a [u8],
    addr: SocketAddr,
}

impl<'a> SendTo<'a> {
    pub(crate) fn new(socket: Arc<Mutex<UdpSocket>>, buffer: &'a [u8], addr: SocketAddr) -> SendTo {
        SendTo { socket, buffer, addr }
    }
}

impl<'a> Future for SendTo<'a> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let _ = try_ready!{
            self.socket.lock().unwrap().poll_send_to(&self.buffer, &self.addr)
        };

        Ok(Async::Ready(()))
    }
}

/// A future representing a UDP datagram currently being sent.
#[derive(Debug)]
pub struct Send {
    socket: Arc<Mutex<UdpSocket>>,
    datagram: UdpDatagram,
}

impl Send {
    pub(crate) fn new(socket: Arc<Mutex<UdpSocket>>, datagram: UdpDatagram) -> Send {
        Send { socket, datagram }
    }
}

impl Future for Send {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let _ = try_ready!{
            self.socket.lock().unwrap().poll_send_to(&self.datagram.data, &self.datagram.peer)
        };

        Ok(Async::Ready(()))
    }
}
