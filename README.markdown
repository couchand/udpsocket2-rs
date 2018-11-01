udpsocket2
==========

A more ergonomic Tokio-enabled UDP socket.

In particular, attention is paid to the fact that a UDP socket can both send and receive datagrams, and that a practical consumer would like to be able to do both of these things, interleaved on the same socket, with non-blocking I/O.
