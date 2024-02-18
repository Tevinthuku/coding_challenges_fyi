use bytes::Bytes;
use log::debug;
use std::{
    collections::{BTreeSet, HashMap},
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
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
    data: Mutex<Data>,
    background_task: Notify,
}

#[derive(Debug)]
struct Data {
    inner: HashMap<String, Bytes>,
    /// A set of keys that have an expiry time
    expiry: BTreeSet<(Instant, String)>,
    /// True when the Db instance is shutting down. This happens when all `Db`
    /// values drop. Setting this to `true` signals to the background task to
    /// exit.
    shutdown: bool,
}

impl Default for Db {
    fn default() -> Self {
        let db_inner = DbInner {
            data: Mutex::new(Data {
                inner: HashMap::new(),
                expiry: BTreeSet::new(),
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
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_data<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&mut HashMap<String, Bytes>) -> T,
    {
        f(&mut self.inner.data.lock().unwrap().inner)
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

    /// returns the previous value for the key if it existed.
    pub fn set(&self, key: String, value: Bytes, expire: Option<Duration>) -> Option<Bytes> {
        let mut state = self.inner.data.lock().unwrap();

        let mut notify = false;

        let expires_at = expire.map(|duration| {
            let when = Instant::now() + duration;

            notify = state
                .expiry
                .iter()
                .next()
                .map(|(current, _)| *current > when)
                .unwrap_or(true);

            when
        });

        let previous_value = state.inner.insert(key.clone(), value);

        if let Some(_previous_value) = &previous_value {
            if let Some(expires_at) = expires_at {
                state.expiry.remove(&(expires_at, key.clone()));
            }
        }

        if let Some(when) = expires_at {
            state.expiry.insert((when, key));
        }

        drop(state);

        if notify {
            // Finally, only notify the background task if it needs to update
            // its state to reflect a new expiration.
            self.inner.background_task.notify_one();
        }

        previous_value
    }
}

impl DbInner {
    fn is_shutdown(&self) -> bool {
        self.data.lock().unwrap().shutdown
    }

    /// Signals the purge background task to shut down. This is called by the
    /// `Db`s `Drop` implementation.
    fn shutdown_purge_task(&self) {
        // The background task must be signaled to shut down. This is done by
        // setting `Data::shutdown` to `true` and signalling the task.
        let mut state = self.data.lock().unwrap();
        state.shutdown = true;

        // Drop the lock before signalling the background task. This helps
        // reduce lock contention by ensuring the background task doesn't
        // wake up only to be unable to acquire the mutex.
        drop(state);

        self.background_task.notify_one();
    }

    fn purge_expired_keys(&self) -> Option<Instant> {
        let mut data = self.data.lock().unwrap();
        if data.shutdown {
            // The database is shutting down. All handles to the shared state
            // have dropped. The background task should exit.
            return None;
        }
        let now = Instant::now();

        while let Some((when, key)) = data.expiry.iter().next().cloned() {
            if when > now {
                // Done purging, `when` is the instant at which the next key
                // expires. The worker task will wait until this instant.
                return Some(when);
            }

            data.inner.remove(&key);
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
            // There are no keys to expire, so we wait until we are notified
            shared.background_task.notified().await;
        }
    }

    debug!("Background task is shutting down");
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bytes::Bytes;

    #[tokio::test]
    async fn test_key_expiry() {
        let db = super::Db::new();
        let value = Bytes::from("value");
        db.set(
            "key".to_owned(),
            value.clone(),
            Some(Duration::from_secs(1)),
        );
        let result = db.with_data(|data| data.get("key").cloned()).unwrap();
        assert_eq!(result, value);
        tokio::time::sleep(Duration::from_secs(2)).await;
        let result = db.with_data(|data| data.get("key").cloned());
        assert_eq!(result, None);
    }
}
