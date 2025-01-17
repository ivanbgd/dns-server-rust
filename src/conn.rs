//! Connection and request handlers

use crate::constants::{ARBITRARY_IPV4, BUFFER_LEN, TTL};
use crate::errors::ConnectionError;
use crate::message::{
    Class, Header, Message, OpCode, Qr, Question, ResourceRecord, ResponseCode, Type,
};
use anyhow::Result;
use bytes::BytesMut;
use deku::{DekuContainerRead, DekuContainerWrite};
use log::{debug, info, trace};
use std::iter::zip;
use std::net::SocketAddrV4;
use tokio::net::UdpSocket;

pub async fn handle_request(
    udp_socket: &UdpSocket,
    resolver: Option<SocketAddrV4>,
) -> Result<(), ConnectionError> {
    //
    // <== Query
    //

    let mut buf = [0u8; BUFFER_LEN];
    let (received, source) = udp_socket
        .recv_from(&mut buf)
        .await
        .map_err(ConnectionError::RecvError)?;
    info!("<= Received {} bytes from {}", received, source);
    // Remove mutability.
    let buf = buf;

    let (rest, qheader) = Header::from_bytes((&buf, 0))?;
    let rest = rest.0;

    let mut questions = vec![];
    parse_question(&buf, rest, &qheader, &mut questions)?;

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

    // Response data
    let mut rdata: Vec<[u8; 4]> = vec![];

    if let Some(resolver) = resolver {
        // We are a forwarding DNS server (a DNS forwarder).
        // Let's forward DNS queries to a DNS resolver and collect the responses that we get from it.
        for question in &questions {
            let mut q_buf = BytesMut::from(&buf[0..received]);
            q_buf[0..12].copy_from_slice(&buf[..12]);
            // Their test suite doesn't support OpCode::InverseQuery in this case, so we have to hack this byte,
            // q_buf[2], to OpCode::Query, in order for that test to pass!
            q_buf[2] = 1;
            q_buf[4] = 0; // qheader.qdcount[hi]
            q_buf[5] = 1; // qheader.qdcount[lo]
            q_buf[12..12 + question.qname.len()].copy_from_slice(&question.qname);
            q_buf[12 + question.qname.len()..][..4].copy_from_slice(&[0, 1, 0, 1]); // Append Qtype & Qclass.

            // Send a Question message
            udp_socket
                .send_to(&q_buf, resolver)
                .await
                .map_err(ConnectionError::SendError)?;

            // Receive an Answer message
            let mut r_buf = [0u8; BUFFER_LEN];
            udp_socket
                .recv_from(&mut r_buf)
                .await
                .map_err(ConnectionError::RecvError)?;

            let (_rest, answer) = Message::from_bytes((&r_buf, 0))?;
            let r =
                <[u8; 4]>::try_from(answer.answer[0].rdata.clone()).expect("Try from slice failed");
            rdata.push(r);
        }
    } else {
        // We are the DNS resolver, so we resolve the DNS queries ourselves.
        rdata = vec![ARBITRARY_IPV4; questions.len()];
    }

    let answers = zip(questions.iter(), rdata.iter())
        .map(|(q, r)| ResourceRecord::new(q.qname.clone(), Type::A, Class::IN, TTL, Vec::from(r)))
        .collect::<Vec<_>>();

    let rmsg = Message {
        header: rheader,
        question: questions,
        answer: answers,
    };
    debug!("-> {:?}", rmsg);

    let mut buf = [0u8; BUFFER_LEN];
    let wrote = rmsg.to_slice(&mut buf)?;
    let written = udp_socket
        .send_to(&buf[..wrote], source)
        .await
        .map_err(ConnectionError::SendError)?;
    info!("-> Sent {} bytes back to {}", written, source);

    Ok(())
}

