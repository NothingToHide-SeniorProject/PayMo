use crate::msgs;
use clap::{ArgGroup, Parser};
use colored::Colorize;
use log::{debug, info};
use msgs::{peer_msg, peerd_msg};
use prost::Message;
use std::{fmt::Display, net, str::FromStr, thread, time::Duration};

#[derive(Debug, Clone)]
pub enum Protocol {
    Tcp,
    Http,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => f.write_str("tcp"),
            Protocol::Http => f.write_str("http"),
        }
    }
}

impl FromStr for Protocol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tcp" => Ok(Protocol::Tcp),
            "http" => Ok(Protocol::Http),
            _ => Err(Error::InvalidProtocol(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Url {
    pub protocol: Protocol,
    pub socket_addr: net::SocketAddrV4,
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}://{}", self.protocol, self.socket_addr))
    }
}

impl FromStr for Url {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = Err(Error::InvalidUrl(s.to_string()));

        let url = url::Url::parse(s)?;

        let scheme: Protocol = url.scheme().parse()?;
        let host = url.host_str();
        let port = url.port();

        if url.cannot_be_a_base() || host.is_none() || port.is_none() {
            return err;
        }

        let host = host.unwrap();
        let port = port.unwrap();

        let socket_addr = format!("{host}:{port}");
        let socket_addr = net::SocketAddrV4::from_str(&socket_addr).unwrap();

        Ok(Url {
            protocol: scheme,
            socket_addr,
        })
    }
}

#[derive(Parser, Debug)]
#[command(name="peerd", bin_name="peerd", author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("zmq")
        .required(true)
        .args(["bind", "connect"]),
))]
pub struct Opts {
    #[clap(flatten)]
    pub shared: crate::opts::SharedOpts,

    #[clap(long)]
    pub bind: Option<String>,

    #[clap(long)]
    pub connect: Option<Url>,
}

impl Opts {
    pub fn try_init() -> crate::Result<Self> {
        let mut opts = Opts::parse();
        opts.shared.expand_data_dir()?;

        debug!("PEERD options: {opts:#?}");

        Ok(opts)
    }
}

pub struct Peerd {
    zmq_context: zmq::Context,

    to_client_socket: Option<zmq::Socket>,
    from_client_socket: Option<zmq::Socket>,

    peerd_socket: Option<zmq::Socket>,
}

impl Peerd {
    pub fn new() -> Self {
        Self {
            zmq_context: zmq::Context::new(),

            to_client_socket: None,
            from_client_socket: None,

            peerd_socket: None,
        }
    }

    fn bind_alice(&mut self, addr: &str) -> crate::Result<()> {
        let peerd_socket = self.zmq_context.socket(zmq::REP)?;
        peerd_socket.bind(addr)?;

        self.peerd_socket = Some(peerd_socket);

        Ok(())
    }

    fn connect_bob(&mut self, addr: &str) -> crate::Result<()> {
        let peerd_socket = self.zmq_context.socket(zmq::REQ)?;
        peerd_socket.connect(addr)?;

        self.peerd_socket = Some(peerd_socket);

        Ok(())
    }

    pub fn run(mut self, opts: Opts) -> crate::Result<()> {
        let (to_client_socket, from_client_socket) = crate::bus::connect_to_client_sockets(
            opts.shared.data_dir,
            self.zmq_context.clone(),
            msgs::Process::Peerd,
        )?;

        // syncronize PUB/SUB sockeets; not the best solution
        thread::sleep(Duration::from_millis(300));

        self.to_client_socket = Some(to_client_socket);
        self.from_client_socket = Some(from_client_socket);

        if let Some(addr) = opts.bind {
            self.bind_alice(&addr)?;
        } else if let Some(url) = opts.connect {
            self.connect_bob(&url.to_string())?;

            self.init_communication()?;
        }

        self.recv()?;

        Ok(())
    }

    fn init_communication(&mut self) -> crate::Result<()> {
        self.send_to_peer(peer_msg::PeerMsgType::AckMe, None)
    }

