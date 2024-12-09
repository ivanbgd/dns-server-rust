//! # Errors
//!
//! Error types and helper functions used in the library

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

    #[error(transparent)]
    DekuError(#[from] DekuError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
