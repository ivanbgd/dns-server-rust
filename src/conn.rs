//! Connection and request handlers

use crate::constants::{ARBITRARY_IPV4, BUFFER_LEN, TTL};
use crate::errors::ConnectionError;
use crate::message::{Class, Header, Message, OpCode, Qr, ResourceRecord, ResponseCode, Type};
use anyhow::Result;
use deku::{DekuContainerRead, DekuContainerWrite};
use log::{debug, info};
use tokio::net::UdpSocket;

pub async fn handle_request(udp_socket: &UdpSocket) -> Result<(), ConnectionError> {
    //
    // <== Query
    //

    let mut buf = [0u8; BUFFER_LEN];
    let (read, source) = udp_socket
        .recv_from(&mut buf)
        .await
        .map_err(ConnectionError::RecvError)?;
    info!("<= Received {} bytes from {}", read, source);
    debug!("<= Received {} bytes from {}", read, source); // todo rem

    eprintln!("<= buf = {:02x?}, {read}", &buf[..read]); // todo rem
    let qmsg = Message::from_bytes((&mut buf, 0))?;
    debug!("<= {:?}", qmsg.1);
    eprintln!("<= qmsg.1 = {:02x?}", qmsg.1); // todo rem
    let qheader = qmsg.1.header;
    eprintln!("<= qheader = {:?}", qheader); // todo rem
    let questions = qmsg.1.question;
    eprintln!("<= questions = {:02x?}", questions); // todo rem

    //
    // --> Response
    //

    // Response code
    let rcode = if qheader.opcode == OpCode::Query {
        ResponseCode::NoError
    } else {
        ResponseCode::NotImplemented
    };

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
        qdcount: qheader.qdcount,
        ancount: qheader.qdcount,
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
    debug!("-> {:?}", rmsg);
    eprintln!("-> rmsg = {:?}", rmsg); // todo rem

    let mut buf = [0u8; BUFFER_LEN];
    let wrote = rmsg.to_slice(&mut buf)?;
    eprintln!("-> response = {:02x?}, {wrote}", &buf[..wrote]); // todo rem
    let written = udp_socket
        .send_to(&buf[..wrote], source)
        .await
        .map_err(ConnectionError::SendError)?;
    info!("-> Sent {} bytes back to {}", written, source);
    eprintln!("-> Sent {} bytes back to {}", written, source); // todo rem
    eprintln!("\n\n"); // todo rem

    Ok(())
}

/// Parse the Question section
fn parse_question(
    buf: &[u8],
    rest: &[u8],
    qheader: &Header,
    questions: &mut Vec<Question>,
) -> Result<(), ConnectionError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::conn::parse_question;
    use crate::message::{Header, Qclass, Qtype};
    use deku::DekuContainerRead;

    #[test]
    fn one_question_uncompressed() {
        let buf: [u8; 43] = [
            77, 77, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0,
            //
            // "abc.longassdomainname.com"
            3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110, 110,
            97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
        ];

        let (rest, qheader) = Header::from_bytes((&buf, 0)).unwrap();
        let rest = rest.0;

        let mut questions = vec![];
        parse_question(&buf, rest, &qheader, &mut questions).unwrap();

        assert_eq!(1, questions.len());

        assert_eq!(
            vec![
                3u8, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110,
                110, 97, 109, 101, 3, 99, 111, 109, 0
            ],
            questions[0].qname
        );
        assert_eq!(Qtype::A, questions[0].qtype);
        assert_eq!(Qclass::IN, questions[0].qclass);
    }

    #[test]
    fn two_questions_uncompressed() {
        let buf: [u8; 74] = [
            77, 77, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0,
            //
            // "abc.longassdomainname.com"
            3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110, 110,
            97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
            //
            // "def.longassdomainname.com"
            3, 100, 101, 102, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110,
            110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
        ];

        let (rest, qheader) = Header::from_bytes((&buf, 0)).unwrap();
        let rest = rest.0;

        let mut questions = vec![];
        parse_question(&buf, rest, &qheader, &mut questions).unwrap();

        assert_eq!(2, questions.len());

        assert_eq!(
            vec![
                3u8, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110,
                110, 97, 109, 101, 3, 99, 111, 109, 0
            ],
            questions[0].qname
        );
        assert_eq!(Qtype::A, questions[0].qtype);
        assert_eq!(Qclass::IN, questions[0].qclass);

        assert_eq!(
            vec![
                3u8, 100, 101, 102, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105,
                110, 110, 97, 109, 101, 3, 99, 111, 109, 0
            ],
            questions[1].qname
        );
        assert_eq!(Qtype::A, questions[1].qtype);
        assert_eq!(Qclass::IN, questions[1].qclass);
    }

    #[test]
    fn two_questions_compressed() {
        let buf: [u8; 53] = [
            77, 77, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0,
            //
            // "abc.longassdomainname.com"
            3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110, 110,
            97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
            //
            // "def.longassdomainname.com"
            3, 100, 101, 102, 192, 16, 0, 1, 0, 1,
        ];

        let (rest, qheader) = Header::from_bytes((&buf, 0)).unwrap();
        let rest = rest.0;

        let mut questions = vec![];
        parse_question(&buf, rest, &qheader, &mut questions).unwrap();

        assert_eq!(2, questions.len());

        assert_eq!(
            vec![
                3u8, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110,
                110, 97, 109, 101, 3, 99, 111, 109, 0
            ],
            questions[0].qname
        );
        assert_eq!(Qtype::A, questions[0].qtype);
        assert_eq!(Qclass::IN, questions[0].qclass);

        assert_eq!(
            vec![
                3u8, 100, 101, 102, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105,
                110, 110, 97, 109, 101, 3, 99, 111, 109, 0
            ],
            questions[1].qname
        );
        assert_eq!(Qtype::A, questions[1].qtype);
        assert_eq!(Qclass::IN, questions[1].qclass);
    }

    #[test]
    fn four_questions_compressed() {
        let buf: [u8; 52] = [
            // 0..=19: header & "aa"
            77, 77, 1, 0, 0, 4, 0, 0, 0, 0, 0, 0, 2, 97, 97, 0, 0, 1, 0, 1,
            //
            // 20..=35: "f.isi.arpa"
            1, 102, 3, 105, 115, 105, 4, 97, 114, 112, 97, 0, 0, 1, 0, 1,
            //
            // 36..=51: "foo.f.isi.arpa", "arpa"
            3, 102, 111, 111, 192, 20, 0, 1, 0, 1, 192, 26, 0, 1, 0, 1,
        ];

        let (rest, qheader) = Header::from_bytes((&buf, 0)).unwrap();
        let rest = rest.0;

        let mut questions = vec![];
        parse_question(&buf, rest, &qheader, &mut questions).unwrap();

        assert_eq!(4, questions.len());

        assert_eq!(vec![2u8, 97, 97, 0], questions[0].qname); // "aa"
        assert_eq!(
            vec![1u8, 102, 3, 105, 115, 105, 4, 97, 114, 112, 97, 0], // "f.isi.arpa"
            questions[1].qname
        );
        assert_eq!(
            vec![3u8, 102, 111, 111, 1, 102, 3, 105, 115, 105, 4, 97, 114, 112, 97, 0], // "foo.f.isi.arpa"
            questions[2].qname
        );
        assert_eq!(vec![4u8, 97, 114, 112, 97, 0], questions[3].qname); // "arpa"
    }
}
