use std::collections::HashMap;
use tokio::sync::{Mutex, mpsc};

use crate::pipe::Request;

#[derive(Debug)]
pub enum StorageCmd {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
    List,
}

#[derive(Debug)]
pub enum StorageResponse {
    List(Option<Vec<String>>),
    Value(Option<String>),
    Ok,
}

#[derive(Debug)]
pub struct Storage {
    pub name: String,
    pub tx: mpsc::Sender<Request<StorageCmd, StorageResponse>>,
    pub data: Mutex<HashMap<String, String>>,
}

pub trait StorageTrait {
    fn get_name(&self) -> String;
    fn get_tx(&self) -> mpsc::Sender<Request<StorageCmd, StorageResponse>>;
    async fn get(&self, key: String) -> Option<String>;
    async fn set(&self, key: String, value: String);
    async fn delete(&self, key: String);
    async fn list(&self) -> Vec<String>;
}

impl StorageTrait for Storage {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_tx(&self) -> mpsc::Sender<Request<StorageCmd, StorageResponse>> {
        self.tx.clone()
    }

    async fn get(&self, key: String) -> Option<String> {
        self.data.lock().await.get(&key).cloned()
    }

    async fn set(&self, key: String, value: String) {
        self.data.lock().await.insert(key, value);
    }

    async fn delete(&self, key: String) {
        self.data.lock().await.remove(&key);
    }

    async fn list(&self) -> Vec<String> {
        self.data.lock().await.keys().cloned().collect()
    }
}

impl Storage {
    pub fn new(name: String, tx: mpsc::Sender<Request<StorageCmd, StorageResponse>>) -> Self {
        Storage {
            name,
            tx,
            data: Mutex::new(HashMap::new()),
        }
    }
}

pub async fn handle_storage_command(
    storage: Storage,
    mut rx: mpsc::Receiver<Request<StorageCmd, StorageResponse>>,
) {
    while let Some(Request { msg, resp }) = rx.recv().await {
        let response = match msg {
            StorageCmd::Get { key } => {
                let value = storage.get(key);
                StorageResponse::Value(value.await)
            }
            StorageCmd::Set { key, value } => {
                storage.set(key, value).await;
                StorageResponse::Ok
            }
            StorageCmd::Delete { key } => {
                storage.delete(key).await;
                StorageResponse::Ok
            }
            StorageCmd::List => {
                let keys = storage.list();
                StorageResponse::List(Some(keys.await))
            }
        };
        let _ = resp.send(response);
    }
}
