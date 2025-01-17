//! # Constants
//!
//! Constants used throughout the application

/// Local host IPv4 address and port
pub const LOCAL_SOCKET_ADDR_STR: &str = "127.0.0.1:2053";

/// Length of buffer for handling connections, 512 bytes
pub const BUFFER_LEN: usize = 1 << 9;

/// Time-to-live
pub const TTL: u32 = 60;

/// An arbitrary IPv4 address
pub const ARBITRARY_IPV4: [u8; 4] = [192, 168, 1, 1];

/// Application exit codes
#[derive(Debug)]
pub enum ExitCode {
    Shutdown = -1,
    UdpRecv = -2,
    ForwardingError = -3,
}
