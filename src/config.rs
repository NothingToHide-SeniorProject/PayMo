use config::File;
use log::info;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct MoneroConfig {
    pub daemon: String,
    pub daemon_zmq: String,
    pub wallet_rpc: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bind_port: u16,
    pub bind_ip: String,
    pub monero: MoneroConfig,
}

impl Config {
    pub fn from_path(path: &Path) -> crate::Result<Self> {
        info!("Loading config from {}", path.to_str().unwrap());

        let conf: config::Config = config::Config::builder()
            .add_source(File::new(path.to_str().unwrap(), config::FileFormat::Toml).required(true))
            .build()
            .map_err(Into::<Error>::into)?;

        let conf: Config = conf.try_deserialize().map_err(Into::<Error>::into)?;

        Ok(conf)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
}
