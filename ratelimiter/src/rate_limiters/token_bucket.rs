use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::Ip;

#[derive(Debug, Clone, Default)]
pub struct IpRateLimiter {
    buckets: HashMap<Ip, TokenBucket>,
}

impl IpRateLimiter {
    pub fn consume_token(&mut self, ip: Ip) -> bool {
        let bucket = self
            .buckets
            .entry(ip)
            .or_insert_with(|| TokenBucket::new(10, 1));
        bucket.consume_token()
    }
}

#[derive(Debug, Clone)]
pub struct TokenBucket {
    data: Arc<Mutex<TokenBucketData>>,
}

#[derive(Debug, Clone, Copy)]
pub struct TokenBucketData {
    capacity: usize,
    num_of_tokens: usize,
    tokens_to_add_per_sec: usize,
}

impl TokenBucket {
    pub fn new(capacity: usize, tokens_to_add_per_sec: usize) -> Self {
        let data = TokenBucketData {
            capacity,
            num_of_tokens: capacity,
            tokens_to_add_per_sec,
        };

        let data = Arc::new(Mutex::new(data));
        tokio::spawn(refill_tokens(data.clone()));
        TokenBucket { data }
    }

    pub fn consume_token(&self) -> bool {
        let mut data = self.data.lock().unwrap();
        if data.num_of_tokens > 0 {
            data.num_of_tokens -= 1;
            true
        } else {
            false
        }
    }
}

async fn refill_tokens(data: Arc<Mutex<TokenBucketData>>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        {
            let mut data = data.lock().unwrap();
            data.num_of_tokens =
                (data.num_of_tokens + data.tokens_to_add_per_sec).min(data.capacity);
        }
    }
}
