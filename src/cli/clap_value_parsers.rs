use crate::peerd;

use super::Error;
use std::net::SocketAddrV4;
use std::str::FromStr;
use std::time;

pub fn parse_address_network(s: &str) -> Result<monero::Address, String> {
    let addr = monero::Address::from_str(s).map_err(|e| e.to_string())?;

    // regtest and mainnet have the same magic bytes
    if addr.network != monero::Network::Mainnet {
        return Err(Error::UnsupportedAddress.to_string());
    }

    Ok(addr)
}

pub fn parse_t_duration(s: &str) -> Result<time::Duration, String> {
    let min_secs = time::Duration::from_secs(10);

    let secs = s.parse().map_err(|e| Into::<Error>::into(e).to_string())?;
    let secs = time::Duration::from_secs(secs);

    if secs < min_secs {
        return Err(Error::InvalidTime(secs).to_string());
    }

    Ok(secs)
}

pub fn parse_connect(s: &str) -> Result<peerd::Url, String> {
    use url::Url;

    let err = Err(Error::InvalidUrl(s.to_string()).to_string());

    let url = Url::parse(s).map_err(|e| e.to_string())?;

    let scheme = url.scheme();
    let host = url.host_str();
    let port = url.port();
    let path = url.path_segments();

    if url.cannot_be_a_base()
        || scheme != "tcp"
        || host.is_none()
        || port.is_none()
        || path.is_none()
    {
        return err;
    }

    let host = host.unwrap();
    let port = port.unwrap();
    let path = path.unwrap().collect::<Vec<_>>();

    if path.len() != 1 {
        return err;
    }

    let uuid = uuid::Uuid::parse_str(path[0]).map_err(|e| e.to_string())?;

    let socket_addr = format!("{host}:{port}");
    let socket_addr = SocketAddrV4::from_str(&socket_addr).unwrap();

    Ok(peerd::Url {
        protocol: peerd::Protocol::Tcp,
        uuid,
        socket_addr,
    })
}
