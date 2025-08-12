use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use data_cluster::hashing::HashRing;
use data_cluster::storage::{Storage, StorageCmd, StorageResponse, handle_storage_command};

#[tokio::main]
async fn main() {
    let mut ring = HashRing::new(64);

    for i in 0..3 {
        let storage = Storage::new(format!("storage_{}", i));
        ring.add_node(&storage);
    }

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    println!(
        "Server listening on {addr} — commands: GET <key> | SET <key> <value> | DELETE <key> | LIST"
    );

    loop {
        let (mut socket, _) = listener
            .accept()
            .await
            .expect("Failed to accept connection");
        let (reader, mut writer) = socket.split();
        let mut lines = BufReader::new(reader).lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();
            let cmd = parts.next().unwrap_or("");

            match cmd.to_uppercase().as_str() {
                "GET" => {
                    if let Some(key) = parts.next() {
                        if let Some(storage) = ring.get(key) {
                            let response = handle_storage_command(
                                StorageCmd::Get {
                                    key: key.to_string(),
                                },
                                storage,
                            );
                            match response {
                                StorageResponse::Value(value) => {
                                    if let Some(val) = value {
                                        let _ = writer.write_all(val.as_bytes()).await;
                                        let _ = writer.write_all(b"\n").await;
                                    } else {
                                        let _ = writer.write_all(b"not found\n").await;
                                    }
                                }
                                _ => {
                                    let _ = writer.write_all(b"ERR unexpected response\n").await;
                                }
                            }
                        } else {
                            let _ = writer.write_all(b"ERR storage not found\n").await;
                        }
                    } else {
                        let _ = writer.write_all(b"ERR missing key\n").await;
                    }
                }
                "SET" => {
                    if let Some(key) = parts.next() {
                        if let Some(value) = parts.next() {
                            if let Some(storage) = ring.get(key) {
                                let response = handle_storage_command(
                                    StorageCmd::Set {
                                        key: key.to_string(),
                                        value: value.to_string(),
                                    },
                                    storage,
                                );
                                if let StorageResponse::Ok = response {
                                    let _ = writer.write_all(b"OK\n").await;
                                } else {
                                    let _ = writer.write_all(b"ERR unexpected response\n").await;
                                }
                            } else {
                                let _ = writer.write_all(b"ERR storage not found\n").await;
                            }
                        } else {
                            let _ = writer.write_all(b"ERR missing value\n").await;
                        }
                    } else {
                        let _ = writer.write_all(b"ERR missing key\n").await;
                    }
                }
                "DELETE" => {
                    if let Some(key) = parts.next() {
                        if let Some(storage) = ring.get(key) {
                            let response = handle_storage_command(
                                StorageCmd::Delete {
                                    key: key.to_string(),
                                },
                                storage,
                            );
                            if let StorageResponse::Ok = response {
                                let _ = writer.write_all(b"OK\n").await;
                            } else {
                                let _ = writer.write_all(b"ERR unexpected response\n").await;
                            }
                        } else {
                            let _ = writer.write_all(b"ERR storage not found\n").await;
                        }
                    } else {
                        let _ = writer.write_all(b"ERR missing key\n").await;
                    }
                }
                "LIST" => {
                    let mut keys = Vec::new();
                    for storage in ring.nodes() {
                        let response = handle_storage_command(StorageCmd::List, storage);
                        if let StorageResponse::List(Some(storage_keys)) = response {
                            keys.extend(storage_keys);
                        }
                    }
                    if keys.is_empty() {
                        let _ = writer.write_all(b"no keys\n").await;
                    } else {
                        let response = keys.join(", ");
                        let _ = writer.write_all(response.as_bytes()).await;
                        let _ = writer.write_all(b"\n").await;
                    }
                }
                _ => {
                    let _ = writer.write_all(b"ERR unknown command\n").await;
                }
            }
        }
    }
}
