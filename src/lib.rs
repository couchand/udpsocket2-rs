//! A more ergonomic Tokio-enabled UDP socket.
//!
//! In particular, attention is paid to the fact that a UDP socket can both
//! send and receive datagrams, and that a practical consumer would like to be
//! able to do both of these things, interleaved on the same socket, with
//! non-blocking I/O.
//!
//! See the [`UdpSocket`] struct documentation for more details.
//!
//! [`UdpSocket`]: struct.UdpSocket.html
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
//! let socket = UdpSocket::bind("127.0.0.1:34254")?;
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
//!     socket.send_to(&[0xde, 0xad, 0xbe, 0xef], "127.0.0.1:34254")?
//!         .map_err(|_| ())
//! );
//! #
//! #         Ok(())
//! #
//! #     }).map_err(|_: std::io::Error| ()));
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UdpDatagram {
    /// The peer for this datagram: if it was received, the source, if it is
    /// to be sent, the destination.
    pub peer: SocketAddr,
    /// The data content of the datagram.
    pub data: Vec<u8>,
}

/// A UDP socket, using non-blocking I/O.
///
/// Intended to be used within a Tokio runtime, this offers a more ergonomic
/// interface compared to the standard tokio::net::UdpSocket.
///
/// Bind to a socket with the [`bind`] method.  Receive datagrams as a
/// [`Stream`] with [`incoming`], and send datagrams with [`send_to`].
///
/// [`bind`]: #method.bind
/// [`Stream`]: ../futures/stream/trait.Stream.html
/// [`incoming`]: #method.incoming
/// [`send_to`]: #method.send_to
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
/// let socket = UdpSocket::bind("127.0.0.1:34254")?;
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
///     socket.send_to(&[0xde, 0xad, 0xbe, 0xef], "127.0.0.1:34254")?
///         .map_err(|_| ())
/// );
/// #
/// #         Ok(())
/// #
/// #     }).map_err(|_: std::io::Error| ()));
/// #
/// #     Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct UdpSocket {
    socket: Arc<Mutex<TokioUdpSocket>>,
}

impl UdpSocket {
    fn new(socket: TokioUdpSocket) -> UdpSocket {
        UdpSocket { socket: Arc::new(Mutex::new(socket)) }
    }

    /// Creates a UDP socket from the given address.
    ///
    /// The address parameter, which must implement [`ToSocketAddrs`], is
    /// treated the same as for [`std::net::UdpSocket::bind`].  See the
    /// documentation there for details.
    ///
    /// [`ToSocketAddrs`]: https://doc.rust-lang.org/std/net/trait.ToSocketAddrs.html
    /// [`std::net::UdpSocket::bind`]: https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.bind
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate udpsocket2;
    /// # use udpsocket2::UdpSocket;
    /// #
    /// # fn main() -> std::io::Result<()> {
    /// #
    /// let socket = UdpSocket::bind("127.0.0.1:34254")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn bind<Addrs: ToSocketAddrs>(addrs: Addrs) -> Result<UdpSocket, io::Error> {
        each_addr(addrs, TokioUdpSocket::bind).map(UdpSocket::new)
    }

    /// Returns a stream of datagrams (as [`UdpDatagram`] structs) received on
    /// this socket.
    ///
    /// [`UdpDatagram`]: struct.UdpDatagram.html
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
    /// #     let socket = UdpSocket::bind("127.0.0.1:34254")?;
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
    /// with [`std::net::UdpSocket::send_to`].
    ///
    /// [`std::net::UdpSocket::send_to`]: https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.send_to
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
    /// #     let socket = UdpSocket::bind("127.0.0.1:34254")?;
    /// #
    /// #     tokio::run(ok(()).and_then(move |_| {
    /// #
    /// tokio::spawn(
    ///     socket.send_to(&[0xde, 0xad, 0xbe, 0xef], "127.0.0.1:34254")?
    ///         .map_err(|_| ())
    /// );
    /// #
    /// #         Ok(())
    /// #
    /// #     }).map_err(|_: std::io::Error| ()));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn send_to<'a, Addr: ToSocketAddrs>(
        &self, buffer: &'a [u8], addr: Addr
    ) ->
        Result<SendTo<'a>, io::Error>
    {
        let addr = match addr.to_socket_addrs()?.next() {
            Some(addr) => addr,
            None => return Err(
                io::Error::new(io::ErrorKind::InvalidInput,
                     "no addresses to send data to")
            ),
        };

        Ok(SendTo::new(self.socket.clone(), buffer, addr))
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
    /// #     let socket = UdpSocket::bind("127.0.0.1:34254")?;
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
