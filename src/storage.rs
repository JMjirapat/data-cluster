use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

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
    pub data: Mutex<HashMap<String, String>>,
}

#[async_trait::async_trait]
pub trait StorageTrait {
    fn get_name(&self) -> String;
    async fn get(&self, key: String) -> Option<String>;
    async fn set(&self, key: String, value: String);
    async fn delete(&self, key: String);
    async fn list(&self) -> Vec<String>;
}

#[async_trait::async_trait]
impl StorageTrait for Storage {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    async fn get(&self, key: String) -> Option<String> {
        let data = self.data.lock().await;
        data.get(&key).cloned()
    }

    async fn set(&self, key: String, value: String) {
        let mut data = self.data.lock().await;
        data.insert(key, value);
    }

    async fn delete(&self, key: String) {
        let mut data = self.data.lock().await;
        data.remove(&key);
    }

    async fn list(&self) -> Vec<String> {
        let data = self.data.lock().await;
        data.keys().cloned().collect()
    }
}

impl Storage {
    pub fn new(name: String) -> Self {
        Storage {
            name,
            data: Mutex::new(HashMap::new()),
        }
    }
}

pub async fn handle_storage_command(cmd: StorageCmd, storage: Arc<Storage>) -> StorageResponse {
    match cmd {
        StorageCmd::Get { key } => {
            let value = storage.get(key).await;
            StorageResponse::Value(value)
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
            let keys: Vec<String> = storage.list().await;
            StorageResponse::List(Some(keys))
        }
    }
}
