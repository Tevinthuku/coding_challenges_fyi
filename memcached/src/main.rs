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

    let tcp_listener = TcpListener::bind(address)
        .await
        .context("Failed to bind to address")?;

    server::run(tcp_listener, signal::ctrl_c()).await
}
