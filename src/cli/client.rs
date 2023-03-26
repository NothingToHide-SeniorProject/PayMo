use colored::Colorize;
use log::debug;
use prost::Message;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use super::opts::Opts;
use crate::config::Config;
use crate::core::{self, Role};
use crate::msgs::{self, peerd_msg};

pub struct Client {
    role: core::Role,

    zmq_context: zmq::Context,

    pub_socket: Option<zmq::Socket>,
    sub_socket: Option<zmq::Socket>,

    channel: core::Channel,

    walled_process: Option<process::Child>,
    watcherd_process: Option<process::Child>,
    peerd_process: Option<process::Child>,

    data_dir: PathBuf,

    peerd_url: Option<crate::peerd::Url>,

    monerod_rpc_url: Option<crate::peerd::Url>,
    monerod_zmq_url: Option<crate::peerd::Url>,
    monero_wallet_url: Option<crate::peerd::Url>,
}

impl Client {
    pub fn from_opts(opts: Opts) -> Self {
        let peerd_url = if opts.role == Role::Bob {
            opts.bob_opts.clone().unwrap().connect
        } else {
            None
        };

        Self {
            role: opts.role.clone(),

            zmq_context: zmq::Context::new(),

            pub_socket: None,
            sub_socket: None,

            channel: core::Channel::from_opts(&opts),

            walled_process: None,
            watcherd_process: None,
            peerd_process: None,

            data_dir: opts.shared.data_dir,

            peerd_url,

            monerod_rpc_url: None,
            monerod_zmq_url: None,
            monero_wallet_url: None,
        }
    }

    pub fn add_conf(mut self, conf: Config) -> crate::Result<Self> {
        use crate::peerd::Url;

        let peerd_url = if self.role == Role::Alice {
            let url = format!("tcp://{}:{}", conf.bind_ip, conf.bind_port);
            let url = Url::from_str(&url)?;

            Some(url)
        } else {
            self.peerd_url.take()
        };

        self.peerd_url = peerd_url;
        self.monerod_rpc_url = Some(conf.monero.daemon.parse()?);
        self.monerod_zmq_url = Some(conf.monero.daemon_zmq.parse()?);
        self.monero_wallet_url = Some(conf.monero.wallet_rpc.parse()?);

        Ok(self)
    }

    fn bind_client_sockets(&mut self) -> crate::Result<()> {
        let pub_addr = str::replace(
            crate::bus::CLIENT_PUB_SOCKET,
            "{data_dir}",
            self.data_dir.to_str().unwrap(),
        );
        debug!("Client pub socket: {}", pub_addr);

        let sub_addr = str::replace(
            crate::bus::CLIENT_SUB_SOCKET,
            "{data_dir}",
            self.data_dir.to_str().unwrap(),
        );
        debug!("Client sub socket: {}", sub_addr);

        let pub_socket = self.zmq_context.socket(zmq::PUB)?;
        pub_socket.bind(&pub_addr)?;

        let sub_socket = self.zmq_context.socket(zmq::SUB)?;
        sub_socket.bind(&sub_addr)?;

        sub_socket.set_subscribe(msgs::Process::Peerd.as_str_name().as_bytes())?;
        sub_socket.set_subscribe(msgs::Process::Walletd.as_str_name().as_bytes())?;
        sub_socket.set_subscribe(msgs::Process::Watcherd.as_str_name().as_bytes())?;

        self.pub_socket = Some(pub_socket);
        self.sub_socket = Some(sub_socket);

        Ok(())
    }

    fn spawn_peerd(&self) -> crate::Result<process::Child> {
        let mut args = vec![("-d", self.data_dir.to_str().unwrap())];
        let peerd_url = self.peerd_url.as_ref().unwrap().to_string();

        if self.role == Role::Alice {
            args.push(("--bind", &peerd_url));
        } else {
            args.push(("--connect", &peerd_url));
        };

        core::spawn_process(core::PaymoProcess::Peerd, args)
    }

    // TODO spawn all processes (walletd, watcherd)
    pub fn run(mut self) -> crate::Result<()> {
        self.bind_client_sockets()?;

        self.peerd_process = Some(self.spawn_peerd()?);
        thread::sleep(Duration::from_millis(200));

        if self.role == Role::Alice {
            let peerd_url = self.peerd_url.as_ref().unwrap().to_string();
            println!(
                "ALICE: give this address to Bob: {}",
                peerd_url.bold().bright_cyan(),
            );
        }

        self.recv()?;

        Ok(())
    }

