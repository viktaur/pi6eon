use anyhow::{anyhow, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use std::net::Ipv6Addr;
use crate::msg::messaging_loop;
use x25519_dalek::{EphemeralSecret, PublicKey};

/// Start a new TCP connection with the destination, perform Diffie-Hellman, start
/// messaging.
pub async fn setup_connection(address: Ipv6Addr, port: u16) -> Result<()> {
    // Connect to Bob
    let stream = TcpStream::connect((address, port)).await?;
    println!("Successfully connected to {address}:{port}");

    // Split the stream into reader and writer, which can be used simultaneously
    let (reader, mut writer) = stream.into_split();
    let mut stream_reader = BufReader::new(reader);

    // Generate keys for Alice
    let alice_secret = EphemeralSecret::random();
    let alice_public = PublicKey::from(&alice_secret);
    println!("Secret and public key pair generated.");

    // Send Alice's public key to Bob
    writer.write_all(alice_public.as_bytes()).await?;
    writer.flush().await?;

    // Receive Bob's public key
    let mut buf: [u8; 32] = [0; 32];
    match stream_reader.read_exact(&mut buf).await {
        Ok(32) => {
            println!("Friend's public key received: {:x?}", buf);
        },
        _ => {
            return Err(
                anyhow!(
                    "Incorrect number of bytes read for friend's public key. Aborting."
                )
            );
        }
    };
    let bob_public = PublicKey::from(buf);

    // Create the shared secret
    let shared_secret = alice_secret.diffie_hellman(&bob_public);
    println!("Shared secret: {:x?}", shared_secret.to_bytes());

    // Start encrypted messaging
    messaging_loop(stream_reader, writer, shared_secret).await?;

    Ok(())
}

/// Listen for incoming connections on the provided port, perform Diffie-Hellman and start
/// messaging.
pub async fn listen_for_connection(port: u16) -> Result<()> {
    let listener = TcpListener::bind((Ipv6Addr::LOCALHOST, port)).await?;
    println!("TCP Listener created on port {port}");

    // We'll only accept one connection at a time.
    while let Ok((stream, socket_addr)) = listener.accept().await {
        println!("Accepted connection request from {socket_addr}");

        // Split the stream into reader and writer, which can be used simultaneously
        let (reader, mut writer) = stream.into_split();
        let mut stream_reader = BufReader::new(reader);

        // Generate keys for Bob
        let bob_secret = EphemeralSecret::random();
        let bob_public = PublicKey::from(&bob_secret);
        println!("Secret and public key pair generated.");

        // Send Bob's keys to Alice
        writer.write_all(bob_public.as_bytes()).await?;
        writer.flush().await?;

        // Receive Alice's public key
        let mut buf: [u8; 32] = [0; 32];
        match stream_reader.read_exact(&mut buf).await {
            Ok(32) => {
                println!("Friend's public key received: {:x?}", buf);
            },
            _ => {
                return Err(
                    anyhow!(
                        "Incorrect number of bytes read for friend's public key. Aborting."
                    )
                );
            }
        };
        let alice_public = PublicKey::from(buf);

        // Create the shared secret
        let shared_secret = bob_secret.diffie_hellman(&alice_public);
        println!("Shared secret: {:x?}", shared_secret.to_bytes());

        // Start encrypted messaging
        messaging_loop(stream_reader, writer, shared_secret).await?;
    }

    Ok(())
}
