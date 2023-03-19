use config::File;
use log::{debug, info};
use serde::Deserialize;
use std::path::Path;

// TODO
// pub const FARCASTER_INFO_SOCKET_NAME: &str = "{data_dir}/info";

#[derive(Debug, Deserialize)]
struct MoneroConfig {
    daemon: String,
    wallet_rpc: String,
}

#[derive(Debug, Deserialize)]
struct PaymoConfig {
    bind_port: u16,
    bind_ip: String,
    monero: MoneroConfig,
}

// TODO add zmq sockets addr config
#[derive(Debug, Deserialize)]
pub struct Config {
    paymo: PaymoConfig,
}

impl Config {
    pub fn from_path(path: &Path) -> crate::Result<Self> {
        info!("Loading config from {}", path.to_str().unwrap());

        let conf: config::Config = config::Config::builder()
            .add_source(File::new(path.to_str().unwrap(), config::FileFormat::Toml).required(true))
            .build()
            .map_err(Into::<Error>::into)?;

        let conf: Config = conf.try_deserialize().map_err(Into::<Error>::into)?;

        debug!("{:#?}", conf);

        Ok(Self {
            paymo: PaymoConfig {
                bind_port: 1234,
                bind_ip: "0.0.0.0".to_string(),
                monero: MoneroConfig {
                    daemon: "".to_string(),
                    wallet_rpc: "".to_string(),
                },
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
}
