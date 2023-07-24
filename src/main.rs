use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncBufReadExt;
use tokio::io::{split, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn handle_client(stream: TcpStream, map: Arc<Mutex<HashMap<String, String>>>) {
    let (reader, writer) = split(stream);
    let mut reader = BufReader::new(reader);
    let mut writer = writer;

    let mut buffer = String::new();
    loop {
        match reader.read_line(&mut buffer).await {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                {
                    let mut map = map.lock().unwrap();
                    match serde_json::from_str::<HashMap<String, String>>(&buffer) {
                        Ok(kv) => {
                            map.extend(kv);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse JSON; err = {:?}", e);
                        }
                    }

                    println!("current map: {:?}", map);
                }

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

    let map = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    println!("current map: {:?}", map.lock().unwrap());

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                println!("New connection!");
                let map = Arc::clone(&map);
                tokio::spawn(handle_client(stream, map));
            }
            Err(e) => {
                eprintln!("Failed to accept connection; error = {:?}", e);
            }
        }
    }
}
