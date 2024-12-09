//! # Message & its parts
//!
//! https://datatracker.ietf.org/doc/html/rfc1035#section-4.1
//!
//! https://en.wikipedia.org/wiki/Domain_Name_System#DNS_message_format

use deku::prelude::*;

/// DNS Message
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
pub struct Message {
    pub header: Header,
}

/// DNS Message Header
///
///       0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                      ID                       |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                    QDCOUNT                    |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                    ANCOUNT                    |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                    NSCOUNT                    |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                    ARCOUNT                    |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
pub struct Header {
    /// A 16-bit identifier assigned by the program that generates any kind of query.
    /// This identifier is copied into the corresponding reply and can be used by the requester
    /// to match up replies to outstanding queries.
    #[deku(endian = "big")]
    pub id: u16,

    /// A one-bit field that specifies whether this message is a query (0), or a response (1).
    pub qr: Qr,

    /// A four-bit field that specifies kind of query in this message.
    /// This value is set by the originator of a query and copied into the response.
    pub opcode: OpCode,

    /// Authoritative Answer - this bit is valid in responses, and specifies that the responding name server is an
    /// authority for the domain name in question section.
    #[deku(bits = 1)]
    pub aa: u8,

    /// TrunCation - specifies that this message was truncated due to length greater than that permitted on the
    /// transmission channel.
    #[deku(bits = 1)]
    pub tc: u8,

    /// Recursion Desired - this bit may be set in a query and is copied into the response.
    /// If RD is set, it directs the name server to pursue the query recursively.
    /// Recursive query support is optional.
    #[deku(bits = 1)]
    pub rd: u8,

    /// Recursion Available - this be is set or cleared in a response,
    /// and denotes whether recursive query support is available in the name server.
    #[deku(bits = 1)]
    pub ra: u8,

    /// Reserved for future use.  Must be zero in all queries and responses.
    #[deku(bits = 3)]
    pub z: u8,

    /// Response code - this 4-bit field is set as part of responses.
    pub rcode: ResponseCode,

    /// An unsigned 16-bit integer specifying the number of entries in the question section.
    #[deku(endian = "big")]
    pub qdcount: u16,

    /// An unsigned 16-bit integer specifying the number of resource records in the answer section.
    #[deku(endian = "big")]
    pub ancount: u16,

    /// An unsigned 16-bit integer specifying the number of name server resource records
    /// in the authority records section.
    #[deku(endian = "big")]
    pub nscount: u16,

    /// An unsigned 16-bit integer specifying the number of resource records in the additional records section.
    #[deku(endian = "big")]
    pub arcount: u16,
}

/// A one-bit field that specifies whether this message is a query (0), or a response (1).
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
#[deku(id_type = "u8", bits = "1", endian = "big")]
pub enum Qr {
    /// Query
    #[deku(id = "0")]
    Query = 0,

    /// Response
    #[deku(id = "1")]
    Response = 1,
}

/// A four-bit field that specifies kind of query in this message.
/// This value is set by the originator of a query and copied into the response.
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
#[deku(id_type = "u8", bits = "4", endian = "big")]
pub enum OpCode {
    /// a standard query (QUERY)
    #[deku(id = "0")]
    Query = 0,

    /// an inverse query (IQUERY)
    #[deku(id = "1")]
    InverseQuery = 1,

    /// a server status request (STATUS)
    #[deku(id = "2")]
    Status = 2,

    /// reserved for future use
    #[deku(id_pat = "3..=15")]
    Reserved,
}

/// Response code - this 4-bit field is set as part of responses.
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
#[deku(id_type = "u8", bits = "4", endian = "big")]
pub enum ResponseCode {
    /// No error condition
    #[deku(id = "0")]
    NoError = 0,

    /// Format error - The name server was unable to interpret the query.
    #[deku(id = "1")]
    FormatError = 1,

    /// Server failure - The name server was unable to process this query due to a problem with the name server.
    #[deku(id = "2")]
    ServerFailure = 2,

    /// Name Error - Meaningful only for responses from an authoritative name server,
    /// this code signifies that the domain name referenced in the query does not exist.
    #[deku(id = "3")]
    NameError = 3,

    /// Not Implemented - The name server does not support the requested kind of query.
    #[deku(id = "4")]
    NotImplemented = 4,

    /// Refused - The name server refuses to perform the specified operation for policy reasons.
    #[deku(id = "5")]
    Refused = 5,

    /// Reserved for future use.
    #[deku(id_pat = "6..=15")]
    Reserved,
}
