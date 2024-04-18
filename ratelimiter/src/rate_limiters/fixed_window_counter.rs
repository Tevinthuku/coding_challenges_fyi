use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::time::Instant;

use super::Ip;

#[derive(Default, Clone)]
pub struct IpRateLimiter {
    buckets: HashMap<Ip, FixedWindowCounter>,
}

impl IpRateLimiter {
    pub fn consume_token(&mut self, ip: Ip) -> bool {
        let bucket = self
            .buckets
            .entry(ip)
            .or_insert_with(|| FixedWindowCounter::new(std::time::Duration::from_secs(60), 60));
        bucket.consume_token()
    }
}

#[derive(Debug, Clone)]
pub struct FixedWindowCounter {
    window_size: std::time::Duration,
    max_requests: usize,
    current_window: Arc<Mutex<Window>>,
}

impl FixedWindowCounter {
    pub fn new(window_size: std::time::Duration, max_requests: usize) -> Self {
        FixedWindowCounter {
            window_size,
            max_requests,
            current_window: Arc::new(Mutex::new(Window::new_starting_now(
                window_size,
                max_requests,
            ))),
        }
    }

    pub fn consume_token(&mut self) -> bool {
        let mut current_window = self.current_window.lock().unwrap();
        if current_window.is_expired() {
            *current_window = Window::new_starting_now(self.window_size, self.max_requests);
            return current_window.consume_token();
        }
        current_window.consume_token()
    }
}

#[derive(Debug, Clone, Copy)]
struct Window {
    end: Instant,
    remaining_requests: usize,
}

impl Window {
    fn new_starting_now(window_size: std::time::Duration, max_requests: usize) -> Self {
        let now = Instant::now();
        Window {
            end: now + window_size,
            remaining_requests: max_requests,
        }
    }
    fn is_expired(&self) -> bool {
        Instant::now() >= self.end
    }

    fn consume_token(&mut self) -> bool {
        if self.remaining_requests > 0 {
            self.remaining_requests -= 1;
            true
        } else {
            false
        }
    }
}
