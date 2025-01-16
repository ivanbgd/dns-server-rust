//! # Errors
//!
//! Error types and helper functions used in the library

use std::array::TryFromSliceError;

use deku::DekuError;
use thiserror::Error;

/// Application errors
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Errors related to working with [`crate::conn`]
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Error receiving data: {0}")]
    RecvError(std::io::Error),

    #[error("Failed to send response to {0}")]
    SendError(std::io::Error),

    #[error("received '\0' where we shoudn't have")]
    ZeroByte,

    #[error(transparent)]
    DekuError(#[from] DekuError),

    #[error(transparent)]
    Slice(#[from] TryFromSliceError),

    #[error(transparent)]
    QtypeError(#[from] QtypeError),

    #[error(transparent)]
    QclassError(#[from] QclassError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Errors related to working with [`crate::message::Qtype`]
#[derive(Debug, Error)]
pub enum QtypeError {
    #[error("Unsupported Qtype: {0}")]
    UnsupportedQtype(u16),
}

/// Errors related to working with [`crate::message::Qclass`]
#[derive(Debug, Error)]
pub enum QclassError {
    #[error("Unsupported Qclass: {0}")]
    UnsupportedQclass(u16),
}