    fn recv(&mut self) -> crate::Result<()> {
        loop {
            let sub_socket = self.sub_socket.as_ref().unwrap();

            let process_key = sub_socket.recv_string(0)?;
            let process_key = msgs::Process::from_str_name(&process_key.unwrap());
            let process_key = process_key.unwrap();

            let data = sub_socket.recv_bytes(0)?;

            match process_key {
                msgs::Process::Peerd => self.recv_from_peerd(data)?,
                msgs::Process::Walletd => todo!(),
                msgs::Process::Watcherd => todo!(),
                msgs::Process::TypeUnspecified => break,
            }
        }

        Ok(())
    }

    fn recv_from_peerd(&mut self, data: Vec<u8>) -> crate::Result<()> {
        use peerd_msg::PeerdMsgType::*;

        let msg = msgs::PeerdMsg::decode(data.as_slice())?;
        debug!("Received message from peerd: {msg:?}");

        match msg.msg_type() {
            ReqChannelInfo => {
                debug!("Received ReqChannelInfo");

                let channel_amount = self.channel.channel_amount.as_ref();
                let channel_amount = channel_amount.unwrap().as_pico();

                let time = self.channel.time.as_ref();
                let time = time.unwrap().as_secs();

                let confirmations = self.channel.confirmations.as_ref();
                let confirmations = *confirmations.unwrap();

                let channel_info = msgs::ChannelInfo {
                    channel_amount,
                    time,
                    confirmations,
                };

                let msg = peerd_msg::Data::ChannelInfo(channel_info);

                self.send_to_peerd(ResChannelInfo, Some(msg))?;
            }

            SendChannelInfo => {
                debug!("Received SendChannelInfo");

                let channel_info = msg.data.unwrap();
                let channel_info = if let peerd_msg::Data::ChannelInfo(channel_info) = channel_info
                {
                    channel_info
                } else {
                    unreachable!()
                };

                let channel_amount = monero::Amount::from_pico(channel_info.channel_amount);
                let time = Duration::from_secs(channel_info.time);

                self.channel.channel_amount = Some(channel_amount);
                self.channel.time = Some(time);
                self.channel.confirmations = Some(channel_info.confirmations);

                debug!("{:#?}", self.channel);
            }

            AliceReqAddress => {
                let address = self.channel.alice_address.as_ref().unwrap().to_string();
                let data = peerd_msg::Data::Address(address);
                self.send_to_peerd(peerd_msg::PeerdMsgType::ResAddress, Some(data))?;
            }

            BobReqAddress => {
                let address = self.channel.bob_address.as_ref().unwrap().to_string();
                let data = peerd_msg::Data::Address(address);
                self.send_to_peerd(peerd_msg::PeerdMsgType::ResAddress, Some(data))?;
            }

            AliceUpdateBobAddress => {
                debug!("Received AliceUpdateBobAddress");

                let bob_address = if let peerd_msg::Data::Address(bob_address) = msg.data.unwrap() {
                    bob_address
                } else {
                    unreachable!()
                };
                self.channel.bob_address = Some(monero::Address::from_str(&bob_address)?);

                debug!("{:#?}", self.channel);
            }

            BobUpdateAliceAddress => {
                debug!("Received BobUpdateAliceAddress");

                let alice_address =
                    if let peerd_msg::Data::Address(alice_address) = msg.data.unwrap() {
                        alice_address
                    } else {
                        unreachable!()
                    };
                self.channel.alice_address = Some(monero::Address::from_str(&alice_address)?);

                debug!("{:#?}", self.channel);
            }

            ResChannelInfo => unreachable!(),
            ResAddress => unreachable!(),
            Unspecified => unreachable!(),
        }

        Ok(())
    }

    fn send_to_peerd(
        &self,
        msg_type: peerd_msg::PeerdMsgType,
        data: Option<peerd_msg::Data>,
    ) -> crate::Result<()> {
        let process_key = msgs::Process::Peerd.as_str_name();
        let msg = msgs::PeerdMsg {
            msg_type: msg_type as i32,
            data,
        };

        let pub_socket = self.pub_socket.as_ref().unwrap();

        pub_socket.send(process_key, zmq::SNDMORE)?;
        pub_socket.send(msg.encode_to_vec(), 0)?;

        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if self.peerd_process.is_some() {
            self.peerd_process.take().unwrap().kill().unwrap();
        }
        if self.walled_process.is_some() {
            self.walled_process.take().unwrap().kill().unwrap();
        }
        if self.watcherd_process.is_some() {
            self.watcherd_process.take().unwrap().kill().unwrap();
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
