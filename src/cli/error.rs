use crate::cli::Opts;
use clap::CommandFactory;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("File does not exist: {0}")]
    FileNotFound(PathBuf),

    #[error("Unsupported address: only regtest addresses are supported at the moment")]
    UnsupportedAddress,

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Invalid --time: must be greater than 100, but {0:?} was provided")]
    InvalidTime(u64),

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