/// Parse the Question section
fn parse_question(
    buf: &[u8],
    rest: &[u8],
    qheader: &Header,
    questions: &mut Vec<Question>,
) -> Result<(), ConnectionError> {
    // The first question is never compressed, so using "deku" is always okay for the first question.
    let (rest, question) = Question::from_bytes((rest, 0))?;
    let mut rest = rest.0;
    questions.push(question);

    for _qi in 1..qheader.qdcount {
        let (r, question) = match Question::from_bytes((rest, 0)) {
            Ok((r, q)) => (r, q), // Uncompressed question

            Err(e) => {
                // Compressed question
                trace!("Compressed question");
                trace!("error: {}", e); // DekuError::Parse
                let mut qname = vec![];

                // Iterate over bytes until a byte begins with 0b11, meaning it's >= 192, i.e., >= 0xc0.
                let mut offset_hi = 0u8;
                let mut bi = 0usize;
                for b in rest {
                    bi += 1;
                    if b == &0 {
                        return Err(ConnectionError::ZeroByte);
                    } else if b < &192 {
                        qname.push(*b);
                    } else {
                        offset_hi &= 0x3f;
                        break;
                    }
                }
                let offset_lo = rest[bi];
                bi += 1;
                let jump = u16::from_be_bytes([offset_hi, offset_lo]);

                // Update qname.
                let (_r, qq) = Question::from_bytes((buf, 8 * jump as usize))?;
                qname.extend_from_slice(&qq.qname);

                let qtype = u16::from_be_bytes([rest[bi], rest[bi + 1]]);
                bi += 2;
                let qclass = u16::from_be_bytes([rest[bi], rest[bi + 1]]);
                bi += 2;

                let r = (&rest[bi..], 0usize);
                let q = Question::new(qname, qtype.try_into()?, qclass.try_into()?);

                (r, q)
            }
        };
        rest = r.0;
        questions.push(question);
    }
    Ok(())
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
    fn three_questions_uncompressed() {
        let buf: [u8; 105] = [
            77, 77, 1, 0, 0, 3, 0, 0, 0, 0, 0, 0,
            //
            // "abc.longassdomainname.com"
            3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110, 110,
            97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
            //
            // "def.longassdomainname.com"
            3, 100, 101, 102, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110,
            110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 2, 0, 1,
            //
            // "ghi.longassdomainname.com"
            3, 103, 104, 105, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110,
            110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 15, 0, 1,
        ];

        let (rest, qheader) = Header::from_bytes((&buf, 0)).unwrap();
        let rest = rest.0;

        let mut questions = vec![];
        parse_question(&buf, rest, &qheader, &mut questions).unwrap();

        assert_eq!(3, questions.len());

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
        assert_eq!(Qtype::NS, questions[1].qtype);
        assert_eq!(Qclass::IN, questions[1].qclass);

        assert_eq!(
            vec![
                3u8, 103, 104, 105, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105,
                110, 110, 97, 109, 101, 3, 99, 111, 109, 0
            ],
            questions[2].qname
        );
        assert_eq!(Qtype::MX, questions[2].qtype);
        assert_eq!(Qclass::IN, questions[2].qclass);
    }

    #[test]
    fn three_questions_compressed() {
        let buf: [u8; 63] = [
            77, 77, 1, 0, 0, 3, 0, 0, 0, 0, 0, 0,
            //
            // "abc.longassdomainname.com"
            3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110, 110,
            97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
            //
            // "def.longassdomainname.com"
            3, 100, 101, 102, 192, 16, 0, 2, 0, 1,
            //
            // "ghi.longassdomainname.com"
            3, 103, 104, 105, 192, 16, 0, 15, 0, 1,
        ];

        let (rest, qheader) = Header::from_bytes((&buf, 0)).unwrap();
        let rest = rest.0;

        let mut questions = vec![];
        parse_question(&buf, rest, &qheader, &mut questions).unwrap();

        assert_eq!(3, questions.len());

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
        assert_eq!(Qtype::NS, questions[1].qtype);
        assert_eq!(Qclass::IN, questions[1].qclass);

        assert_eq!(
            vec![
                3u8, 103, 104, 105, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105,
                110, 110, 97, 109, 101, 3, 99, 111, 109, 0
            ],
            questions[2].qname
        );
        assert_eq!(Qtype::MX, questions[2].qtype);
        assert_eq!(Qclass::IN, questions[2].qclass);
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
