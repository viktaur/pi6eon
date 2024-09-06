use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use std::net::Ipv6Addr;
use crate::msg::messaging_loop;

/// Start a new TCP connection with the destination.
pub async fn setup_connection(address: Ipv6Addr, port: u16) -> Result<()> {
    let stream = TcpStream::connect((address, port)).await?;
    println!("Successfully connected to {address}:{port}");
    messaging_loop(stream).await?;
    Ok(())
}

/// Listen for incoming connections on the provided port.
pub async fn listen_for_connection(port: u16) -> Result<()> {
    let listener = TcpListener::bind((Ipv6Addr::LOCALHOST, port)).await?;

    // We'll only accept one connection at a time.
    while let Ok((stream, socket_addr)) = listener.accept().await {
        println!("Accepted connection request from {socket_addr}");
        messaging_loop(stream).await?;
    }

    Ok(())
}