    fn recv(&mut self) -> crate::Result<()> {
        loop {
            let from_client_socket = self.from_client_socket.as_mut().unwrap();
            let peerd_socket = self.peerd_socket.as_mut().unwrap();

            let mut items = [
                from_client_socket.as_poll_item(zmq::POLLIN),
                peerd_socket.as_poll_item(zmq::POLLIN),
            ];

            zmq::poll(&mut items, -1)?;

            // TODO
            if items[0].is_readable() {}

            if items[1].is_readable() {
                let data = peerd_socket.recv_bytes(0)?;
                self.recv_from_peer(data)?;
            }
        }
    }

    fn send_to_peer(
        &self,
        msg_type: peer_msg::PeerMsgType,
        data: Option<peer_msg::Data>,
    ) -> crate::Result<()> {
        let msg = msgs::PeerMsg {
            msg_type: msg_type as i32,
            data,
        };

        let peerd_socket = self.peerd_socket.as_ref().unwrap();
        peerd_socket.send(msg.encode_to_vec(), 0)?;

        Ok(())
    }

    fn send_to_client(
        &self,
        msg_type: peerd_msg::PeerdMsgType,
        data: Option<peerd_msg::Data>,
    ) -> crate::Result<()> {
        let process_key = msgs::Process::Peerd.as_str_name();
        let msg = msgs::PeerdMsg {
            msg_type: msg_type as i32,
            data,
        };

        let to_client_socket = self.to_client_socket.as_ref().unwrap();

        to_client_socket.send(process_key, zmq::SNDMORE)?;
        to_client_socket.send(msg.encode_to_vec(), 0)?;

        Ok(())
    }

