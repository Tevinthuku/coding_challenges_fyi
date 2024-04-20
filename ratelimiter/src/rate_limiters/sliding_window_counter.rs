use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::{DateTime, Utc};

use super::Ip;

struct Window {
    expiry_date_time: DateTime<Utc>,
    max_requests_in_window: usize,
    count: usize,
}

impl Window {
    fn new(duration: Duration, max_requests_in_window: usize) -> Self {
        Self {
            expiry_date_time: Utc::now() + duration,
            max_requests_in_window,
            count: 0,
        }
    }
    fn is_expired(&self) -> bool {
        Utc::now() > self.expiry_date_time
    }

    fn is_full(&self) -> bool {
        self.count >= self.max_requests_in_window
    }
}
struct WindowWithPreviousRequestsCount {
    window: Window,
    previous_requests_count: usize,
}

#[derive(Clone)]
struct SlidingWindowCounter {
    window_duration: Duration,
    max_requests_in_window: usize,
    data: Arc<Mutex<WindowWithPreviousRequestsCount>>,
}

impl SlidingWindowCounter {
    fn consume_token(&mut self) -> bool {
        let mut data = self.data.lock().unwrap();
        if data.window.is_expired() {
            data.previous_requests_count = data.window.count;
            data.window = Window::new(self.window_duration, self.max_requests_in_window);
        }
        if data.window.is_full() {
            return false;
        }

        let current_window_percentage_consumed =
            data.window.count as f64 / self.max_requests_in_window as f64;

        let previous_window_percentage_to_consider = 1.0 - current_window_percentage_consumed;
        let previous_requests_count_to_consider = (previous_window_percentage_to_consider
            * data.previous_requests_count as f64)
            .round() as usize;

        let total_requests_count_to_consider =
            data.window.count + previous_requests_count_to_consider;

        if total_requests_count_to_consider < self.max_requests_in_window {
            data.window.count += 1;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Default)]
pub struct IpRateLimiter {
    window_duration: Option<Duration>,
    max_requests_in_window: Option<usize>,
    buckets: HashMap<Ip, SlidingWindowCounter>,
}

impl IpRateLimiter {
    pub fn new(window_duration: Option<Duration>, max_requests_in_window: Option<usize>) -> Self {
        Self {
            window_duration,
            max_requests_in_window,
            buckets: HashMap::new(),
        }
    }
    pub fn consume_token(&mut self, ip: Ip) -> bool {
        let bucket = self.buckets.entry(ip).or_insert_with(|| {
            let window_duration = self.window_duration.unwrap_or(Duration::from_secs(60));
            let max_requests_in_window = self.max_requests_in_window.unwrap_or(60);

            SlidingWindowCounter {
                window_duration,
                max_requests_in_window,
                data: Arc::new(Mutex::new(WindowWithPreviousRequestsCount {
                    window: Window::new(window_duration, max_requests_in_window),
                    previous_requests_count: 0,
                })),
            }
        });
        bucket.consume_token()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{IpRateLimiter, SlidingWindowCounter, Window, WindowWithPreviousRequestsCount};
    use std::collections::HashMap;
    use std::iter;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter_consumes_all_tokens_if_previous_window_had_no_requests() {
        let ip = "192.23.11.1";
        let max_requests_in_window = 10;
        let mut ip_rate_limiter = IpRateLimiter {
            window_duration: Some(Duration::from_secs(60)),
            max_requests_in_window: Some(max_requests_in_window),
            buckets: HashMap::new(),
        };
        for _ in 0..max_requests_in_window {
            assert!(ip_rate_limiter.consume_token(ip.to_owned()));
        }
        // cannot consume past the max_requests_in_window
        assert!(!ip_rate_limiter.consume_token(ip.to_owned()));
    }
    #[test]
    fn test_ip_rate_limiter_correctly_uses_previous_count_to_determine_if_token_can_be_consumed() {
        let ip = "192.23.11.1";
        let window_duration = Duration::from_secs(60);
        // This rate limiter maxed out the previous window's tokens and the current window's tokens are half consumed
        let mut ip_rate_limiter = IpRateLimiter {
            window_duration: Some(Duration::from_secs(60)),
            max_requests_in_window: Some(100),
            buckets: HashMap::from_iter(iter::once((
                ip.to_owned(),
                SlidingWindowCounter {
                    window_duration,
                    max_requests_in_window: 100,
                    data: Arc::new(Mutex::new(WindowWithPreviousRequestsCount {
                        window: Window {
                            expiry_date_time: Utc::now() + window_duration,
                            max_requests_in_window: 100,
                            count: 50,
                        },
                        previous_requests_count: 100,
                    })),
                },
            ))),
        };

        assert!(!ip_rate_limiter.consume_token(ip.to_owned()));
    }

    #[test]
    fn test_rate_limiter_can_only_consume_up_to_100_percent_of_current_and_previous_window_count() {
        let ip = "192.23.11.1";
        let window_duration = Duration::from_secs(60);
        let current_window_count = 50;
        let max_requests_in_window = 100;
        let mut ip_rate_limiter = IpRateLimiter {
            window_duration: Some(window_duration),
            max_requests_in_window: Some(max_requests_in_window),
            buckets: HashMap::from_iter(iter::once((
                ip.to_owned(),
                SlidingWindowCounter {
                    window_duration,
                    max_requests_in_window,
                    data: Arc::new(Mutex::new(WindowWithPreviousRequestsCount {
                        window: Window {
                            expiry_date_time: Utc::now() + window_duration,
                            max_requests_in_window,
                            count: current_window_count,
                        },
                        previous_requests_count: 50,
                    })),
                },
            ))),
        };
        for _ in current_window_count..max_requests_in_window - 1 {
            assert!(ip_rate_limiter.consume_token(ip.to_owned()));
        }
        assert!(!ip_rate_limiter.consume_token(ip.to_owned()));
    }
}
