//! Connection and request handlers

use crate::errors::ConnectionError;
use anyhow::Result;
use log::debug;
use tokio::net::UdpSocket;

pub async fn handle_request(udp_socket: &UdpSocket, buf: &mut [u8]) -> Result<(), ConnectionError> {
    let (read, source) = udp_socket
        .recv_from(buf)
        .await
        .map_err(ConnectionError::RecvError)?;
    debug!("<= Received {} bytes from {}", read, source);
    let response = [];
    let written = udp_socket
        .send_to(&response, source)
        .await
        .map_err(ConnectionError::SendError)?;
    debug!("-> Sent {} bytes back to {}", written, source);

    Ok(())
}
