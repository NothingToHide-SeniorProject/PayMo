use crate::{cli, client, config, core, peerd, walletd, watcherd};
use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Config error: {0}")]
    Config(#[from] config::Error),

    #[error("CLI error: {0}")]
    Cli(#[from] cli::Error),

    #[error("Client error: {0}")]
    Client(#[from] client::Error),

    #[error("Core error: {0}")]
    Core(#[from] core::Error),

    #[error("Peerd error: {0}")]
    Peerd(#[from] peerd::Error),

    #[error("Walletd error: {0}")]
    Walletd(#[from] walletd::Error),

    #[error("Watcherd error: {0}")]
    Watcherd(#[from] watcherd::Error),

    #[error(transparent)]
    MoneroAddress(#[from] monero::util::address::Error),
}
