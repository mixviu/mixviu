// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

#[test]
fn owned_port_survives_bounces_and_stop_is_offline() {
    let mut broker = test_broker::Broker::bounce_stable();
    let url = broker.url.clone();
    let address = (std::net::Ipv4Addr::LOCALHOST, broker.tcp_port());
    for _ in 0..3 {
        // A live fixture owns the selected port; another fixture cannot bind it.
        let error = std::net::TcpListener::bind(address).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AddrInUse);
        std::net::TcpStream::connect(address).unwrap();
        broker.stop();
        let error = std::net::TcpStream::connect(address).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::ConnectionRefused);
        broker.bounce();
        assert_eq!(broker.url, url);
        assert_eq!(broker.tcp_port(), address.1);
    }
}
