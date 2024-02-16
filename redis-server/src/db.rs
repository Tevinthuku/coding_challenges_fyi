use bytes::Bytes;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct Db {
    data: Arc<Mutex<Data>>,
}

#[derive(Debug)]
struct Data {
    inner: HashMap<String, Bytes>,
}

impl Default for Db {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                inner: HashMap::new(),
            })),
        }
    }
}

impl Db {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_data<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&mut HashMap<String, Bytes>) -> T,
    {
        f(&mut self.data.lock().unwrap().inner)
    }
}
