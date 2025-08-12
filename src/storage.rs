use std::{cell::RefCell, collections::HashMap, fmt::Display};

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

#[derive(Debug, Clone)]
pub struct Storage {
    pub name: String,
    pub data: RefCell<HashMap<String, String>>,
}

impl Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Storage {
    pub fn new(name: String) -> Self {
        Storage {
            name,
            data: RefCell::new(HashMap::new()),
        }
    }
}

pub fn handle_storage_command(cmd: StorageCmd, storage: &Storage) -> StorageResponse {
    match cmd {
        StorageCmd::Get { key } => {
            let value = storage.data.borrow().get(&key).cloned();
            StorageResponse::Value(value)
        }
        StorageCmd::Set { key, value } => {
            storage.data.borrow_mut().insert(key, value);
            StorageResponse::Ok
        }
        StorageCmd::Delete { key } => {
            storage.data.borrow_mut().remove(&key);
            StorageResponse::Ok
        }
        StorageCmd::List => {
            let keys: Vec<String> = storage.data.borrow().keys().cloned().collect();
            StorageResponse::List(Some(keys))
        }
    }
}
