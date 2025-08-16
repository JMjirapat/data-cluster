use std::{cell::RefCell, collections::HashMap};

#[derive(Debug)]
pub struct Storage {
    pub data: RefCell<HashMap<String, String>>,
}

pub trait StorageTrait {
    fn get(&self, key: String) -> Option<String>;
    fn set(&self, key: String, value: String);
    fn delete(&self, key: String);
    fn list(&self) -> Vec<String>;
}

impl StorageTrait for Storage {
    fn get(&self, key: String) -> Option<String> {
        self.data.borrow().get(&key).cloned()
    }

    fn set(&self, key: String, value: String) {
        self.data.borrow_mut().insert(key, value);
    }

    fn delete(&self, key: String) {
        self.data.borrow_mut().remove(&key);
    }

    fn list(&self) -> Vec<String> {
        self.data.borrow().keys().cloned().collect()
    }
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            data: RefCell::new(HashMap::new()),
        }
    }
}
