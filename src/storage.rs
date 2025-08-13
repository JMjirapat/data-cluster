use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

pub trait StorageTrait {
    fn get_name(&self) -> String;
    fn get(&self, key: String) -> Option<String>;
    fn set(&self, key: String, value: String);
    fn delete(&self, key: String);
    fn list(&self) -> Vec<String>;
}

impl StorageTrait for Storage {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get(&self, key: String) -> Option<String> {
        let data = self.data.borrow();
        data.get(&key).cloned()
    }

    fn set(&self, key: String, value: String) {
        self.data.borrow_mut().insert(key, value);
    }

    fn delete(&self, key: String) {
        self.data.borrow_mut().remove(&key);
    }

    fn list(&self) -> Vec<String> {
        let data = self.data.borrow();
        data.keys().cloned().collect()
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

pub fn handle_storage_command(cmd: StorageCmd, storage: Rc<Storage>) -> StorageResponse {
    match cmd {
        StorageCmd::Get { key } => {
            let value = storage.get(key);
            StorageResponse::Value(value)
        }
        StorageCmd::Set { key, value } => {
            storage.set(key, value);
            StorageResponse::Ok
        }
        StorageCmd::Delete { key } => {
            storage.delete(key);
            StorageResponse::Ok
        }
        StorageCmd::List => {
            let keys: Vec<String> = storage.list();
            StorageResponse::List(Some(keys))
        }
    }
}
