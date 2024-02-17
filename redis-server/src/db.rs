use bytes::Bytes;
use std::{
    collections::HashMap,
    io,
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

    pub fn with_integer<F>(&self, key: String, f: F) -> io::Result<i64>
    where
        F: FnOnce(i64, &mut HashMap<String, Bytes>) -> i64,
    {
        self.with_data(|data| {
            let entry = data.entry(key.clone());
            let new_val = match entry {
                std::collections::hash_map::Entry::Occupied(mut val) => {
                    let value = val.get_mut();
                    let value = String::from_utf8(value.to_vec())
                        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
                    match value.parse::<i64>() {
                        Ok(value) => f(value, data),
                        Err(_) => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                "Value is not an integer",
                            ))
                        }
                    }
                }
                std::collections::hash_map::Entry::Vacant(_) => f(0, data),
            };

            Ok(new_val)
        })
    }
}
