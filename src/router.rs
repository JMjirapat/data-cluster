use std::{cell::RefCell, sync::Arc};

use tokio::sync::{RwLock, mpsc::Receiver, oneshot};

use crate::{
    cluster::ClusterTrait,
    pipe::Request,
    storage::{StorageCmd, StorageResponse, StorageTrait},
};

#[derive(Debug, Clone)]
pub enum RouterRequest {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
    List,
}

#[derive(Debug, Clone)]
pub enum RouterResponse {
    Value(String),
    List(Vec<String>),
    None,
    Ok,
    Err(String),
}

// Add generic parameter S for StorageTrait
pub async fn handle_router_command<C, S>(
    mut rx: Receiver<Request<RouterRequest, RouterResponse>>,
    ring: Arc<RwLock<C>>,
) where
    C: ClusterTrait<S>,
    S: StorageTrait,
{
    while let Some(Request { msg, resp }) = rx.recv().await {
        match msg {
            RouterRequest::Get { key } => {
                if let Some(storage) = ring.read().await.get(&key) {
                    let (tx, rx) = oneshot::channel();
                    let request = Request {
                        msg: StorageCmd::Get { key: key.clone() },
                        resp: tx,
                    };

                    match storage.get_tx().try_send(request) {
                        Ok(_) => {}
                        Err(e) => {
                            let _ = resp.send(RouterResponse::Err(e.to_string()));
                            continue;
                        }
                    }
                    match rx.await {
                        Ok(response) => match response {
                            StorageResponse::Value(value) => match value {
                                Some(v) => {
                                    let _ = resp.send(RouterResponse::Value(v));
                                }
                                None => {
                                    let _ = resp.send(RouterResponse::None);
                                }
                            },
                            _ => {
                                let _ = resp
                                    .send(RouterResponse::Err("Unexpected response".to_string()));
                            }
                        },
                        Err(_) => {
                            let _ = resp.send(RouterResponse::Err(
                                "Failed to receive response".to_string(),
                            ));
                        }
                    }
                } else {
                    let _ = resp.send(RouterResponse::Err("Storage not found".to_string()));
                }
            }
            RouterRequest::Set { key, value } => {
                if let Some(storage) = ring.read().await.get(&key) {
                    let (tx, rx) = oneshot::channel();
                    let request = Request {
                        msg: StorageCmd::Set { key, value },
                        resp: tx,
                    };

                    match storage.get_tx().try_send(request) {
                        Ok(_) => {}
                        Err(e) => {
                            let _ = resp.send(RouterResponse::Err(e.to_string()));
                            continue;
                        }
                    }
                    match rx.await {
                        Ok(StorageResponse::Ok) => {
                            let _ = resp.send(RouterResponse::Ok);
                        }
                        _ => {
                            let _ =
                                resp.send(RouterResponse::Err("Unexpected response".to_string()));
                        }
                    }
                } else {
                    let _ = resp.send(RouterResponse::Err("Storage not found".to_string()));
                }
            }
            RouterRequest::Delete { key } => {
                if let Some(storage) = ring.read().await.get(&key) {
                    let (tx, rx) = oneshot::channel();
                    let request = Request {
                        msg: StorageCmd::Delete { key },
                        resp: tx,
                    };

                    match storage.get_tx().try_send(request) {
                        Ok(_) => {}
                        Err(e) => {
                            let _ = resp.send(RouterResponse::Err(e.to_string()));
                            continue;
                        }
                    }
                    match rx.await {
                        Ok(StorageResponse::Ok) => {
                            let _ = resp.send(RouterResponse::Ok);
                        }
                        _ => {
                            let _ =
                                resp.send(RouterResponse::Err("Unexpected response".to_string()));
                        }
                    }
                } else {
                    let _ = resp.send(RouterResponse::Err("Storage not found".to_string()));
                }
            }
            RouterRequest::List => {
                let nodes = ring.read().await.nodes();
                let keys = RefCell::new(Vec::new());
                let mut response = RouterResponse::List(keys.borrow().clone());
                for storage in nodes {
                    let (tx, rx) = oneshot::channel();
                    let request = Request {
                        msg: StorageCmd::List,
                        resp: tx,
                    };
                    match storage.get_tx().try_send(request) {
                        Ok(_) => {}
                        Err(e) => {
                            response = RouterResponse::Err(e.to_string());
                            break;
                        }
                    }
                    match rx.await {
                        Ok(StorageResponse::List(Some(node_keys))) => {
                            keys.borrow_mut().extend(node_keys);
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                match response {
                    RouterResponse::List(keys) => {
                        let _ = resp.send(RouterResponse::List(keys));
                    }
                    _ => {
                        let _ = resp.send(response);
                    }
                }
            }
        }
    }
}
