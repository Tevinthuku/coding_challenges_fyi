use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use super::Ip;

#[derive(Default, Clone)]
pub struct IpRateLimiter {
    window_duration: Option<Duration>,
    max_requests_in_window: Option<usize>,
    buckets: HashMap<Ip, Window>,
}

impl IpRateLimiter {
    pub fn consume_token(&mut self, ip: Ip) -> bool {
        let bucket = self.buckets.entry(ip).or_insert_with(|| {
            let window_duration = self.window_duration.unwrap_or(Duration::from_secs(60));
            let max_requests_in_window = self.max_requests_in_window.unwrap_or(60);
            Window::new(window_duration, max_requests_in_window)
        });
        bucket.consume_token()
    }
}

#[derive(Debug, Clone)]
pub struct Window {
    requests: Arc<Mutex<VecDeque<Instant>>>,
    window_duration: Duration,
    max_requests_in_window: usize,
}

impl Window {
    pub fn new(window_duration: Duration, max_requests_in_window: usize) -> Self {
        Window {
            requests: Arc::new(Mutex::new(VecDeque::new())),
            window_duration,
            max_requests_in_window,
        }
    }
    fn consume_token(&mut self) -> bool {
        // TODO: Proper error handling
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        let window_duration = self.window_duration;
        requests.retain(|t| now.duration_since(*t) <= window_duration);

        let can_consume_token = requests.len() < self.max_requests_in_window;

        if can_consume_token {
            requests.push_back(now);
        }

        can_consume_token
    }
}
