use bytes::Bytes;
use std::{
    collections::HashMap,
    io::{self},
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

    /// The integer result from the closure is the new value for the key
    pub fn with_integer_data_mut<F>(&self, key: String, f: F) -> io::Result<i64>
    where
        F: FnOnce(i64) -> i64,
    {
        self.with_data(|data| {
            let entry = data.entry(key.clone());
            let new_val = match entry {
                std::collections::hash_map::Entry::Occupied(mut val) => {
                    let value = val.get_mut();

                    let value: i64 = serde_json::from_slice(value).map_err(|_| {
                        io::Error::new(io::ErrorKind::Other, "Value is not an integer")
                    })?;

                    f(value)
                }
                std::collections::hash_map::Entry::Vacant(_) => f(0),
            };

            data.insert(key, Bytes::from(format!("{}", new_val)));

            Ok(new_val)
        })
    }

    /// The result from the closure is the new value for the key
    pub fn with_list_data_mut<F>(&self, key: String, f: F) -> io::Result<Vec<Bytes>>
    where
        F: FnOnce(Vec<Bytes>) -> Vec<Bytes>,
    {
        self.with_data(|data| {
            let entry = data.entry(key.clone());
            let new_value = match entry {
                std::collections::hash_map::Entry::Occupied(val) => {
                    let existing_value = val.into_mut();

                    let content: Vec<Bytes> = serde_json::from_slice(existing_value)
                        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

                    f(content)
                }
                std::collections::hash_map::Entry::Vacant(_) => f(vec![]),
            };

            let result = serde_json::to_string(&new_value)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
            let result = Bytes::from(result);
            data.insert(key, result);
            Ok(new_value)
        })
    }
}
