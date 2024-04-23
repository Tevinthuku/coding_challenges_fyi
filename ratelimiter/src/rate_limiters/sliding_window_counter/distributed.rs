use std::time::Duration;

use anyhow::{anyhow, Context};
use redis::aio::ConnectionManager;

#[derive(Clone)]
pub struct DistributedSlidingWindowCounter {
    client: ConnectionManager,
    window_duration: Duration,
    max_window_tokens: usize,
}

impl DistributedSlidingWindowCounter {
    pub async fn new() -> anyhow::Result<Self> {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_owned());
        let client = redis::Client::open(url).context("Failed to create redis client")?;
        let conn_manager = ConnectionManager::new(client)
            .await
            .context("Failed to create connection manager")?;

        let window_duration = Duration::from_secs(60);
        let max_window_tokens = 60;

        Ok(Self {
            client: conn_manager,
            window_duration,
            max_window_tokens,
        })
    }

    pub async fn consume_token(&mut self) -> anyhow::Result<bool> {
        let (current_window, current_window_count, previous_window_count): (
            bool,
            Option<usize>,
            Option<usize>,
        ) = redis::pipe()
            .atomic()
            .exists("current_window")
            .get("current_window_count")
            .get("previous_window_count")
            .query_async(&mut self.client)
            .await
            .context("Failed to get values from the DB")?;

        let (current_window_count, previous_window_count) = {
            let expires_at = self.window_duration.as_secs();
            // the window expired: reset the window and set previous window count to current window count
            if !current_window {
                let (current_window_count, previous_window_count): (usize, usize) = redis::pipe()
                    .atomic()
                    .set_ex("current_window", 0, expires_at)
                    .ignore()
                    .set("current_window_count", 0)
                    .ignore()
                    .set("previous_window_count", current_window_count.unwrap_or(0))
                    .ignore()
                    .get("current_window_count")
                    .get("previous_window_count")
                    .query_async(&mut self.client)
                    .await
                    .context("Failed to set values in the DB")?;
                (current_window_count, previous_window_count)
            } else {
                (
                    current_window_count
                        .ok_or(anyhow!("current_window_count should have a value already"))?,
                    previous_window_count
                        .ok_or(anyhow!("previous_window_count should have a value already"))?,
                )
            }
        };

        if current_window_count >= self.max_window_tokens {
            return Ok(false);
        }

        let current_window_percentage_consumed =
            current_window_count as f64 / self.max_window_tokens as f64;

        let previous_window_percentage_to_consider = 1.0 - current_window_percentage_consumed;
        let previous_requests_count_to_consider = (previous_window_percentage_to_consider
            * previous_window_count as f64)
            // we are using .floor so that we consider a full request from the previous window count.
            .floor() as usize;

        let total_requests_count_to_consider =
            current_window_count + previous_requests_count_to_consider;

        if total_requests_count_to_consider < self.max_window_tokens {
            redis::pipe()
                .atomic()
                .incr("current_window_count", 1)
                .ignore()
                .query_async(&mut self.client)
                .await
                .context("Failed to increment the current window count")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