    fn recv_from_peer(&self, data: Vec<u8>) -> crate::Result<()> {
        use peer_msg::PeerMsgType::*;

        let data = msgs::PeerMsg::decode(data.as_slice())?;

        match data.msg_type() {
            AckMe => {
                println!("{}", "BOB CONNECTED".cyan());
                self.send_to_peer(Acked, None)?
            }
            Acked => {
                println!("{}", "ALICE ACKED".cyan());
                println!("{}", "NOW ASKING FOR CHANNEL INFO...".cyan());
                self.send_to_peer(ReqChannelInfo, None)?
            }

            ReqChannelInfo => {
                println!("{}", "RECEIVED REQUEST FOR CHANNEL INFO".cyan());

                println!("{}", "Asking client for channel info...".cyan());
                self.send_to_client(peerd_msg::PeerdMsgType::ReqChannelInfo, None)?;

                let client_data = self.recv_from_client(peerd_msg::PeerdMsgType::ResChannelInfo)?;

                let channel_info = if let peerd_msg::Data::ChannelInfo(channel_info) = client_data {
                    channel_info
                } else {
                    unreachable!()
                };

                println!("{}", "Received channel info from client...".cyan());
                println!("{channel_info:?}");

                println!("{}", "Now sending channel info to Bob...".cyan());

                self.send_to_peer(
                    ResChannelInfo,
                    Some(peer_msg::Data::ChannelInfo(channel_info)),
                )?;

                println!("{}", "Sent".cyan());
            }

            ResChannelInfo => {
                println!(
                    "{}",
                    "RECEIVED CHANNEL INFO, NOW SENDING IT TO CLIENT".cyan()
                );

                let channel_info =
                    if let Some(peer_msg::Data::ChannelInfo(channel_info)) = data.data {
                        channel_info
                    } else {
                        unreachable!()
                    };

                self.send_to_client(
                    peerd_msg::PeerdMsgType::SendChannelInfo,
                    Some(peerd_msg::Data::ChannelInfo(channel_info)),
                )?;

                println!("{}", "ASKING CLIENT FOR MY ADDRESS".cyan());
                self.send_to_client(peerd_msg::PeerdMsgType::BobReqAddress, None)?;

                let client_data = self.recv_from_client(peerd_msg::PeerdMsgType::ResAddress)?;
                let address = if let peerd_msg::Data::Address(address) = client_data {
                    address
                } else {
                    unreachable!()
                };

                println!("{} {}", "CLIENT SAYS MY ADDRESS IS".cyan(), address.cyan());

                println!(
                    "{}",
                    "SENDING MY ADDRESS TO ALICE AND, AT THE SAME TIME, REQUEST ALICE'S ADDRESS"
                        .cyan()
                );

                self.send_to_peer(
                    peer_msg::PeerMsgType::ReqAddress,
                    Some(peer_msg::Data::Address(address)),
                )?;
            }

            ReqAddress => {
                let bob_address = if let Some(peer_msg::Data::Address(bob_address)) = data.data {
                    bob_address
                } else {
                    unreachable!()
                };

                println!("{}", "UPDATING BOB'S ADDRESS IN MY CHANNEL".cyan());
                self.send_to_client(
                    peerd_msg::PeerdMsgType::AliceUpdateBobAddress,
                    Some(peerd_msg::Data::Address(bob_address)),
                )?;

                println!("{}", "ASKING CLIENT FOR MY ADDRESS".cyan());
                self.send_to_client(peerd_msg::PeerdMsgType::AliceReqAddress, None)?;

                let client_data = self.recv_from_client(peerd_msg::PeerdMsgType::ResAddress)?;
                let alice_address = if let peerd_msg::Data::Address(address) = client_data {
                    address
                } else {
                    unreachable!()
                };

                println!(
                    "{} {}",
                    "CLIENT SAYS MY ADDRESS IS".cyan(),
                    alice_address.cyan()
                );
                println!("{}", "SENDING IT TO BOB".cyan());

                self.send_to_peer(ResAddress, Some(peer_msg::Data::Address(alice_address)))?;

                println!("{}", "SENT".cyan());
                println!(
                    "\n{}\n",
                    "NOW WE ARE READY TO BEGIN THE CREATION OF THE MULTISIG ADDRESS".purple()
                );
            }

            ResAddress => {
                let alice_address = if let Some(peer_msg::Data::Address(alice_address)) = data.data
                {
                    alice_address
                } else {
                    unreachable!()
                };

                println!("{}", "UPDATING ALICE'S ADDRESS IN MY CHANNEL".cyan());
                self.send_to_client(
                    peerd_msg::PeerdMsgType::BobUpdateAliceAddress,
                    Some(peerd_msg::Data::Address(alice_address)),
                )?;

                println!(
                    "\n{}\n",
                    "NOW WE ARE READY TO BEGIN THE CREATION OF THE MULTISIG ADDRESS".purple()
                );

                // TODO signal to start creation of multisig address
            }

            Unspecified => unreachable!(),
        };

        Ok(())
    }

    fn recv_from_client(
        &self,
        msg_type: peerd_msg::PeerdMsgType,
    ) -> crate::Result<peerd_msg::Data> {
        let from_client_socket = self.from_client_socket.as_ref().unwrap();

        let _ = from_client_socket.recv_string(0)?;
        let data = from_client_socket.recv_bytes(0)?;

        let data = msgs::PeerdMsg::decode(data.as_slice())?;

        if data.msg_type() == msg_type {
            Ok(data.data.unwrap())
        } else {
            Err(Error::UnmatchedPeerdMsgType(msg_type, data.msg_type()).into())
        }
    }
}

impl Default for Peerd {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid protocol: {0}")]
    InvalidProtocol(String),

    #[error(
        "\
Invalid url: {0}; for now, it must be a well formatted URL that must a tcp:// and \
must contain a port and a host, being IPv4 formatted."
    )]
    InvalidUrl(String),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    #[error("Unmatched peerd msg types. Expected: {0:?}, got: {1:?}")]
    UnmatchedPeerdMsgType(peerd_msg::PeerdMsgType, peerd_msg::PeerdMsgType),
}
