use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use linked_hash_map::LinkedHashMap;
use tokio::sync::Notify;

#[derive(Clone)]
pub struct Db {
    inner: Arc<DbInner>,
}

pub(crate) struct DbDropGuard {
    db: Db,
}

impl DbDropGuard {
    pub fn new(max_cache_size_in_bytes: u64) -> Self {
        Self {
            db: Db::new(max_cache_size_in_bytes),
        }
    }

    pub fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Drop for DbDropGuard {
    fn drop(&mut self) {
        self.db.signal_shut_down();
    }
}

// TODO: Setup a special HashMap that will allow us to store the data in a hashmap and also keep track of the oldest data to be removed once the cache size is above the threshold

struct DbInner {
    data: RwLock<DbState>,
    background_task: Notify,
}

impl DbInner {
    fn is_shutting_down(&self) -> bool {
        self.data.read().unwrap().shut_down
    }
}

struct DbState {
    max_cache_size_in_bytes: u64,
    shut_down: bool,
    entries: LinkedHashMap<String, Content>,
}

impl Db {
    pub fn new(max_cache_size_in_bytes: u64) -> Self {
        let db = DbInner {
            data: RwLock::new(DbState {
                entries: LinkedHashMap::new(),
                max_cache_size_in_bytes,
                shut_down: false,
            }),
            background_task: Notify::new(),
        };

        let inner = Arc::new(db);
        tokio::spawn(purge_older_keys_if_cache_size_is_above_threshold_task(
            inner.clone(),
        ));
        Self { inner }
    }

    pub fn with_data_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut LinkedHashMap<String, Content>) -> T,
    {
        let result = f(&mut self.inner.data.write().unwrap().entries);
        self.inner.background_task.notify_one();
        result
    }

    pub fn with_data<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&LinkedHashMap<String, Content>) -> T,
    {
        f(&self.inner.data.read().unwrap().entries)
    }

    fn signal_shut_down(&self) {
        let mut data = self.inner.data.write().unwrap();
        data.shut_down = true;
        drop(data);
        self.inner.background_task.notify_one();
        println!("Signaled shut down");
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

async fn purge_older_keys_if_cache_size_is_above_threshold_task(db: Arc<DbInner>) {
    while !db.is_shutting_down() {
        remove_old_entries(&db);
        db.background_task.notified().await;
        println!("Notified");
    }
    println!("Shutting down old entry removing background task")
}

fn remove_old_entries(db: &DbInner) {
    let data = db.data.read().unwrap();
    println!("Here");
    let current_content_byte_size = data
        .entries
        .values()
        .map(|data| data.byte_count as u64)
        .sum::<u64>();
    let cache_size = data.max_cache_size_in_bytes;
    if current_content_byte_size > cache_size {
        // using i64 because we can go below 0
        let mut min_bytes_to_remove = (current_content_byte_size - cache_size) as i64;
        let mut keys_to_remove = Vec::new();
        // linked hashmap maintains insertion order, so iter() gives us the oldest data first.
        for (key, value) in data.entries.iter() {
            if min_bytes_to_remove <= 0 {
                break;
            }
            min_bytes_to_remove -= value.byte_count as i64;
            keys_to_remove.push(key.clone());
        }
        // dropping the read lock.
        drop(data);
        let mut content = db.data.write().unwrap();
        for key in keys_to_remove {
            content.entries.remove(&key);
        }
    }
}
