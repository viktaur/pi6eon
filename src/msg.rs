use std::io::Write;

use tokio::net::TcpStream;
use tokio::io::{self, BufReader, AsyncBufReadExt, AsyncWriteExt};
use anyhow::Result;

/// Starts two asynchronous tasks which read from and write to a stream.
pub async fn messaging_loop(stream: TcpStream) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut stream_reader = BufReader::new(reader);

    let stdin = io::stdin();
    let mut stdin_reader = BufReader::new(stdin);

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
                    print!("\rFriend: {}", buf);
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
            print!("\rYou: ");
            std::io::stdout().flush()?;

            input.clear();
            stdin_reader.read_line(&mut input).await?;

            if input.trim() == "quit" { break }

            writer.write_all(input.as_bytes()).await?;
            writer.flush().await?;
        }

        Ok::<(), anyhow::Error>(())
    };

    tokio::try_join!(read_task, write_task)?;

    Ok(())
}
