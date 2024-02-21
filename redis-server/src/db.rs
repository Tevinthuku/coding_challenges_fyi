use bytes::Bytes;
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    io,
    sync::{Arc, RwLock},
    time::{Duration, Instant, SystemTime},
};
use tokio::sync::Notify;

#[derive(Debug, Clone)]
pub struct Db {
    inner: Arc<DbInner>,
}

impl Drop for Db {
    fn drop(&mut self) {
        self.inner.shutdown_purge_task();
    }
}

#[derive(Debug)]
struct DbInner {
    data: RwLock<Data>,
    background_task: Notify,
}

#[derive(Debug)]
struct Data {
    inner: HashMap<String, Bytes>,
    expiry: BTreeSet<(Instant, String)>,
    // Since Instant is an opaque type, we cannot serialize it directly and save it to disk
    // this is why we maintain a separate hashMap to store the expiry time in DateTime<Utc> format.
    // When loading the stored data from disk, we can convert the DateTime<Utc> to an Instant.
    _expiry_serializable: HashMap<String, DateTime<Utc>>,
    shutdown: bool,
}

impl Default for Db {
    fn default() -> Self {
        let db_inner = DbInner {
            data: RwLock::new(Data {
                inner: HashMap::new(),
                expiry: BTreeSet::new(),
                _expiry_serializable: HashMap::new(),
                shutdown: false,
            }),
            background_task: Notify::new(),
        };
        let inner = Arc::new(db_inner);
        tokio::spawn(purge_expired_tasks(inner.clone()));
        Self { inner }
    }
}

impl Db {
    pub fn new() -> io::Result<Self> {
        SerializableState::restore_db_from_file()
    }

    fn new_with_data_mut(
        data: HashMap<String, Bytes>,
        expiry: BTreeSet<(Instant, String)>,
        _expiry_serializable: HashMap<String, DateTime<Utc>>,
    ) -> Self {
        let db_inner = DbInner {
            data: RwLock::new(Data {
                inner: data,
                expiry,
                _expiry_serializable,
                shutdown: false,
            }),
            background_task: Notify::new(),
        };
        let inner = Arc::new(db_inner);
        tokio::spawn(purge_expired_tasks(inner.clone()));
        Self { inner }
    }

