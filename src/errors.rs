//! # Errors
//!
//! Error types and helper functions used in the library

use deku::DekuError;
use std::array::TryFromSliceError;
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

    #[error(transparent)]
    DekuError(#[from] DekuError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Errors related to working with [`crate::message`]
#[derive(Debug, Error)]
pub enum MessageError {
    #[error(transparent)]
    Slice(#[from] TryFromSliceError),

    #[error("Unsupported OpCode: {0}")]
    OpCodeError(u8),

    #[error("Unsupported ResponseCode: {0}")]
    ResponseCodeError(u8),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
