//! Connection and request handlers

use crate::constants::BUFFER_LEN;
use crate::errors::ConnectionError;
use crate::message::{Header, Message, OpCode, Qr, ResponseCode};
use anyhow::Result;
use deku::{DekuContainerRead, DekuContainerWrite};
use log::{debug, info};
use tokio::net::UdpSocket;

pub async fn handle_request(udp_socket: &UdpSocket, buf: &mut [u8]) -> Result<(), ConnectionError> {
    // <= Query
    let (read, source) = udp_socket
        .recv_from(buf)
        .await
        .map_err(ConnectionError::RecvError)?;
    info!("<= Received {} bytes from {}", read, source);
    let qmsg = Message::from_bytes((buf, 0))?;
    debug!("{:?}", qmsg.1);
    let qheader = qmsg.1.header;

    // -> Response
    let rcode = if qheader.opcode == OpCode::Query {
        ResponseCode::NoError
    } else {
        ResponseCode::NotImplemented
    };
    let rheader = Header {
        id: qheader.id,
        qr: Qr::Response,
        opcode: OpCode::Query,
        aa: 0,
        tc: 0,
        rd: qheader.rd,
        ra: 0,
        z: 0,
        rcode,
        qdcount: 0,
        ancount: 0,
        nscount: 0,
        arcount: 0,
    };
    let rmsg = Message { header: rheader };
    let mut response = [0; BUFFER_LEN];
    let wrote = rmsg.to_slice(&mut response)?;
    let written = udp_socket
        .send_to(&response[..wrote], source)
        .await
        .map_err(ConnectionError::SendError)?;
    info!("-> Sent {} bytes back to {}", written, source);

    Ok(())
}
