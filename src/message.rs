//! # Message & its parts
//!
//! https://datatracker.ietf.org/doc/html/rfc1035#section-4.1
//!
//! https://en.wikipedia.org/wiki/Domain_Name_System#DNS_message_format
//!
//! https://www.rfc-editor.org/rfc/rfc1035#section-3.2

use crate::errors::{QclassError, QtypeError};
use anyhow::Result;
use deku::prelude::*;

/// # DNS Message
///
/// All communications inside of the domain protocol are carried in a single
/// format called a message.  The top level format of message is divided
/// into 5 sections (some of which are empty in certain cases) shown below:
///
///     +---------------------+
///     |        Header       |
///     +---------------------+
///     |       Question      | the question for the name server
///     +---------------------+
///     |        Answer       | RRs answering the question
///     +---------------------+
///     |      Authority      | RRs pointing toward an authority
///     +---------------------+
///     |      Additional     | RRs holding additional information
///     +---------------------+
///
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
pub struct Message {
    /// The header
    pub header: Header,

    /// Questions for the name server
    #[deku(count = "header.qdcount")]
    pub question: Vec<Question>,

    /// Answers to the questions asked in the question section
    #[deku(count = "header.ancount")]
    pub answer: Vec<ResourceRecord>,
}

/// # DNS Message Header
///
/// The header section is always present.  The header includes fields that
/// specify which of the remaining sections are present, and also specify
/// whether the message is a query or a response, a standard query or some
/// other opcode, etc.
///
/// A header's length is always 12 bytes.
///
///                                     1  1  1  1  1  1
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
///
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
#[deku(id_type = "u8", bits = "1")]
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
#[deku(id_type = "u8", bits = "4")]
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
    Reserved,
}

/// Response code - this 4-bit field is set as part of responses.
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
#[deku(id_type = "u8", bits = "4")]
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
    Reserved,
}

/// # DNS Question
///
/// The question section is used to carry the "question" in most queries,
/// i.e., the parameters that define what is being asked.  The section
/// contains QDCOUNT (usually 1) entries, each of the following format:
///
///                                     1  1  1  1  1  1
///       0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                                               |
///     /                     QNAME                     /
///     /                                               /
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                     QTYPE                     |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                     QCLASS                    |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
pub struct Question {
    /// QNAME:          a domain name represented as a sequence of labels, where
    ///                 each label consists of a length octet followed by that
    ///                 number of octets.  The domain name terminates with the
    ///                 zero length octet for the null label of the root.  Note
    ///                 that this field may be an odd number of octets; no
    ///                 padding is used.
    #[deku(until = "|v: &u8| *v == 0")]
    pub qname: Vec<u8>,

    /// QTYPE:          a two octet code which specifies the type of the query.
    ///                 The values for this field include all codes valid for a
    ///                 TYPE field, together with some more general codes which
    ///                 can match more than one type of RR.
    pub qtype: Qtype,

    /// QCLASS:         a two octet code that specifies the class of the query.
    ///                 For example, the QCLASS field is IN for the Internet.
    pub qclass: Qclass,
}

impl Question {
    pub fn new(qname: Vec<u8>, qtype: Qtype, qclass: Qclass) -> Self {
        Self {
            qname,
            qtype,
            qclass,
        }
    }
}

/// QTYPE fields appear in the question part of a query.  QTYPES are a
/// superset of TYPEs, hence all TYPEs are valid QTYPEs.
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
#[deku(id_type = "u16", bits = "16", endian = "big")]
pub enum Qtype {
    /// a host address
    #[deku(id = "1")]
    A = 1,

    /// an authoritative name server
    #[deku(id = "2")]
    NS = 2,

    /// mail exchange
    #[deku(id = "15")]
    MX = 15,
}

impl TryFrom<u16> for Qtype {
    type Error = QtypeError;

    fn try_from(value: u16) -> Result<Qtype, QtypeError> {
        match value {
            1 => Ok(Qtype::A),
            2 => Ok(Qtype::NS),
            15 => Ok(Qtype::MX),
            v => Err(QtypeError::UnsupportedQtype(v)),
        }
    }
}

/// QCLASS fields appear in the question section of a query.  QCLASS values
/// are a superset of CLASS values; every CLASS is a valid QCLASS.
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
#[deku(id_type = "u16", bits = "16", endian = "big")]
pub enum Qclass {
    /// the Internet
    #[deku(id = "1")]
    IN = 1,
}

impl TryFrom<u16> for Qclass {
    type Error = QclassError;

    fn try_from(value: u16) -> Result<Qclass, QclassError> {
        match value {
            1 => Ok(Qclass::IN),
            v => Err(QclassError::UnsupportedQclass(v)),
        }
    }
}

/// # DNS Resource record
///
/// The answer, authority, and additional sections all share the same
/// format: a variable number of resource records, where the number of
/// records is specified in the corresponding count field in the header.
/// Each resource record has the following format:
///
///                                     1  1  1  1  1  1
///       0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                                               |
///     /                                               /
///     /                      NAME                     /
///     |                                               |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                      TYPE                     |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                     CLASS                     |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                      TTL                      |
///     |                                               |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                   RDLENGTH                    |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--|
///     /                     RDATA                     /
///     /                                               /
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
pub struct ResourceRecord {
    /// NAME:           a domain name to which this resource record pertains.
    #[deku(until = "|v: &u8| *v == 0")]
    pub name: Vec<u8>,

    /// TYPE:           two octets containing one of the RR type codes.  This
    ///                 field specifies the meaning of the data in the RDATA
    ///                 field.
    pub type_: Type,

    /// CLASS:          two octets which specify the class of the data in the
    ///                 RDATA field.
    pub class: Class,

    /// TTL             a 32-bit unsigned integer that specifies the time
    ///                 interval (in seconds) that the resource record may be
    ///                 cached before it should be discarded.  Zero values are
    ///                 interpreted to mean that the RR can only be used for the
    ///                 transaction in progress, and should not be cached.
    #[deku(endian = "big")]
    pub ttl: u32,

    /// RDLENGTH        an unsigned 16-bit integer that specifies the length in
    ///                 octets of the RDATA field.
    #[deku(endian = "big")]
    pub rdlength: u16,

    /// RDATA           a variable-length string of octets that describes the
    ///                 resource.  The format of this information varies
    ///                 according to the TYPE and CLASS of the resource record.
    ///                 For example, if the TYPE is A and the CLASS is IN,
    ///                 the RDATA field is a 4-octet ARPA Internet address.
    #[deku(count = "rdlength", endian = "big")]
    pub rdata: Vec<u8>,
}

impl ResourceRecord {
    pub fn new(name: Vec<u8>, type_: Type, class: Class, ttl: u32, rdata: Vec<u8>) -> Self {
        Self {
            name,
            type_,
            class,
            ttl,
            rdlength: rdata.len() as u16,
            rdata,
        }
    }
}

/// TYPE fields are used in resource records.  Note that these types are a
/// subset of QTYPEs.
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
#[deku(id_type = "u16", bits = "16", endian = "big")]
pub enum Type {
    /// a host address
    #[deku(id = "1")]
    A = 1,

    /// an authoritative name server
    #[deku(id = "2")]
    NS = 2,

    /// mail exchange
    #[deku(id = "15")]
    MX = 15,
}

/// CLASS fields appear in resource records.  Note that these types are a
/// subset of QCLASSes.
#[derive(Debug, DekuRead, DekuWrite, PartialEq)]
#[deku(id_type = "u16", bits = "16", endian = "big")]
pub enum Class {
    /// the Internet
    #[deku(id = "1")]
    IN = 1,
}
