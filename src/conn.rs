use anyhow::{anyhow, Result};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use std::net::Ipv6Addr;
use crate::msg::messaging_loop;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

/// Start a new TCP connection with the destination, perform Diffie-Hellman, start
/// messaging.
pub async fn setup_connection(address: Ipv6Addr, port: u16) -> Result<()> {
    // Connect to Bob
    let stream = TcpStream::connect((address, port)).await?;
    println!("Successfully connected to {address}:{port}");

    // Split the stream into reader and writer, which can be used simultaneously
    let (reader, mut writer) = stream.into_split();
    let mut stream_reader = BufReader::new(reader);

    // Exchange keys and get the shared secret
    let shared_secret = key_exhange(&mut writer, &mut stream_reader).await?;

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

        // Exchange keys and get the shared secret
        let shared_secret = key_exhange(&mut writer, &mut stream_reader).await?;

        // Start encrypted messaging
        messaging_loop(stream_reader, writer, shared_secret).await?;
    }

    Ok(())
}

/// Generate a pair of private a public keys, exchange them with the other party, and
/// generate a shared secret through Diffie-Hellman.
async fn key_exhange(
    writer: &mut OwnedWriteHalf,
    stream_reader: &mut BufReader<OwnedReadHalf>
) -> Result<SharedSecret> {
    // Generate own keys
    let own_secret = EphemeralSecret::random();
    let own_public = PublicKey::from(&own_secret);
    println!("Secret and public key pair generated.");

    // Send own public key to friend
    writer.write_all(own_public.as_bytes()).await?;
    writer.flush().await?;
    println!("Own public key sent to friend: {}", hex::encode(own_public.as_bytes()));

    // Receive friend's public key
    let mut buf: [u8; 32] = [0; 32];
    match stream_reader.read_exact(&mut buf).await {
        Ok(32) => {
            println!("Friend's public key received: {}", hex::encode(buf));
        },
        _ => {
            return Err(
                anyhow!(
                    "Incorrect number of bytes read for friend's public key. Aborting."
                )
            );
        }
    };
    let friend_public = PublicKey::from(buf);

    // Create the shared secret
    let shared_secret = own_secret.diffie_hellman(&friend_public);
    println!("Shared secret's hash: {}", sha256::digest(&shared_secret.to_bytes()));
    Ok(shared_secret)
}
