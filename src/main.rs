use tokio::io::AsyncBufReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, BufReader, split};
use std::error::Error;

async fn handle_client(stream: TcpStream) {
    let (reader, writer) = split(stream);
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    let mut buffer = String::new();
    loop {
        match reader.read_line(&mut buffer).await {
            Ok(n) => {
                // 0 bytes read means the client closed the connection.
                if n == 0 {
                    break;
                }

                // write what we read back to the client.
                if let Err(e) = writer.write_all(buffer.as_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                break;
            }
        }
        buffer.clear();
    }

    println!("Connection closed.");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:8080").await?;
    println!("Server listening...");

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                println!("New connection!");
                tokio::spawn(handle_client(stream));
            }
            Err(e) => {
                eprintln!("Failed to accept connection; error = {:?}", e);
            }
        }
    }
}
