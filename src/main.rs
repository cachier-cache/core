use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncBufReadExt;
use tokio::io::WriteHalf;
use tokio::io::{split, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, serde::Deserialize)]
struct Command {
    command: String,
    key: String,
    value: Option<String>,
    ttl: Option<i64>,
}

#[derive(Debug)]
struct Hash {
    value: String,
    exp: Option<i64>,
}

async fn write_to_stream(
    stream: &mut WriteHalf<TcpStream>,
    data: String,
) -> Result<(), std::io::Error> {
    stream.write_all(data.as_bytes()).await
}

async fn handle_client(stream: TcpStream, map: Arc<Mutex<HashMap<String, Hash>>>) {
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

                let mut response = HashMap::<String, String>::new();

                let command: Command = match serde_json::from_str(&buffer) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        eprintln!("Failed to parse JSON; err = {:?}", e);
                        response.insert("status".to_string(), "error".to_string());
                        response.insert("message".to_string(), e.to_string());
                        if let Err(e) =
                            write_to_stream(&mut writer, serde_json::to_string(&response).unwrap())
                                .await
                        {
                            eprintln!("failed to write to socket; err = {:?}", e);
                        }

                        continue;
                    }
                };

                {
                    let mut map = map.lock().unwrap();
                    match command.command.as_str() {
                        "set" => {
                            if let Some(value) = &command.value {
                                let exp = match command.ttl {
                                    Some(ttl) => Some(ttl + chrono::Utc::now().timestamp()),
                                    None => None,
                                };

                                let new_hash = Hash {
                                    value: value.clone(),
                                    exp,
                                };

                                map.insert(command.key.clone(), new_hash);
                                response.insert("status".to_string(), "ok".to_string());
                            } else {
                                eprintln!("'set' command requires 'value' field");
                                response.insert("status".to_string(), "error".to_string());
                                response.insert(
                                    "message".to_string(),
                                    "'set' command requires 'value' field".to_string(),
                                );
                                // TODO: i need to send the response here
                                continue;
                            }
                        }
                        "get" => {
                            let key = &command.key;
                            let value = map.get(key);
                            response.insert("status".to_string(), "ok".to_string());
                            match value {
                                Some(v) => {
                                    let now = chrono::Utc::now().timestamp();
                                    let value = match v.exp {
                                        Some(exp) => {
                                            if exp < now {
                                                "".to_string()
                                            } else {
                                                v.value.to_string()
                                            }
                                        }
                                        None => v.value.to_string(),
                                    };
                                    response.insert("value".to_string(), value);
                                }
                                None => {
                                    response.insert("value".to_string(), "".to_string());
                                }
                            }
                        }
                        _ => {
                            eprintln!("Unknown command: {}", command.command);
                        }
                    }

                    println!("current map: {:?}", map);
                }

                if let Err(e) = writer
                    .write_all(serde_json::to_string(&response).unwrap().as_bytes())
                    .await
                {
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

    let map = Arc::new(Mutex::new(HashMap::<String, Hash>::new()));
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
