use std::sync::Arc;

use tokio::sync::mpsc::{self, Receiver};

use crate::{
    cluster::ClusterTrait,
    pipe::Request,
    shard::{ShardCmd, ShardResponse},
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
    Busy,
    ShardBusy,
    Err(String),
}

pub async fn send(
    tx: &tokio::sync::mpsc::Sender<Request<RouterRequest, RouterResponse>>,
    request: RouterRequest,
) -> RouterResponse {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    let request = Request {
        msg: request,
        resp: resp_tx,
    };

    match tx.try_send(request) {
        Ok(_) => resp_rx.await.unwrap_or(RouterResponse::Err(
            "failed to receive response".to_string(),
        )),
        Err(mpsc::error::TrySendError::Full(_req)) => RouterResponse::Busy,
        Err(_) => RouterResponse::Err("failed to send request".to_string()),
    }
}

pub async fn handle_router_command(
    ring: Arc<dyn ClusterTrait + Send + Sync>,
    mut rx: Receiver<Request<RouterRequest, RouterResponse>>,
) {
    while let Some(Request { msg, resp }) = rx.recv().await {
        let ring = ring.clone();
        tokio::spawn(async move {
            match msg {
                RouterRequest::Get { key } => {
                    if let Some(shard) = ring.get(&key) {
                        let response = shard.send(ShardCmd::Get { key }).await;
                        let _ = resp.send(match response {
                            ShardResponse::Value(value) => match value {
                                Some(v) => RouterResponse::Value(v),
                                None => RouterResponse::None,
                            },
                            ShardResponse::Busy => RouterResponse::ShardBusy,
                            ShardResponse::Err(err) => RouterResponse::Err(err),
                            _ => RouterResponse::Err("Unexpected response".to_string()),
                        });
                    }
                }
                RouterRequest::Set { key, value } => {
                    if let Some(shard) = ring.get(&key) {
                        let response = shard.send(ShardCmd::Set { key, value }).await;
                        let _ = resp.send(match response {
                            ShardResponse::Ok => RouterResponse::Ok,
                            ShardResponse::Busy => RouterResponse::ShardBusy,
                            ShardResponse::Err(err) => RouterResponse::Err(err),
                            _ => RouterResponse::Err("Unexpected response".to_string()),
                        });
                    }
                }
                RouterRequest::Delete { key } => {
                    if let Some(shard) = ring.get(&key) {
                        let response = shard.send(ShardCmd::Delete { key }).await;
                        let _ = resp.send(match response {
                            ShardResponse::Ok => RouterResponse::Ok,
                            ShardResponse::Busy => RouterResponse::ShardBusy,
                            ShardResponse::Err(err) => RouterResponse::Err(err),
                            _ => RouterResponse::Err("Unexpected response".to_string()),
                        });
                    }
                }
                RouterRequest::List => {
                    let shards = ring.nodes();
                    let mut all_keys = Vec::new();
                    let mut error: Option<RouterResponse> = None;
                    for shard in shards {
                        let response = shard.send(ShardCmd::List).await;
                        match response {
                            ShardResponse::List(keys) => all_keys.extend(keys),
                            ShardResponse::Busy => {
                                error = Some(RouterResponse::Busy);
                                break;
                            }
                            ShardResponse::Err(err) => {
                                error = Some(RouterResponse::Err(err));
                                break;
                            }
                            _ => {
                                error =
                                    Some(RouterResponse::Err("Unexpected response".to_string()));
                                break;
                            }
                        }
                    }
                    let _ = match error {
                        Some(e) => resp.send(e),
                        None => resp.send(match all_keys.is_empty() {
                            true => RouterResponse::None,
                            false => RouterResponse::List(all_keys),
                        }),
                    };
                }
            }
        });
    }
}
