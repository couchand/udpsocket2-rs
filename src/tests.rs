use futures::{Future, Stream};

use crate::*;

#[test]
fn test_listener() {
    use std::sync::{Arc, Mutex};

    let results = Arc::new(Mutex::new(Vec::new()));
    let results2 = results.clone();

    let socket = UdpSocket::bind("localhost:9999").unwrap();
    let listener = socket
        .incoming()
        .map_err(|e| panic!("udp accept err: {:?}", e))
        .for_each(move |datagram| {
            results2.lock().unwrap().push(datagram);
            Ok(())
        });

    let outgoing = tokio::timer::Delay::new(
        std::time::Instant::now()
    ).map_err(|e| eprintln!("timer err: {:?}", e))
        .and_then(|_| {
            tokio::spawn(listener);

            tokio::timer::Delay::new(
        std::time::Instant::now() + std::time::Duration::from_millis(10)
    )
        .map_err(|e| eprintln!("timer err: {:?}", e))
    })
        .and_then(|_| {

    let socket2 = UdpSocket::bind("localhost:9998").unwrap();
    socket2.send_to(&[42], "localhost:9999")
        .expect("error sending udp datagram!")
        .map_err(|e| eprintln!("udp send err: {:?}", e))
        .and_then(|_| {
            let socket3 = UdpSocket::bind("localhost:9997").unwrap();
            let peer = "localhost:9999".to_socket_addrs().unwrap().nth(0).unwrap();
            let data = vec![0, 1, 2, 3];
            socket3.send(UdpDatagram { peer, data })
                .map_err(|e| eprintln!("udp send2 err: {:?}", e))
        })
        .and_then(|_| {
            tokio::timer::Delay::new(
                std::time::Instant::now() + std::time::Duration::from_millis(10)
            ).map_err(|e| eprintln!("timer err: {:?}", e))
        })
        .then(move |_| {
            let res = results.lock().unwrap();
            assert_eq!(res.len(), 2);

            let actual = &res[0];
            assert_eq!(actual.data.len(), 1);
            assert_eq!(actual.data[0], 42);

            let actual = &res[1];
            assert_eq!(actual.data.len(), 4);
            assert_eq!(actual.data[0], 0);
            assert_eq!(actual.data[1], 1);
            assert_eq!(actual.data[2], 2);
            assert_eq!(actual.data[3], 3);
            Ok(())
        })

        });

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(outgoing).unwrap();
}
