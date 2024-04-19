use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Timelike, Utc};

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
            .or_insert_with(|| FixedWindowCounter::new(60));
        bucket.consume_token()
    }
}

#[derive(Debug, Clone)]
pub struct FixedWindowCounter {
    max_requests: usize,
    current_window: Arc<Mutex<Window>>,
}

impl FixedWindowCounter {
    pub fn new(max_requests: usize) -> Self {
        FixedWindowCounter {
            max_requests,
            current_window: Arc::new(Mutex::new(Window::new(max_requests))),
        }
    }

    pub fn consume_token(&mut self) -> bool {
        let mut current_window = self.current_window.lock().unwrap();
        if current_window.is_expired() {
            *current_window = Window::new(self.max_requests);
        }
        current_window.consume_token()
    }
}

#[derive(Debug, Clone, Copy)]
struct Window {
    end: DateTime<Utc>,
    remaining_requests: usize,
}

impl Window {
    fn new(max_requests: usize) -> Self {
        let date_time_now = Utc::now();
        // safe to unwrap as we are using a valid seconds value of 59.
        // this window ends at the 59th second of the current minute.
        let end = date_time_now.with_second(59).unwrap();

        Window {
            end,
            remaining_requests: max_requests,
        }
    }
    fn is_expired(&self) -> bool {
        Utc::now() >= self.end
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
