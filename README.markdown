udpsocket2
==========

A more ergonomic Tokio-enabled UDP socket.

In particular, attention is paid to the fact that a UDP socket can both send and receive datagrams, and that a practical consumer would like to be able to do both of these things, interleaved on the same socket, with non-blocking I/O.

```rust
extern crate futures;
extern crate udpsocket2;

use udpsocket2::UdpSocket;

fn main() -> std::io::Result<()> {

    use futures::{Future, Stream};
    use futures::future::ok;

    let socket = UdpSocket::bind("127.0.0.1:34254")?;

    tokio::run(ok(()).and_then(move |_| {

        tokio::spawn(
            socket.incoming()
                .for_each(|datagram| { println!("{:?}", datagram); Ok(()) })
                .map_err(|_| ())
        );

        tokio::spawn(
            socket.send_to(&[0xde, 0xad, 0xbe, 0xef], "127.0.0.1:34254")?
                .map_err(|_| ())
        );

        Ok(())

    }).map_err(|_: std::io::Error| ()));

    Ok(())
}
```
