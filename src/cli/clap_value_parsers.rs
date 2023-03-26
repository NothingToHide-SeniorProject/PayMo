use super::Error;
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
