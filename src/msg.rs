use std::io::Write;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng}, Aes256Gcm, Key, Nonce // Or `Aes128Gcm`
};
use tokio::{io::AsyncReadExt, net::tcp::{OwnedReadHalf, OwnedWriteHalf}};
use tokio::io::{self, BufReader, AsyncBufReadExt, AsyncWriteExt};
use anyhow::{Result, anyhow};
use x25519_dalek::SharedSecret;
// use crate::crypto::{encrypt_with_nonce, decrypt_with_nonce};

async fn parse_nonce(stream_reader: &mut BufReader<OwnedReadHalf>) -> Result<[u8; 12]> {
    let mut buf = [0u8; 12];
    if let Ok(12) = stream_reader.read_exact(&mut buf).await {
        Ok(buf)
    } else {
        Err(anyhow!("Error when attempting to parse the nonce."))
    }
}

async fn parse_bytes_to_read(stream_reader: &mut BufReader<OwnedReadHalf>) -> Result<usize> {
    let mut buf = [0u8; 2];
    if let Ok(2) = stream_reader.read_exact(&mut buf).await {
        Ok(u16::from_be_bytes(buf) as usize)
    } else {
        Err(anyhow!("Error when attempting to parse the number of bytes to read."))
    }
}

async fn parse_ciphertext(
    stream_reader: &mut BufReader<OwnedReadHalf>,
    bytes_to_read: usize
) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; bytes_to_read];

    match stream_reader.read_exact(&mut buf).await {
        Ok(0) => Err(anyhow!("Connection closed.")),
        Ok(_) => Ok(buf),
        Err(e) => Err(anyhow!("Error when reading from the stream: {}", e))
    }
}

fn construct_encoded_msg(
    nonce: [u8; 12],
    bytes_to_read: u16,
    ciphertext: &[u8]
) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.extend(nonce);
    bytes.extend(bytes_to_read.to_be_bytes());
    bytes.extend(ciphertext);
    bytes
}

async fn read_task(
    cipher: &Aes256Gcm,
    mut stream_reader: BufReader<OwnedReadHalf>
) -> Result<()> {
    loop {
        let nonce: Nonce<_> = parse_nonce(&mut stream_reader).await?.into();
        let bytes_to_read = parse_bytes_to_read(&mut stream_reader).await?;
        let ciphertext = parse_ciphertext(&mut stream_reader, bytes_to_read).await?;

        let plaintext_bytes = cipher.decrypt(&nonce, ciphertext.as_slice())
            .map_err(|e| anyhow!(e))?;
        let plaintext = std::str::from_utf8(&plaintext_bytes)?;
        print!("\rFriend: {}", plaintext);
        print!("\rYou: ");
        std::io::stdout().flush()?;
    }
}

async fn write_task(
    cipher: &Aes256Gcm,
    mut writer: OwnedWriteHalf
) -> Result<()> {
    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin);

    let mut input = String::new();
    loop {
        print!("\rYou: ");
        std::io::stdout().flush()?;

        input.clear();
        stdin_reader.read_line(&mut input).await?;
        // if input.trim() == "quit" { break }

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let ciphertext = cipher.encrypt(&nonce, input.as_bytes())
            .map_err(|e| anyhow!(e))?;
        let bytes_to_read = ciphertext.len();
        let encoded_msg = construct_encoded_msg(nonce.into(), bytes_to_read as u16, &ciphertext);

        writer.write_all(&encoded_msg).await?;
        writer.flush().await?;
    }

}

/// Starts two asynchronous tasks which read from and write to a stream.
pub async fn messaging_loop(
    stream_reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    secret: SharedSecret
) -> Result<()> {
    let key = Key::<Aes256Gcm>::from_slice(secret.as_bytes());
    let cipher = Aes256Gcm::new(&key);

    tokio::try_join!(
        read_task(&cipher, stream_reader),
        write_task(&cipher, writer)
    )?;

    Ok(())
}
