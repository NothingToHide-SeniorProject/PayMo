use crate::cli;
use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("CLI error: {0}")]
    Cli(#[from] cli::Error),

    #[error(transparent)]
    MoneroAddress(#[from] monero::util::address::Error),
}
