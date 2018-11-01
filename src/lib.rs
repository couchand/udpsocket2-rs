//! A more ergonomic Tokio-enabled UDP socket.
//!
//! # Examples
//!
//! ```no_run
//! # extern crate futures;
//! # extern crate udpsocket2;
//! #
//! # use udpsocket2::UdpSocket;
//! #
//! # fn main() -> std::io::Result<()> {
//! #
//! #     use futures::{Future, Stream};
//! #     use futures::future::ok;
//! #
//! let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
//! #
//! #     tokio::run(ok(()).and_then(move |_| {
//!
//! tokio::spawn(
//!     socket.incoming()
//!         .for_each(|datagram| { println!("{:?}", datagram); Ok(()) })
//!         .map_err(|_| ())
//! );
//!
//! tokio::spawn(
//!     socket.send_to(&[0xde, 0xad, 0xbe, 0xef], "127.0.0.1:34254")
//!         .map_err(|_| ())
//! );
//! #
//! #         Ok(())
//! #
//! #     }));
//! #
//! #     Ok(())
//! # }
//! ```

#[macro_use]
extern crate futures;
extern crate tokio;

pub mod incoming;
pub mod send_to;

#[cfg(test)]
mod tests;

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};

use tokio::net::{UdpSocket as TokioUdpSocket};

use incoming::Incoming;
use send_to::{Send, SendTo};

/// An individual UDP datagram, either having been received or to be sent over
/// a UDP socket.
#[derive(Debug)]
pub struct UdpDatagram {
    pub peer: SocketAddr,
    pub data: Vec<u8>,
}

/// A UDP socket, using non-blocking I/O.
///
/// Intended to be used within a Tokio runtime, this offers a few advantages
/// over the standard tokio::net::UdpSocket, for instance, a more ergonomic
/// interface.
///
/// # Examples
///
/// ```no_run
/// # extern crate futures;
/// # extern crate udpsocket2;
/// #
/// # use udpsocket2::UdpSocket;
/// #
/// # fn main() -> std::io::Result<()> {
/// #
/// #     use futures::{Future, Stream};
/// #     use futures::future::ok;
/// #
/// let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
/// #
/// #     tokio::run(ok(()).and_then(move |_| {
///
/// tokio::spawn(
///     socket.incoming()
///         .for_each(|datagram| { println!("{:?}", datagram); Ok(()) })
///         .map_err(|_| ())
/// );
///
/// tokio::spawn(
///     socket.send_to(&[0xde, 0xad, 0xbe, 0xef], "127.0.0.1:34254")
///         .map_err(|_| ())
/// );
/// #
/// #         Ok(())
/// #
/// #     }));
/// #
/// #     Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct UdpSocket {
    socket: Arc<Mutex<TokioUdpSocket>>,
}

impl UdpSocket {
    fn new(socket: TokioUdpSocket) -> UdpSocket {
        UdpSocket { socket: Arc::new(Mutex::new(socket)) }
    }

    /// Creates a UDP socket from the given address.
    ///
    /// The address parameter is treated the same as `std::net::UdpSocket::bind`,
    /// see the documentation there for precise details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate udpsocket2;
    /// # use udpsocket2::UdpSocket;
    /// #
    /// # fn main() -> std::io::Result<()> {
    /// #
    /// let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn bind<Addrs: ToSocketAddrs>(addrs: Addrs) -> Result<UdpSocket, io::Error> {
        each_addr(addrs, TokioUdpSocket::bind).map(UdpSocket::new)
    }

    /// Returns a stream of datagrams received on this socket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate futures;
    /// # extern crate udpsocket2;
    /// #
    /// # use udpsocket2::UdpSocket;
    /// #
    /// # fn main() -> std::io::Result<()> {
    /// #
    /// #     use futures::{Future, Stream};
    /// #     use futures::future::ok;
    /// #
    /// #     let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
    /// #
    /// #     tokio::run(ok(()).and_then(move |_| {
    /// #
    /// tokio::spawn(
    ///     socket.incoming()
    ///         .for_each(|datagram| { println!("{:?}", datagram); Ok(()) })
    ///         .map_err(|_| ())
    /// );
    /// #
    /// #         Ok(())
    /// #
    /// #     }));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn incoming(&self) -> Incoming {
        Incoming::new(self.socket.clone())
    }

    /// Sends data to the given address via the socket.  Returns a future which
    /// resolves when the datagram has been written.
    ///
    /// Though `addr` might resolve to multiple socket addresses, this will
    /// only attempt to send to the first resolved address, to be consistent
    /// with `std::net::UdpSocket::send_to`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate futures;
    /// # extern crate udpsocket2;
    /// #
    /// # use udpsocket2::UdpSocket;
    /// #
    /// # fn main() -> std::io::Result<()> {
    /// #
    /// #     use futures::{Future, Stream};
    /// #     use futures::future::ok;
    /// #
    /// #     let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
    /// #
    /// #     tokio::run(ok(()).and_then(move |_| {
    /// #
    /// tokio::spawn(
    ///     socket.send_to(&[0xde, 0xad, 0xbe, 0xef], "127.0.0.1:34254")
    ///         .map_err(|_| ())
    /// );
    /// #
    /// #         Ok(())
    /// #
    /// #     }));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn send_to<'a, Addr: ToSocketAddrs>(&mut self, buffer: &'a [u8], addr: Addr) -> SendTo<'a> {
        // TODO: not unwrap!
        let addr = addr.to_socket_addrs().unwrap().nth(0).unwrap();
        SendTo::new(self.socket.clone(), buffer, addr)
    }

    /// Sends a datagram to the given address via the socket.  Returns a future
    /// which resolves when the datagram has been written.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate futures;
    /// # extern crate udpsocket2;
    /// #
    /// # use udpsocket2::UdpSocket;
    /// #
    /// # fn main() -> std::io::Result<()> {
    /// #
    /// #     use futures::{Future, Stream};
    /// #     use futures::future::ok;
    /// #
    /// #     let mut socket = UdpSocket::bind("127.0.0.1:34254")?;
    /// #
    /// #     tokio::run(ok(()).and_then(move |_| {
    /// #
    /// tokio::spawn(
    ///     socket.incoming()
    ///         .map_err(|_| ())
    ///         .for_each(move |datagram| {
    ///             // echo!
    ///             socket.send(datagram)
    ///                 .map_err(|_| ())
    ///         })
    /// );
    /// #
    /// #         Ok(())
    /// #
    /// #     }));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn send(&self, datagram: UdpDatagram) -> Send {
        Send::new(self.socket.clone(), datagram)
    }
}

// the below is totally cribbed from std::net
fn each_addr<A: ToSocketAddrs, F, T>(addr: A, mut f: F) -> io::Result<T>
    where F: FnMut(&SocketAddr) -> io::Result<T>
{
    let mut last_err = None;
    for addr in addr.to_socket_addrs()? {
        match f(&addr) {
            Ok(l) => return Ok(l),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput,
                   "could not resolve to any addresses")
    }))
}
