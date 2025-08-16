use std::sync::Arc;

use data_cluster::pipe;
use data_cluster::router::{RouterRequest, RouterResponse, handle_router_command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use data_cluster::cluster::{ClusterTrait, HashRing};
use data_cluster::storage::{Storage, StorageCmd, StorageResponse};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let ring = Arc::new(tokio::sync::RwLock::new(HashRing::new(64)));

    for i in 0..3 {
        let (tx, rx) = pipe::channel::<StorageCmd, StorageResponse>(1024);
        let storage = Arc::new(Storage::new(format!("storage_{}", i), tx));
        ring.write().await.add_node(storage.clone());
    }

    let (router_tx, router_rx) = pipe::channel::<RouterRequest, RouterResponse>(2048);
    let ring_clone = Arc::clone(&ring);
    tokio::spawn(async move {
        handle_router_command(router_rx, ring_clone).await;
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
                            let (tx, rx) = oneshot::channel();
                            let request = RouterRequest::Get {
                                key: key.to_string(),
                            };
                            router_tx
                                .send(pipe::Request {
                                    msg: request,
                                    resp: tx,
                                })
                                .await
                                .unwrap();
                            match rx.await {
                                Ok(RouterResponse::Value(value)) => {
                                    let _ = writer.write_all(format!("{value}\n").as_bytes()).await;
                                }
                                Ok(RouterResponse::None) => {
                                    let _ = writer.write_all(b"not found\n").await;
                                }
                                Ok(RouterResponse::Err(err)) => {
                                    let _ =
                                        writer.write_all(format!("ERR {err}\n").as_bytes()).await;
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
                            let (tx, rx) = oneshot::channel();
                            let request = RouterRequest::Set {
                                key: key.to_string(),
                                value: value.to_string(),
                            };
                            router_tx
                                .send(pipe::Request {
                                    msg: request,
                                    resp: tx,
                                })
                                .await
                                .unwrap();
                            match rx.await {
                                Ok(RouterResponse::Ok) => {
                                    let _ = writer.write_all(b"OK\n").await;
                                }
                                Ok(RouterResponse::Err(err)) => {
                                    let _ =
                                        writer.write_all(format!("ERR {err}\n").as_bytes()).await;
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
                            let (tx, rx) = oneshot::channel();
                            let request = RouterRequest::Delete {
                                key: key.to_string(),
                            };
                            router_tx
                                .send(pipe::Request {
                                    msg: request,
                                    resp: tx,
                                })
                                .await
                                .unwrap();
                            match rx.await {
                                Ok(RouterResponse::Ok) => {
                                    let _ = writer.write_all(b"OK\n").await;
                                }
                                Ok(RouterResponse::Err(err)) => {
                                    let _ =
                                        writer.write_all(format!("ERR {err}\n").as_bytes()).await;
                                }
                                _ => {
                                    let _ = writer.write_all(b"ERR unexpected response\n").await;
                                }
                            }
                        } else {
                            let _ = writer.write_all(b"ERR missing key or value\n").await;
                        }
                    }
                    "LIST" => {
                        let (tx, rx) = oneshot::channel();
                        let request = RouterRequest::List;
                        router_tx
                            .send(pipe::Request {
                                msg: request,
                                resp: tx,
                            })
                            .await
                            .unwrap();
                        match rx.await {
                            Ok(RouterResponse::List(keys)) => {
                                let response = keys.join(", ");
                                let _ = writer.write_all(format!("{response}\n").as_bytes()).await;
                            }
                            Ok(RouterResponse::None) => {
                                let _ = writer.write_all(b"no keys found\n").await;
                            }
                            Ok(RouterResponse::Err(err)) => {
                                let _ = writer.write_all(format!("ERR {err}\n").as_bytes()).await;
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