    /// Useful for read access. Access to data is under a shared access lock.
    pub fn with_data<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&HashMap<String, Bytes>) -> T,
    {
        f(&self.inner.data.read().unwrap().inner)
    }

    pub fn with_data_mut<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&mut HashMap<String, Bytes>) -> T,
    {
        f(&mut self.inner.data.write().unwrap().inner)
    }

    /// The integer result from the closure is the new value for the key
    pub fn with_integer_data_mut<F>(&self, key: String, f: F) -> io::Result<i64>
    where
        F: FnOnce(i64) -> i64,
    {
        self.with_data_mut(|data| {
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
        self.with_data_mut(|data| {
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

    /// returns the previous value for the key if it existed.
    pub fn set(&self, key: String, value: Bytes, expire: Option<Duration>) -> Option<Bytes> {
        let mut state = self.inner.data.write().unwrap();

        let mut notify = false;

        let expiry_data = expire.map(|duration| {
            let when = Instant::now() + duration;
            let system_time = SystemTime::now() + duration;
            let utc_time = DateTime::<Utc>::from(system_time);
            notify = state
                .expiry
                .iter()
                .next()
                .map(|(current, _)| *current > when)
                .unwrap_or(true);

            (when, utc_time)
        });

        let previous_value = state.inner.insert(key.clone(), value);

        if let Some(_previous_value) = &previous_value {
            if let Some(expires_at) = expiry_data.map(|data| data.0) {
                state.expiry.remove(&(expires_at, key.clone()));
            }
        }

        if let Some(when) = expiry_data.map(|data| data.0) {
            state.expiry.insert((when, key.clone()));
        }

        if let Some(date) = expiry_data.map(|data| data.1) {
            state._expiry_serializable.insert(key, date);
        }

        drop(state);

        if notify {
            self.inner.background_task.notify_one();
        }

        previous_value
    }

    pub fn save(&self) -> io::Result<()> {
        SerializableState::save_to_file(self)
    }
}

impl DbInner {
    fn is_shutdown(&self) -> bool {
        self.data.read().unwrap().shutdown
    }

    /// Signals the purge background task to shut down. This is called by the
    /// `Db`s `Drop` implementation.
    fn shutdown_purge_task(&self) {
        let mut state = self.data.write().unwrap();
        state.shutdown = true;
        drop(state);

        self.background_task.notify_one();
    }

    fn purge_expired_keys(&self) -> Option<Instant> {
        let mut data = self.data.write().unwrap();
        if data.shutdown {
            return None;
        }
        let now = Instant::now();

        while let Some((when, key)) = data.expiry.iter().next().cloned() {
            if when > now {
                return Some(when);
            }

            data.inner.remove(&key);
            data._expiry_serializable.remove(&key);
            data.expiry.remove(&(when, key));
        }

        None
    }
}

async fn purge_expired_tasks(shared: Arc<DbInner>) {
    while !shared.is_shutdown() {
        if let Some(when) = shared.purge_expired_keys() {
            tokio::select! {
                _ = tokio::time::sleep_until(when.into()) => {}
                _ = shared.background_task.notified() => {}
            }
        } else {
            shared.background_task.notified().await;
        }
    }

    debug!("Background task is shutting down");
}

#[derive(Serialize, Deserialize)]
struct SerializableState {
    inner: HashMap<String, Bytes>,
    expiry: HashMap<String, DateTime<Utc>>,
}

const FILE: &str = "db.json";

impl SerializableState {
    fn save_to_file(db: &Db) -> io::Result<()> {
        let data = db.inner.data.read().unwrap();
        let expiry = data
            ._expiry_serializable
            .iter()
            .map(|(key, date)| (key.clone(), *date))
            .collect();
        let content = Self {
            inner: data.inner.clone(),
            expiry,
        };

        serde_json::to_writer(std::fs::File::create(FILE)?, &content)?;

        Ok(())
    }

    fn restore_db_from_file() -> io::Result<Db> {
        let file = std::fs::File::open(FILE);
        if let Err(err) = file {
            if err.kind() == io::ErrorKind::NotFound {
                return Ok(Db::default());
            }
            return Err(err);
        }
        let reader = std::io::BufReader::new(file?);
        let content: SerializableState = serde_json::from_reader(reader)?;
        let (expiry, date_time) = generate_expiry_and_date_time(content.expiry);
        Ok(Db::new_with_data_mut(content.inner, expiry, date_time))
    }
}

type Expiry = BTreeSet<(Instant, String)>;
type SerializableExpiry = HashMap<String, DateTime<Utc>>;
fn generate_expiry_and_date_time(
    expiry: HashMap<String, DateTime<Utc>>,
) -> (Expiry, SerializableExpiry) {
    let mut expiry_set = BTreeSet::new();
    let mut valid_inner_expiry = HashMap::new();
    let time_now = SystemTime::now();
    let instant_now = Instant::now();
    for (key, date) in expiry {
        // this will only return an error if the date has already passed
        let duration = SystemTime::from(date).duration_since(time_now);
        if let Ok(duration) = duration {
            let when = instant_now + duration;
            expiry_set.insert((when, key.clone()));
            valid_inner_expiry.insert(key, date);
        }
    }
    (expiry_set, valid_inner_expiry)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bytes::Bytes;

    #[tokio::test]
    async fn test_key_expiry() {
        let db = super::Db::new().unwrap();
        let value = Bytes::from("value");
        db.set(
            "key".to_owned(),
            value.clone(),
            Some(Duration::from_secs(1)),
        );
        let result = db.with_data_mut(|data| data.get("key").cloned()).unwrap();
        assert_eq!(result, value);
        tokio::time::sleep(Duration::from_secs(2)).await;
        let result = db.with_data_mut(|data| data.get("key").cloned());
        assert_eq!(result, None);
    }
}
