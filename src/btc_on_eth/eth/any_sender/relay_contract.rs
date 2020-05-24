use crate::{errors::AppError, types::Result};
use ethereum_types::Address as EthAddress;

/// An any.sender relay contract address.
/// Should be kept up-to-date with [this](https://github.com/PISAresearch/docs.any.sender#addresses) table.
#[derive(Debug, PartialEq)]
pub enum RelayContract {
    Mainnet,
    Ropsten,
    Unknown(EthAddress),
}

impl RelayContract {
    /// Creates new relay contract from Ethereum chain id.
    pub fn from_eth_chain_id(chain_id: u8) -> Result<RelayContract> {
        match chain_id {
            1 => Ok(RelayContract::Mainnet),
            3 => Ok(RelayContract::Ropsten),
            _ => Err(AppError::Custom(
                "✘ Any.sender is only available on Ropsten and Mainnet!".to_string(),
            )),
        }
    }
}

impl From<RelayContract> for EthAddress {
    fn from(item: RelayContract) -> EthAddress {
        match item {
            RelayContract::Mainnet | RelayContract::Ropsten => EthAddress::from_slice(
                &hex::decode("a404d1219Ed6Fe3cF2496534de2Af3ca17114b06").unwrap(),
            ),
            RelayContract::Unknown(address) => address,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_crete_new_relay_contract_from_eth_chain_id() {
        let relay_contract = RelayContract::from_eth_chain_id(1).unwrap();
        assert_eq!(relay_contract, RelayContract::Mainnet);

        let relay_contract = RelayContract::from_eth_chain_id(3).unwrap();
        assert_eq!(relay_contract, RelayContract::Ropsten);

        RelayContract::from_eth_chain_id(42).expect_err("Should fail with unknown chain id.");
    }

    #[test]
    fn should_return_correct_eth_address() {
        // Mainnet
        let relay_contract = RelayContract::from_eth_chain_id(1).unwrap();
        let relay_contract_address = EthAddress::from(relay_contract);
        let expected_contract_address = EthAddress::from_slice(
            &hex::decode("a404d1219Ed6Fe3cF2496534de2Af3ca17114b06").unwrap(),
        );

        assert_eq!(relay_contract_address, expected_contract_address);

        // Ropsten
        let relay_contract = RelayContract::from_eth_chain_id(3).unwrap();
        let relay_contract_address = EthAddress::from(relay_contract);
        let expected_contract_address = EthAddress::from_slice(
            &hex::decode("a404d1219Ed6Fe3cF2496534de2Af3ca17114b06").unwrap(),
        );

        assert_eq!(relay_contract_address, expected_contract_address);
    }
}
