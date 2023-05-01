use super::Error;
use monero_serai::wallet::address;

pub fn parse_address_network(s: &str) -> Result<address::MoneroAddress, String> {
    let addr = address::MoneroAddress::from_str_raw(s).map_err(|e| e.to_string())?;

    // regtest and mainnet have the same magic bytes
    if addr.meta.network != address::Network::Mainnet {
        return Err(Error::UnsupportedAddress.to_string());
    }

    Ok(addr)
}

pub fn parse_t_duration(s: &str) -> Result<u64, String> {
    let min_hardness = 100;

    let hardness = s.parse().map_err(|e| Into::<Error>::into(e).to_string())?;

    if hardness < min_hardness {
        return Err(Error::InvalidTime(hardness).to_string());
    }

    Ok(hardness)
}
