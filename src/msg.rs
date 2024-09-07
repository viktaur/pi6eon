use std::io::Write;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key // Or `Aes128Gcm`
};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::{self, BufReader, AsyncBufReadExt, AsyncWriteExt};
use anyhow::Result;
use x25519_dalek::SharedSecret;

/// Starts two asynchronous tasks which read from and write to a stream.
pub async fn messaging_loop(
    mut stream_reader: BufReader<OwnedReadHalf>,
    mut writer: OwnedWriteHalf,
    secret: SharedSecret
) -> Result<()> {
    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin);

    let key = Key::<Aes256Gcm>::from_slice(secret.as_bytes());
    let cipher = Aes256Gcm::new(&key);

    let read_task = async {
        let mut buf = String::new();
        loop {
            buf.clear();

            match stream_reader.read_line(&mut buf).await {
                Ok(0) => {
                    println!("Connection closed.");
                    break;
                },
                Ok(_) => {
                    let ciphertext = &buf[12..];
                    let plaintext = std::str::from_utf8(
                        cipher.decrypt(nonce, ciphertext)?
                    );
                    print!("\rFriend: {}", plaintext);
                    print!("\rYou: ");
                    std::io::stdout().flush()?;
                },
                Err(e) => {
                    eprintln!("Error when reading from the other side: {}", e);
                    break;
                }
            }
        }

        Ok::<(), anyhow::Error>(())
    };

    let write_task = async {
        let mut input = String::new();
        loop {
            let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
            let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref())?;

            print!("\rYou: ");
            std::io::stdout().flush()?;

            input.clear();
            stdin_reader.read_line(&mut input).await?;

            if input.trim() == "quit" { break }

            let encrypted_input =
            writer.write_all(input.as_bytes()).await?;
            writer.flush().await?;
        }

        Ok::<(), anyhow::Error>(())
    };

    tokio::try_join!(read_task, write_task)?;

    Ok(())
}
