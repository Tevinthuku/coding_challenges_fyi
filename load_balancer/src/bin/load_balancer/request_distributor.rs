use derive_more::{Display, Error};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};
use tokio::time::interval;

use config::{Config, ConfigError};
use log::error;

#[derive(Clone, serde::Deserialize, Debug, PartialEq, Eq)]
struct Server {
    url: String,
    health_endpoint: String,
}

#[derive(Debug, Display, Error)]
pub enum DistributorError {
    PoisonError(#[error(not(source))] String),
    ConfigError(#[error(source)] ConfigError),
    NoServersConfigured,
    NoHealthyServers,
}

#[derive(Clone)]
pub struct Distributor {
    active_servers: Arc<Mutex<VecDeque<Server>>>,
}

impl Distributor {
    pub fn new(shut_down_sender: oneshot::Sender<()>) -> Result<Self, DistributorError> {
        let base_path = {
            let base_path = std::env::current_dir().map_err(|err| {
                DistributorError::ConfigError(ConfigError::NotFound(err.to_string()))
            })?;
            base_path.join("src/bin/load_balancer/servers.toml")
        };

        let settings = Config::builder()
            .add_source(config::File::from(base_path))
            .build()
            .map_err(DistributorError::ConfigError)?;

        let all_servers: Vec<Server> = settings
            .get("server")
            .map_err(DistributorError::ConfigError)?;

        if all_servers.is_empty() {
            return Err(DistributorError::NoServersConfigured);
        }

        let result = Self {
            active_servers: Arc::new(Mutex::new(all_servers.iter().cloned().collect())),
        };

        tokio::spawn(
            result
                .clone()
                .monitor(all_servers.clone(), shut_down_sender),
        );

        Ok(result)
    }

    async fn monitor(self, all_servers: Vec<Server>, shut_down_sender: oneshot::Sender<()>) {
        let (tx, mut rx) = mpsc::channel::<ServerHealth>(all_servers.len());

        tokio::spawn(async move {
            for server in all_servers {
                server.monitor(tx.clone())
            }
        });
        while let Some(data) = rx.recv().await {
            let active_servers = self.active_servers.lock();
            match active_servers {
                Ok(mut active_servers) => {
                    if data.is_healthy {
                        if !active_servers.contains(&data.server) {
                            active_servers.push_back(data.server);
                        }
                    } else {
                        active_servers.retain(|server| server != &data.server);
                    }
                }
                Err(err) => {
                    error!("Failed to acquire a mutex on active servers: {:?}", err);
                    let _ = shut_down_sender.send(());
                    break;
                }
            }
        }
    }

    pub fn get_server(&self) -> Result<String, DistributorError> {
        let mut servers = self
            .active_servers
            .lock()
            .map_err(|err| DistributorError::PoisonError(err.to_string()))?;

        let server = servers.pop_front();
        if let Some(ref server) = server {
            servers.push_back(server.clone());
        }
        server
            .map(|server| server.url)
            .ok_or(DistributorError::NoHealthyServers)
    }
}

impl Server {
    fn monitor(&self, tx: Sender<ServerHealth>) {
        let url = self.url.clone();
        let url = format!("{}{}", url, self.health_endpoint);
        let server = self.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));
            loop {
                let response = reqwest::get(&url)
                    .await
                    .and_then(|response| response.error_for_status());

                if let Err(err) = tx
                    .send(ServerHealth {
                        server: server.clone(),
                        is_healthy: response.is_ok(),
                    })
                    .await
                {
                    error!("Error sending health status: {:?}", err);
                }
                ticker.tick().await;
            }
        });
    }
}

#[derive(Clone, Debug)]
struct ServerHealth {
    server: Server,
    is_healthy: bool,
}
