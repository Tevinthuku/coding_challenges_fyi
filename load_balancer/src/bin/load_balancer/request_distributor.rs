use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use config::{Config, ConfigError};
use log::info;

pub fn config() {
    let settings = Config::builder()
        .add_source(config::File::with_name("./backends.toml"))
        .build();
}

#[derive(Clone)]
struct Distributor {
    current_backend: Arc<Mutex<usize>>,
    backends: Vec<Backend>,
}

#[derive(Clone, serde::Deserialize)]
struct Backend {
    url: String,
}

impl Distributor {
    fn new() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name("./backends.toml"))
            .build()?;

        let backends = settings.try_deserialize::<Vec<Backend>>()?;
        if backends.is_empty() {
            return Err(ConfigError::Message("No backends configured".to_string()));
        }
        Ok(Self {
            backends,
            current_backend: Arc::new(Mutex::new(0)),
        })
    }

    pub fn get_backend(&mut self) -> Result<&str, PoisonError<MutexGuard<'_, usize>>> {
        let mut current_backend = self.current_backend.lock()?;
        let backend = &self.backends[*current_backend];
        info!("Backend {} is selected", backend.url);
        *current_backend = (*current_backend + 1) % self.backends.len();
        Ok(backend.url.as_str())
    }
}
