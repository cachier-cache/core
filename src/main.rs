use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncBufReadExt;
use tokio::io::{split, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, serde::Deserialize)]
struct Command {
    command: String,
    data: HashMap<String, String>,
}

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

                let command: Command = match serde_json::from_str(&buffer) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        eprintln!("Failed to parse JSON; err = {:?}", e);
                        continue;
                    }
                };

                let mut response = HashMap::<String, String>::new();

                {
                    let mut map = map.lock().unwrap();
                    match command.command.as_str() {
                        "set" => {
                            map.extend(command.data);
                            response.insert("status".to_string(), "ok".to_string());
                        },
                        "get" => {
                            let key = command.data.get("key").unwrap();
                            let value = map.get(key);
                            response.insert("status".to_string(), "ok".to_string());
                            match value {
                                Some(v) => {
                                    response.insert("value".to_string(), v.to_string());
                                },
                                None => {
                                    response.insert("value".to_string(), "".to_string());
                                }
                            }
                        },
                        _ => {
                            eprintln!("Unknown command: {}", command.command);
                        }
                    }

                    println!("current map: {:?}", map);
                }

                if let Err(e) = writer.write_all(
                    serde_json::to_string(&response).unwrap().as_bytes(),
                ).await {
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
