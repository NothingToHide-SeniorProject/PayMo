use crate::cli;
use curve25519_dalek::{edwards::EdwardsPoint, scalar::Scalar};
use log::debug;
use monero_serai::wallet::address;
use std::{env, ffi::OsStr, fmt::Display, process};

mod channel;
mod lhtlp;
pub mod utils;
mod vtdlog;

#[derive(Debug)]
pub enum PaymoProcess {
    Walled,
    Peerd,
    Watcherd,
}

impl Display for PaymoProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymoProcess::Walled => write!(f, "walletd"),
            PaymoProcess::Peerd => write!(f, "peerd"),
            PaymoProcess::Watcherd => write!(f, "watcherd"),
        }
    }
}

pub fn spawn_process<'a>(
    paymo_process: PaymoProcess,
    args: impl IntoIterator<Item = (&'a str, impl AsRef<OsStr>)>,
) -> crate::Result<process::Child> {
    // is there a better way? See `security` at https://doc.rust-lang.org/std/env/fn.current_exe.html
    let mut bin_path = env::current_exe()?;
    bin_path.pop();

    bin_path.push(paymo_process.to_string());

    debug!(
        "Spawning {} from binary `{}`",
        paymo_process,
        bin_path.to_string_lossy()
    );

    let mut cmd = process::Command::new(bin_path);

    args.into_iter().for_each(|(flag, arg)| {
        cmd.arg(flag).arg(arg);
    });

    debug!("Executing {cmd:?}");

    cmd.spawn().map_err(|e| e.into())
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum Role {
    #[value(alias = "Sender", rename_all = "PascalCase")]
    Alice,

    #[value(alias = "Receiver", rename_all = "PascalCase")]
    Bob,
}

#[derive(Debug)]
pub struct Channel {
    role: Role,

    pub alice_address: Option<address::MoneroAddress>,
    pub bob_address: Option<address::MoneroAddress>,

    pub channel_amount: Option<monero::Amount>,

    pub time: Option<u64>,
    pub confirmations: Option<u32>,

    pub alice_secret: Option<Scalar>,
    pub bob_secret: Option<Scalar>,

    pub alice_public_key: Option<EdwardsPoint>,
    pub bob_public_key: Option<EdwardsPoint>,

    pub alice_hash: Option<Vec<u8>>,
    pub bob_hash: Option<Vec<u8>>,

    pub alice_tag: Option<EdwardsPoint>,
    pub bob_tag: Option<EdwardsPoint>,

    pub joint_public_key: Option<EdwardsPoint>,
    pub joint_tag: Option<EdwardsPoint>,
}

impl Channel {
    pub fn from_opts(opts: &cli::Opts) -> Self {
        let mut channel = Self {
            role: opts.role.clone(),

            alice_address: None,
            bob_address: None,
            channel_amount: None,
            time: None,
            confirmations: None,

            alice_secret: None,
            bob_secret: None,

            alice_hash: None,
            bob_hash: None,

            alice_public_key: None,
            bob_public_key: None,

            alice_tag: None,
            bob_tag: None,

            joint_public_key: None,
            joint_tag: None,
        };

        match opts.role {
            Role::Alice => {
                let alice_opts = opts.alice_opts.as_ref().unwrap();

                channel.alice_address = Some(opts.address);
                channel.channel_amount = alice_opts.channel_amount;
                channel.time = alice_opts.time;
                channel.confirmations = alice_opts.confirmations;
            }
            Role::Bob => {
                // For Bob, the other fields will be set later
                channel.bob_address = Some(opts.address);
            }
        };

        channel
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
