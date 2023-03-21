use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use log::{debug, info};
use std::io;
use std::path;
use std::time;

use crate::peerd;

use super::clap_value_parsers::{parse_address_network, parse_connect, parse_t_duration};
use super::client::Role;
use super::error::{CmdError, Error};

#[derive(Parser, Debug)]
#[command(name="paymo-cli", bin_name="paymo-cli", author, version, about, long_about = None)]
pub struct Opts {
    #[clap(flatten)]
    pub shared: crate::opts::SharedOpts,

    #[arg(long, value_enum)]
    pub role: Role,

    #[arg(short, long, value_name = "XMR ADDRESS", value_parser = parse_address_network)]
    pub address: monero::Address,

    #[clap(flatten)]
    pub alice_opts: Option<AliceOpts>,

    #[clap(flatten)]
    pub bob_opts: Option<BobOpts>,

    #[clap(skip)]
    pub config_file: path::PathBuf,

    #[clap(long, value_name = "SHELL", value_enum)]
    pub generate_completion: Option<Shell>,
}

impl Opts {
    pub fn try_init() -> crate::Result<Self> {
        let mut opts = Opts::parse();
        debug!("Initial CLI options: {opts:#?}");

        opts.shared.expand_data_dir()?;

        opts.validate_role_opts()?;

        opts.generate_shell_completion();
        opts.populate_config_file()?;

        debug!("Final CLI options: {opts:#?}");

        Ok(opts)
    }

    fn validate_role_opts(&self) -> crate::Result<()> {
        match self.role {
            Role::Alice => self.validate_alice_opts()?,
            Role::Bob => self.validate_bob_opts()?,
        }

        Ok(())
    }

    fn validate_alice_opts(&self) -> crate::Result<()> {
        if self.bob_opts.is_some() {
            let bob_args = BobOpts::get_arguments();
            let err: Error = CmdError::ArgumentConflict(bob_args, "Alice".to_string()).into();

            return Err(err.into());
        }

        if self.alice_opts.is_none() {
            let alice_args = AliceOpts::get_arguments();
            let err: Error = CmdError::MissingArguments(alice_args, "Alice".to_string()).into();
            return Err(err.into());
        }

        let alice_opts = self.alice_opts.as_ref().unwrap();
        let mut missing_args = vec![];

        if alice_opts.channel_amount.is_none() {
            missing_args.push("channel_amount".to_string());
        }
        if alice_opts.time.is_none() {
            missing_args.push("time".to_string());
        }
        if alice_opts.confirmations.is_none() {
            missing_args.push("confirmations".to_string());
        }

        if !missing_args.is_empty() {
            let err: Error = CmdError::MissingArguments(missing_args, "Alice".to_string()).into();
            return Err(err.into());
        }

        Ok(())
    }

    fn validate_bob_opts(&self) -> crate::Result<()> {
        if self.alice_opts.is_some() {
            let alice_args = AliceOpts::get_arguments();
            let err: Error = CmdError::ArgumentConflict(alice_args, "Bob".to_string()).into();

            return Err(err.into());
        }

        if self.bob_opts.is_none() || self.bob_opts.as_ref().unwrap().connect.is_none() {
            let bob_args = BobOpts::get_arguments();
            let err: Error = CmdError::MissingArguments(bob_args, "Bob".to_string()).into();

            return Err(err.into());
        }

        Ok(())
    }

    fn generate_shell_completion(&self) {
        if let Some(shell) = self.generate_completion {
            info!("Generating completion script for {shell:?}");
            let mut cmd = Opts::command();
            let cmd_name = cmd.get_name().to_string();
            generate(shell, &mut cmd, cmd_name, &mut io::stdout());
        }
    }

    fn populate_config_file(&mut self) -> crate::Result<()> {
        self.config_file = self.shared.data_dir.join("paymo.toml");

        if !self.config_file.is_file() {
            return Err(Error::FileNotFound(self.config_file.clone()).into());
        }

        info!(
            "Using config file at: {}",
            self.config_file.to_string_lossy()
        );

        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct AliceOpts {
    #[clap(long)]
    channel_amount: Option<monero::Amount>,

    /// Time in seconds; must be greater than 10s for now
    #[clap(short, long, value_parser = parse_t_duration)]
    time: Option<time::Duration>,

    #[clap(long)]
    confirmations: Option<u32>,
}

#[derive(Parser, Debug)]
pub struct BobOpts {
    #[clap(long, value_parser = parse_connect)]
    connect: Option<peerd::Url>,
}

trait ArgsList: CommandFactory {
    fn get_arguments() -> Vec<String> {
        let cmd = Self::command();

        cmd.get_arguments()
            .map(|arg| {
                let arg_name = arg.get_long().unwrap().to_string();
                format!("--{arg_name}")
            })
            .collect()
    }
}

impl ArgsList for AliceOpts {}
impl ArgsList for BobOpts {}
