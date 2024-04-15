use std::{
    mem,
    sync::{Arc, RwLock},
    time::Duration,
};

use itertools::Itertools;
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
    entries: MapWithByteSizeCount,
}

// We want to keep track of the byte size of the content so that we can remove the oldest content if the cache size exceeds the max_cache_size_in_bytes
// The LinkedHashMap maintains insertion order, so we can remove the oldest content first
// the byte_count field provides a single lookup to get the total byte size of the content stored in the map
#[derive(Default)]
pub struct MapWithByteSizeCount {
    map: LinkedHashMap<String, Content>,
    byte_count: u64,
}

impl MapWithByteSizeCount {
    fn new() -> Self {
        Self {
            map: LinkedHashMap::new(),
            byte_count: 0,
        }
    }

    pub fn insert(&mut self, key: String, value: Content) {
        self.byte_count += value.byte_count as u64;
        self.map.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&Content> {
        self.map.get(key)
    }

    pub fn iter(&self) -> linked_hash_map::Iter<String, Content> {
        self.map.iter()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) {
        if let Some(content) = self.map.remove(key) {
            self.byte_count -= content.byte_count as u64;
        }
    }

    /// Returns true if the key existed and the value was prepended, otherwise false
    pub fn prepend(&mut self, key: &str, value: Content) -> bool {
        if let Some(content) = self.map.get_mut(key) {
            let existing_content = mem::take(&mut content.data);
            content.data = value.data.into_iter().chain(existing_content).collect_vec();
            content.byte_count += value.byte_count;
            self.byte_count += value.byte_count as u64;
            true
        } else {
            false
        }
    }

    /// Returns true if the key existed and the value was appended, otherwise false
    pub fn append(&mut self, key: &str, value: Content) -> bool {
        if let Some(content) = self.map.get_mut(key) {
            content.data.extend(value.data);
            content.byte_count += value.byte_count;
            self.byte_count += value.byte_count as u64;
            true
        } else {
            false
        }
    }
}

impl Db {
    pub fn new(max_cache_size_in_bytes: u64) -> Self {
        let db = DbInner {
            data: RwLock::new(DbState {
                entries: MapWithByteSizeCount::new(),
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
        F: FnOnce(&mut MapWithByteSizeCount) -> T,
    {
        let result = f(&mut self.inner.data.write().unwrap().entries);
        self.inner.background_task.notify_one();
        result
    }

    pub fn with_data<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&MapWithByteSizeCount) -> T,
    {
        f(&self.inner.data.read().unwrap().entries)
    }

    fn signal_shut_down(&self) {
        let mut data = self.inner.data.write().unwrap();
        data.shut_down = true;
        self.inner.background_task.notify_one();
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
    }
}

fn remove_old_entries(db: &DbInner) {
    let data = db.data.read().unwrap();
    let current_content_byte_size = data.entries.byte_count;
    let cache_size = data.max_cache_size_in_bytes;
    if current_content_byte_size > cache_size {
        // using i64 because we can go below 0
        let mut min_bytes_to_remove = (current_content_byte_size - cache_size) as i64;
        let mut keys_to_remove = Vec::new();
        // linked hashmap maintains insertion order, so iter() gives us the oldest data first which is what we want to remove
        for (key, value) in data.entries.iter() {
            if min_bytes_to_remove <= 0 {
                break;
            }
            min_bytes_to_remove -= value.byte_count as i64;
            keys_to_remove.push(key);
        }
        let mut content = db.data.write().unwrap();
        for key in keys_to_remove {
            content.entries.remove(key);
        }
    }
}
