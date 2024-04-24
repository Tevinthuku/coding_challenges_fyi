use actix_web::{web, App, HttpServer};
use anyhow::Context;
use ratelimiter::{
    limited, rate_limiters::sliding_window_counter::distributed::DistributedSlidingWindowCounter,
    AppStateWithIpRateLimiter,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rate_limiter = DistributedSlidingWindowCounter::new().await?;
    HttpServer::new(move || {
        let rate_limiter = AppStateWithIpRateLimiter::new(rate_limiter.clone());
        App::new()
            .app_data(web::Data::new(rate_limiter))
            .service(limited)
    })
    .bind(("127.0.0.1", 8081))
    .context("Failed to bind to port")?
    .run()
    .await
    .context("Failed to run the server")
}
