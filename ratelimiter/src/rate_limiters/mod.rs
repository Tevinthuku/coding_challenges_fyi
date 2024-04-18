use std::collections::HashMap;

pub mod token_bucket;

pub type Ip = String;

#[derive(Debug, Clone, Default)]
pub struct IpRateLimiter {
    buckets: HashMap<Ip, token_bucket::TokenBucket>,
}

impl IpRateLimiter {
    pub fn consume_token(&mut self, ip: Ip) -> bool {
        let bucket = self
            .buckets
            .entry(ip)
            .or_insert_with(|| token_bucket::TokenBucket::new(10, 1));
        bucket.consume_token()
    }
}
