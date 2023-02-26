# PayMo - POC

## Introduction
PayMo [https://eprint.iacr.org/2020/1441](https://eprint.iacr.org/2020/1441) is a protocol that enables uni-directional payment channels in Monero. This repository implements a POC of such paper.

## How to run (what we have so far)
### Pre-requisites
- you need `protoc` installed
- you need a private Monero testnet running; see [https://github.com/moneroexamples/private-testnet](https://github.com/moneroexamples/private-testnet) for details on how to create one

### Running
First, configure `paymo.toml` with configuration options that fit your local workflow.
Next, to run the software, run `cargo run --bin cli` in one tab, and start as Alice. Then, run the same command in another tab, as Bob.

## Architecture
### High-level view
In a high level, PayMo consists of a CLI that spawns a daemon, and such daemon takes care of other processes.

There are two parties that participate in a channel: Alice and Bob. We define Alice as the party that *sends* transactions, i.e. is at the start of the channel; Bob is the party the *receives* the transactions, i.e. is at the end of the channel.

There are two situations: Alice wants to open a channel, or Bob wants to open a channel. Note that we describe the case where Alice wants to open a channel first, but the case where Bob wants to open a channel is symmetric.

#### Alice starts
1. Alice uses the CLI to create an "open channel" offer; the CLI will show params she should select, such as the amount, how long to wait to close the channel if the other party (or Alice herself) is not responsive, how many blocks to wait to consider a transaction "safe", etc
2. After all params are selected, the CLI:
	1. creates an URL that Alice will need to publish somewhere or send to Bob
	2. spawns a daemon process (henceforth called `paymo-daemon`), and `paymo-daemon` spawns another process, namely:
		1. `peerd`: listens for an incoming peer

Now, suppose Bob got the link from Alice. Then, Bob will use the CLI and submit the URL. The URL will show what are the params of the "channel offer", and Bob will be able to accept or refuse.
1. If Bob refuses, the CLI simply exists
2. If Bob accepts, the CLI spawns `paymo-daemon`, and `paymo-daemon` spawns another process, namely:
		1. `peerd`: connects to Alice's `peerd` process

Now, for both Alice and Bob, `paymo-daemon` spawns the following processes:
1. `paymod`: handles the core logic of the protocol
2. `watcherd`: watches for events in the Monero blockchain (e.g. if a transaction was submitted, how many confirmations a transaction has had, etc)
3. `peerd`: has already been spawned; handles communication between the peers
4. `walletd`: handles wallet creation, transaction signing, etc

Note that the CLI will keep running, showing what is happening, what must be done, etc.

#### Notes
- we will start with the case of only a single peer; we will improve later
- the more low-level details of how the protocol works will be described later.
- details on recovering state if one party loses connection, etc, will also be described later
- all processes communicate through `ZeroMQ`, serialized over `ProtoBufs`
- all processes implement command line options (using `clap`), so that they can be spawned with different options
- for now, the communication between peers is not encrypted, but IT MUST BE; we can implement https://github.com/lightning/bolts/blob/master/08-transport.md later OR use `internet2` OR require TLS for peers.

### References
The architecture (multiple processes, name of some processes, etc) is inspired by [https://github.com/farcaster-project/farcaster-node](https://github.com/farcaster-project/farcaster-node). However, `farcaster` is much more complex since it builds upon a fork of [https://github.com/LNP-WG/lnp-node](https://github.com/LNP-WG/lnp-node) and uses `internet2` [https://github.com/cyphernet-dao/rust-internet2](https://github.com/cyphernet-dao/rust-internet2).
We decided to build things from scratch to allow for more experimentation and learning. In the future, we may use `internet2`.
