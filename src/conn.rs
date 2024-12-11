//! Connection and request handlers

use crate::constants::{ARBITRARY_IPV4, BUFFER_LEN, TTL};
use crate::errors::ConnectionError;
use crate::message::{Class, Header, Message, OpCode, Qr, ResourceRecord, ResponseCode, Type};
use anyhow::Result;
use deku::{DekuContainerRead, DekuContainerWrite};
use log::{debug, info};
use tokio::net::UdpSocket;

pub async fn handle_request(udp_socket: &UdpSocket) -> Result<(), ConnectionError> {
    let mut buf = [0u8; BUFFER_LEN];

    //
    // <== Query
    //
    let (read, source) = udp_socket
        .recv_from(&mut buf)
        .await
        .map_err(ConnectionError::RecvError)?;
    info!("<= Received {} bytes from {}", read, source);
    debug!("<= Received {} bytes from {}", read, source); // todo rem

    eprintln!("<= buf = {:02x?}, {read}", &buf[..read]); // todo rem
    let qmsg = Message::from_bytes((&mut buf, 0))?;
    debug!("<= {:?}", qmsg.1);
    eprintln!("<= qmsg.1 = {:?}", qmsg.1); // todo rem
    let qheader = qmsg.1.header;
    eprintln!("<= qheader = {:?}", qheader); // todo rem
    let questions = qmsg.1.question;
    eprintln!("<= questions = {:?}", questions); // todo rem

    //
    // --> Response
    //

    // Response code
    let rcode = if qheader.opcode == OpCode::Query {
        ResponseCode::NoError
    } else {
        ResponseCode::NotImplemented
    };

    // The ordinal of the Q & A count - could be used in a loop if needed.
    let count = 1;

    let rheader = Header {
        id: qheader.id,
        qr: Qr::Response,
        opcode: qheader.opcode,
        aa: 0,
        tc: 0,
        rd: qheader.rd,
        ra: 0,
        z: 0,
        rcode,
        qdcount: count,
        ancount: count,
        nscount: 0,
        arcount: 0,
    };
    eprintln!("-> rheader = {:02x?}", rheader); // todo rem

    // Response data
    let rdata = ARBITRARY_IPV4;

    let answers = questions
        .iter()
        .clone()
        .map(|q| ResourceRecord::new(q.qname.clone(), Type::A, Class::IN, TTL, Vec::from(rdata)))
        .collect::<Vec<_>>();

    let rmsg = Message {
        header: rheader,
        question: questions,
        answer: answers,
    };
    eprintln!("-> rmsg = {:?}", rmsg); // todo rem

    let mut response = [0; BUFFER_LEN];
    let wrote = rmsg.to_slice(&mut response)?;
    eprintln!("-> response = {:02x?}, {wrote}", &response[..wrote]); // todo rem
    let written = udp_socket
        .send_to(&response[..wrote], source)
        .await
        .map_err(ConnectionError::SendError)?;
    info!("-> Sent {} bytes back to {}", written, source);
    eprintln!("-> Sent {} bytes back to {}", written, source); // todo rem
    eprintln!("\n\n"); // todo rem

    Ok(())
}
