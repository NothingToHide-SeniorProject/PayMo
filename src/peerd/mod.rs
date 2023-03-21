use std::net;

#[derive(Debug, Clone)]
pub enum Protocol {
    Tcp,
}

#[derive(Debug, Clone)]
pub struct Url {
    pub protocol: Protocol,
    pub uuid: uuid::Uuid,
    pub socket_addr: net::SocketAddrV4,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
