use crate::btc_on_eos::{
    eos::eos_types::EosNetwork,
    types::{
        Bytes,
        Result,
    },
    utils::{
        convert_usize_to_bytes,
        convert_bytes_to_usize,
    },
};

pub fn convert_eos_network_to_bytes(network: &EosNetwork) -> Result<Bytes> {
    match network {
        EosNetwork::Mainnet => Ok(convert_usize_to_bytes(&0)),
        EosNetwork::Testnet => Ok(convert_usize_to_bytes(&1)),
    }
}

pub fn convert_bytes_to_eos_network(bytes: &Bytes) -> Result<EosNetwork> {
    match convert_bytes_to_usize(bytes)? {
        1 => Ok(EosNetwork::Testnet),
        _ => Ok(EosNetwork::Mainnet),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serde_eos_network() {
        let eos_network = EosNetwork::Mainnet;
        let bytes = convert_eos_network_to_bytes(&eos_network)
            .unwrap();
        let result = convert_bytes_to_eos_network(&bytes)
            .unwrap();
        assert_eq!(result, eos_network);
    }
}
