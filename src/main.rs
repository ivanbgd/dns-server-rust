//! # A DNS Server

use codecrafters_dns_server::constants::{BUFFER_LEN, LOCAL_SOCKET_ADDR_STR};
use log::{info, warn};
use std::net::UdpSocket;

fn main() {
    env_logger::init();
    info!("Starting the server...");

    let udp_socket = UdpSocket::bind(LOCAL_SOCKET_ADDR_STR).expect("Failed to bind to address");
    let mut buf = [0; BUFFER_LEN];

    info!("Waiting for requests...");

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                info!("Received {} bytes from {}", size, source);
                let response = [];
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                warn!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
