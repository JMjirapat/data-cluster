use std::sync::Arc;

use tokio::sync::{RwLock, mpsc::Receiver};

use crate::{cluster::ClusterTrait, pipe::Request, storage::StorageTrait};

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
                    let response = storage.get(key).await;
                    if response.is_some() {
                        let _ = resp.send(RouterResponse::Value(response.unwrap()));
                    } else {
                        let _ = resp.send(RouterResponse::None);
                    }
                } else {
                    let _ = resp.send(RouterResponse::Err("Storage not found".to_string()));
                }
            }
            RouterRequest::Set { key, value } => {
                if let Some(storage) = ring.read().await.get(&key) {
                    storage.set(key, value).await;
                    let _ = resp.send(RouterResponse::Ok);
                } else {
                    let _ = resp.send(RouterResponse::Err("Storage not found".to_string()));
                }
            }
            RouterRequest::Delete { key } => {
                if let Some(storage) = ring.read().await.get(&key) {
                    storage.delete(key).await;
                    let _ = resp.send(RouterResponse::Ok);
                } else {
                    let _ = resp.send(RouterResponse::Err("Storage not found".to_string()));
                }
            }
            RouterRequest::List => {
                let mut all_keys = Vec::new();
                for storage in ring.read().await.nodes() {
                    let keys = storage.list().await;
                    all_keys.extend(keys);
                }
                if all_keys.is_empty() {
                    let _ = resp.send(RouterResponse::None);
                } else {
                    let _ = resp.send(RouterResponse::List(all_keys));
                }
            }
        }
    }
}
