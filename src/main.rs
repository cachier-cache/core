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
) -> Result<(), Box<dyn Error>> {
    stream.write_all(data.as_bytes()).await.map_err(Into::into)
}

async fn handle_client(
    stream: TcpStream,
    map: Arc<Mutex<HashMap<String, Hash>>>,
) -> Result<(), Box<dyn Error>> {
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
                        write_to_stream(&mut writer, serde_json::to_string(&response)?).await?;
                        buffer.clear();
                        continue;
                    }
                };

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

                            let mut map = map.lock().map_err(|_| "Mutex is poisoned")?;
                            map.insert(command.key.clone(), new_hash);
                            response.insert("status".to_string(), "ok".to_string());
                        } else {
                            eprintln!("'set' command requires 'value' field");
                            response.insert("status".to_string(), "error".to_string());
                            response.insert(
                                "message".to_string(),
                                "'set' command requires 'value' field".to_string(),
                            );
                            write_to_stream(&mut writer, serde_json::to_string(&response)?).await?;
                            buffer.clear();
                            continue;
                        }
                    }
                    "get" => {
                        let mut value_str = "".to_string();
                        {
                            let map = map.lock().map_err(|_| "Mutex is poisoned")?;
                            if let Some(value) = map.get(&command.key) {
                                let now = chrono::Utc::now().timestamp();
                                value_str = match value.exp {
                                    Some(exp) => {
                                        if exp < now {
                                            "".to_string()
                                        } else {
                                            value.value.to_string()
                                        }
                                    }
                                    None => value.value.to_string(),
                                };
                            }
                        }
                        response.insert("status".to_string(), "ok".to_string());
                        response.insert("value".to_string(), value_str);
                    }
                    _ => {
                        eprintln!("Unknown command: {}", command.command);
                    }
                }

                writer
                    .write_all(serde_json::to_string(&response)?.as_bytes())
                    .await?;
            }
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                break;
            }
        }
        buffer.clear();
    }

    println!("Connection closed.");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening...");

    let map = Arc::new(Mutex::new(HashMap::<String, Hash>::new()));
    println!("current map: {:?}", map.lock());

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                println!("New connection!");
                let map = Arc::clone(&map);
                tokio::spawn(async {
                    if let Err(e) = handle_client(stream, map).await {
                        eprintln!("Failed to handle client; error = {:?}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection; error = {:?}", e);
            }
        }
    }
}
