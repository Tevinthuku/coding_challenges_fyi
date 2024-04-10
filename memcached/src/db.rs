use std::{collections::HashMap, sync::RwLock, time::Duration};

pub struct Db {
    data: RwLock<HashMap<String, Content>>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_data_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut HashMap<String, Content>) -> T,
    {
        f(&mut self.data.write().unwrap())
    }

    pub fn with_data<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&HashMap<String, Content>) -> T,
    {
        f(&self.data.read().unwrap())
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Content {
    pub data: Vec<u8>,
    pub byte_count: usize,
    pub flags: u32,
    pub exp_duration: Option<Duration>,
}

impl Content {
    pub fn is_expired(&self) -> bool {
        if let Some(exp_duration) = self.exp_duration {
            exp_duration.as_secs()
                < std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
        } else {
            false
        }
    }
}
