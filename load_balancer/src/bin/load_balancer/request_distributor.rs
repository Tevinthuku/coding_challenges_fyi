use derive_more::{Display, Error};
use std::sync::{Arc, Mutex};

use config::{Config, ConfigError};
use log::trace;

#[derive(Clone)]
pub struct Distributor {
    current_server: Arc<Mutex<usize>>,
    all_servers: Vec<Server>,
}

#[derive(Clone, serde::Deserialize, Debug)]
struct Server {
    url: String,
}

#[derive(Debug, Display, Error)]
pub enum DistributorError {
    PoisonError(#[error(not(source))] String),
    ConfigError(#[error(source)] ConfigError),
    NoServersConfigured,
}

impl Distributor {
    pub fn new() -> Result<Self, DistributorError> {
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

        trace!("all servers: {:?}", all_servers);

        if all_servers.is_empty() {
            return Err(DistributorError::NoServersConfigured);
        }
        Ok(Self {
            all_servers,
            current_server: Arc::new(Mutex::new(0)),
        })
    }

    pub fn get_server(&self) -> Result<&str, DistributorError> {
        let mut current_server = self
            .current_server
            .lock()
            .map_err(|err| DistributorError::PoisonError(format!("PoisonError: {:?}", err)))?;
        let backend = &self.all_servers[*current_server];
        trace!("Server running on: {} is selected", backend.url);
        *current_server = (*current_server + 1) % self.all_servers.len();
        Ok(backend.url.as_str())
    }
}
