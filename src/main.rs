use data_cluster::router::{RouterRequest, RouterResponse, handle_router_command};
use data_cluster::shard::{Shard, ShardCmd, ShardResponse, handle_shard_command};
use data_cluster::{pipe, router};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use data_cluster::cluster::{ClusterTrait, HashRing};
use data_cluster::storage::Storage;

#[tokio::main]
async fn main() {
    let mut ring = HashRing::new(64);

    for i in 0..3 {
        let (tx, rx) = pipe::channel::<ShardCmd, ShardResponse>(1024);
        let shard = Shard::new(format!("shard_{}", i), tx);
        ring.add_node(shard);
        tokio::spawn(async move {
            let storage = Storage::new();
            handle_shard_command(storage, rx).await;
        });
    }

    let (router_tx, router_rx) = pipe::channel::<RouterRequest, RouterResponse>(2048);
    tokio::spawn(async move {
        handle_router_command(ring, router_rx).await;
    });

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
        let router_tx = router_tx.clone();

        tokio::spawn(async move {
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
                            let response = router::send(
                                &router_tx,
                                RouterRequest::Get {
                                    key: key.to_string(),
                                },
                            )
                            .await;
                            match response {
                                RouterResponse::Value(value) => {
                                    let _ = writer.write_all(value.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                }
                                RouterResponse::None => {
                                    let _ = writer.write_all(b"ERR key not found\n").await;
                                }
                                RouterResponse::Err(err) => {
                                    let _ = writer.write_all(err.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                }
                                _ => {
                                    let _ = writer.write_all(b"ERR unexpected response\n").await;
                                }
                            }
                        } else {
                            let _ = writer.write_all(b"ERR missing key\n").await;
                        }
                    }
                    "SET" => {
                        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                            let response = router::send(
                                &router_tx,
                                RouterRequest::Set {
                                    key: key.to_string(),
                                    value: value.to_string(),
                                },
                            )
                            .await;
                            match response {
                                RouterResponse::Ok => {
                                    let _ = writer.write_all(b"OK\n").await;
                                }
                                RouterResponse::Err(err) => {
                                    let _ = writer.write_all(err.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                }
                                _ => {
                                    let _ = writer.write_all(b"ERR unexpected response\n").await;
                                }
                            }
                        } else {
                            let _ = writer.write_all(b"ERR missing key or value\n").await;
                        }
                    }
                    "DELETE" => {
                        if let Some(key) = parts.next() {
                            let response = router::send(
                                &router_tx,
                                RouterRequest::Delete {
                                    key: key.to_string(),
                                },
                            )
                            .await;
                            match response {
                                RouterResponse::Ok => {
                                    let _ = writer.write_all(b"OK\n").await;
                                }
                                RouterResponse::Err(err) => {
                                    let _ = writer.write_all(err.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                }
                                _ => {
                                    let _ = writer.write_all(b"ERR unexpected response\n").await;
                                }
                            }
                        } else {
                            let _ = writer.write_all(b"ERR missing key\n").await;
                        }
                    }
                    "LIST" => {
                        let response = router::send(&router_tx, RouterRequest::List).await;
                        match response {
                            RouterResponse::List(keys) => {
                                let keys_str = keys.join(", ");
                                let _ = writer.write_all(keys_str.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                            }
                            RouterResponse::Err(err) => {
                                let _ = writer.write_all(err.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                            }
                            _ => {
                                let _ = writer.write_all(b"ERR unexpected response\n").await;
                            }
                        }
                    }
                    _ => {
                        let _ = writer.write_all(b"ERR unknown command\n").await;
                    }
                }
            }
        });
    }
}
