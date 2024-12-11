//! # A DNS Server Application

use anyhow::{Context, Result};
use codecrafters_dns_server::conn::handle_request;
use codecrafters_dns_server::constants::{ExitCode, BUFFER_LEN, LOCAL_SOCKET_ADDR_STR};
use codecrafters_dns_server::errors::{ApplicationError, ConnectionError};
use log::{error, info, warn};
use std::process::exit;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    env_logger::init();
    info!("Starting the DNS server...");

    let udp_socket = UdpSocket::bind(LOCAL_SOCKET_ADDR_STR)
        .await
        .with_context(|| format!("Failed to bind to address {}", LOCAL_SOCKET_ADDR_STR))?;

    info!("Waiting for requests...");

    loop {
        let mut buf = [0; BUFFER_LEN];

        match handle_request(&udp_socket, &mut buf).await {
            Ok(_) => {}
            Err(ConnectionError::RecvError(e)) => {
                error!("{e}");
                error!("Terminating the app ({})...", ExitCode::UdpRecv as i32);
                exit(ExitCode::UdpRecv as i32)
            }
            Err(e) => {
                warn!("{e}");
            }
        }

        tokio::spawn(async move {
            // Await the shutdown signal
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    info!("CTRL+C received. Shutting down...");
                    exit(0);
                }
                Err(err) => {
                    // We also shut down in case of error.
                    error!("Unable to listen for the shutdown signal: {}", err);
                    error!("Terminating the app ({})...", ExitCode::Shutdown as i32);
                    exit(ExitCode::Shutdown as i32)
                }
            };
        });
    }
}
