//! # Message & its parts
//!
//! https://datatracker.ietf.org/doc/html/rfc1035#section-4.1
//!
//! https://en.wikipedia.org/wiki/Domain_Name_System#DNS_message_format

use deku::prelude::*;

#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
// #[deku(endian = "big")] todo
pub struct Message {
    pub header: Header,
}

#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
// #[deku(endian = "big")] todo
pub struct Header {
    pub id: u16,
    pub small: HeaderSmallFields,
    // pub small: u16, // HeaderSmallFields todo
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
// #[deku(endian = "big")] todo
pub struct HeaderSmallFields {
    #[deku(bits = 1)]
    pub qr: u8,
    pub opcode: OpCode, // OpCode
    // #[deku(bits = 4)] todo
    // pub opcode: u8, // OpCode todo
    #[deku(bits = 1)]
    pub aa: u8,
    #[deku(bits = 1)]
    pub tc: u8,
    #[deku(bits = 1)]
    pub rd: u8,
    #[deku(bits = 1)]
    pub ra: u8,
    #[deku(bits = 3)]
    pub z: u8,
    #[deku(bits = 4)]
    pub rcode: u8,
}

#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
#[deku(id_type = "u8", bits = "4")]
pub enum OpCode {
    #[deku(id = "0")]
    Query = 0,

    #[deku(id = "1")]
    Iquery = 1,

    #[deku(id = "2")]
    Status = 2,

    #[deku(id_pat = "4..=15")]
    Reserved,
}
