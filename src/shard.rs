use tokio::sync::mpsc;

use crate::{pipe::Request, storage::StorageTrait};

#[derive(Debug)]
pub enum ShardCmd {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
    List,
}

#[derive(Debug)]
pub enum ShardResponse {
    List(Vec<String>),
    Value(Option<String>),
    Ok,
    Busy,
    Err(String),
}

#[derive(Debug, Clone)]
pub struct Shard {
    pub shard_id: String,
    tx: mpsc::Sender<Request<ShardCmd, ShardResponse>>,
}

impl Shard {
    pub fn new(shard_id: String, tx: mpsc::Sender<Request<ShardCmd, ShardResponse>>) -> Self {
        Shard { shard_id, tx }
    }

    pub fn get_name(&self) -> &str {
        &self.shard_id
    }

    pub async fn send(&self, cmd: ShardCmd) -> ShardResponse {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let request = Request {
            msg: cmd,
            resp: resp_tx,
        };

        match self.tx.try_send(request) {
            Ok(_) => resp_rx
                .await
                .unwrap_or(ShardResponse::Err("failed to receive response".to_string())),
            Err(mpsc::error::TrySendError::Full(_req)) => ShardResponse::Busy,
            Err(_) => ShardResponse::Err("failed to send request".to_string()),
        }
    }
}

pub async fn handle_shard_command(
    storage: impl StorageTrait,
    mut rx: mpsc::Receiver<Request<ShardCmd, ShardResponse>>,
) {
    while let Some(Request { msg, resp }) = rx.recv().await {
        let response = match msg {
            ShardCmd::Get { key } => {
                let value = storage.get(key);
                ShardResponse::Value(value)
            }
            ShardCmd::Set { key, value } => {
                storage.set(key, value);
                ShardResponse::Ok
            }
            ShardCmd::Delete { key } => {
                storage.delete(key);
                ShardResponse::Ok
            }
            ShardCmd::List => {
                let keys = storage.list();
                ShardResponse::List(keys)
            }
        };
        let _ = resp.send(response);
    }
}
