use derive_more::{Display, Error};
use std::sync::{Arc, Mutex};

use config::{Config, ConfigError};
use log::trace;

#[derive(Clone)]
pub struct Distributor {
    current_backend: Arc<Mutex<usize>>,
    backends: Vec<Backend>,
}

#[derive(Clone, serde::Deserialize, Debug)]
struct Backend {
    url: String,
}

#[derive(Debug, Display, Error)]
pub enum DistributorError {
    PoisonError(#[error(not(source))] String),
    ConfigError(#[error(source)] ConfigError),
    NoBackendsConfigured,
}

impl Distributor {
    pub fn new() -> Result<Self, DistributorError> {
        let base_path = {
            let base_path = std::env::current_dir().map_err(|err| {
                DistributorError::ConfigError(ConfigError::NotFound(err.to_string()))
            })?;
            base_path.join("src/bin/load_balancer/backends.toml")
        };

        let settings = Config::builder()
            .add_source(config::File::from(base_path))
            .build()
            .map_err(DistributorError::ConfigError)?;

        let backends: Vec<Backend> = settings
            .get("backend")
            .map_err(DistributorError::ConfigError)?;

        trace!("backends: {:?}", backends);

        if backends.is_empty() {
            return Err(DistributorError::NoBackendsConfigured);
        }
        Ok(Self {
            backends,
            current_backend: Arc::new(Mutex::new(0)),
        })
    }

    pub fn get_backend(&self) -> Result<&str, DistributorError> {
        let mut current_backend = self
            .current_backend
            .lock()
            .map_err(|err| DistributorError::PoisonError(format!("PoisonError: {:?}", err)))?;
        let backend = &self.backends[*current_backend];
        trace!("Backend running on: {} is selected", backend.url);
        *current_backend = (*current_backend + 1) % self.backends.len();
        Ok(backend.url.as_str())
    }
}
