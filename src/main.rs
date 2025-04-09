//! # A DNS Server Application

use anyhow::{Context, Result};
use dns_server::conn::handle_request;
use dns_server::constants::{ExitCode, LOCAL_SOCKET_ADDR_STR};
use dns_server::errors::{ApplicationError, ConnectionError};
use log::{error, info, warn};
use std::env;
use std::net::SocketAddrV4;
use std::process::exit;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    env_logger::init();
    info!("Starting the DNS server...");

    let args = env::args().collect::<Vec<String>>();
    let mut resolver: Option<SocketAddrV4> = None;
    if args.len() >= 3 && args[1] == "--resolver" {
        info!("Working in the forwarding mode; forward to {}", args[2]);
        resolver = Some(args[2].parse().expect("Failed to parse resolver address"));
    } else {
        info!("Working in the resolver mode.");
    }

    let udp_socket = UdpSocket::bind(LOCAL_SOCKET_ADDR_STR)
        .await
        .with_context(|| format!("Failed to bind to address {}", LOCAL_SOCKET_ADDR_STR))?;

    main_loop(udp_socket, resolver).await
}

/// Resolve DNS queries
async fn main_loop(
    udp_socket: UdpSocket,
    resolver: Option<SocketAddrV4>,
) -> Result<(), ApplicationError> {
    info!("Waiting for requests...");

    loop {
        match handle_request(&udp_socket, resolver).await {
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

        shutdown().await;
    }
}

/// Await the shutdown signal
async fn shutdown() {
    tokio::spawn(async move {
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
