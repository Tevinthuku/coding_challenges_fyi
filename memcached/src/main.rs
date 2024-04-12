pub mod commands;
pub mod db;
pub mod response;
pub mod server;

use std::env;

use anyhow::{anyhow, Context};

use tokio::{net::TcpListener, signal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = env::var("PORT").unwrap_or("11211".to_string());
    let address = format!("127.0.0.1:{port}");
    let cache_size = env::var("CACHE_SIZE")
        .unwrap_or("10000".to_string())
        .parse::<u64>()
        .context("Failed to parse CACHE_SIZE")?;

    let tcp_listener = TcpListener::bind(address)
        .await
        .context("Failed to bind to address")?;

    server::run(tcp_listener, cache_size, signal::ctrl_c()).await
}
