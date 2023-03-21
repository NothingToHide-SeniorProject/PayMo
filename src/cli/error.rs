use crate::cli::Opts;
use clap::CommandFactory;
use std::{path::PathBuf, time};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("File does not exist: {0}")]
    FileNotFound(PathBuf),

    #[error("Unsupported address: only regtest addresses are supported at the moment")]
    UnsupportedAddress,

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Invalid --time: must be greater than 10s, but {0:?} was provided")]
    InvalidTime(time::Duration),

    #[error(
        "\
Invalid url: {0}; for now, it must be a well formatted URL that must a tcp:// and \
must contain a port and a host, being IPv4 formatted. It must also contain a valid \
UUIDv4 as a path. Any other thing passed to the string (such as query params) is ignored."
    )]
    InvalidUrl(String),

    #[error(transparent)]
    Cmd(#[from] CmdError),
}

#[derive(Debug, thiserror::Error)]
pub enum CmdError {
    #[error("Missing arguments: must use {0:?} with {1}")]
    MissingArguments(Vec<String>, String),

    #[error("Argument conflict: cannot use any of {0:?} with {1}")]
    ArgumentConflict(Vec<String>, String),
}

impl From<CmdError> for clap::Error {
    fn from(value: CmdError) -> Self {
        use clap::error::ErrorKind;

        let err_msg = value.to_string();
        let mut cmd = Opts::command();

        match value {
            CmdError::MissingArguments(_, _) => {
                cmd.error(ErrorKind::MissingRequiredArgument, err_msg)
            }
            CmdError::ArgumentConflict(_, _) => cmd.error(ErrorKind::ArgumentConflict, err_msg),
        }
    }
}
