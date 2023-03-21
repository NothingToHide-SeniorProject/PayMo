// TODO
// WatcherD <-> CLI/Client
// WatcherD <-> MoneroD
//
// Peerd <-> CLI/Client
// Peerd <-> Peerd
//
// WalletD <-> CLI/Client
// WalletD <-> monero-wallet

// TODO run of the protocol:
// Alice:
// creates client socket sub and pub (bind for both)
// spawns watcherd and walletd; both bind to client sub and pub, and also bind to other appropriate
// sockets
// creates url and uuid; save it somewhere in temporary memory; prints it to screen
// spawns peerd, probably a simple req/rep? (bind)
// waits for peerd (Bob to join)
//
// Bob:
// gets url/uuid from Alice
// spawns watcherd and walletd, same as above
// spawns peerd (connect)
//
// Flow of messages:
// Bob -> Alice: I am trying to connect and I have this uuid, got it? (every x seconds)
// Alice -> Bob: Yes, I got it; and here is the info of the channel; do you accept?
// Bob-> Alice: Yes, I accept
// Alice -> Bob: cool, I am starting the channel creation protocol
// Bob -> Alice: Got it
// then do the channel creation protocol (first step: multisignature address)
// Both signal once they got the multisig address (check if they are equal)

use super::opts::Opts;
use crate::config::Config;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Role {
    #[value(alias = "Sender", rename_all = "PascalCase")]
    Alice,

    #[value(alias = "Receiver", rename_all = "PascalCase")]
    Bob,
}

// TODO should have:
// zmq context
// zmq socket pub
// zmq socket sub
// channel
pub struct Client {}

impl Client {
    pub fn new(opts: Opts, conf: Config) -> Self {
        // TODO spawn all processes (walletd, watcherd)
        // TODO proceed as if Bob or Alice
        // TODO spawn peerd as Bob or Alice
        // TODO alice needs to create an uuid and url and send to bob
        Self {}
    }

    pub fn run(&self) -> crate::Result<()> {
        Ok(())
    }

    // TODO send to_x
    // TODO recv from_x
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
