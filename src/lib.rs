pub mod cli;
pub use cli::client;

pub mod core;
pub mod peerd;
pub mod walletd;
pub mod watcherd;

pub mod bus;
pub mod config;
pub mod msgs;

pub mod opts;
pub use opts::init_logger;

mod error;
pub use error::Error;
pub use error::Result;
