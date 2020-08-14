use std::fmt;
use crate::{
    types::{
        Byte,
        Result,
    },
    errors::AppError,
};

#[derive(Debug, PartialEq, Eq)]
pub enum EthNetwork {
    Kovan,
    Goerli,
    Mainnet,
    Rinkeby,
    Ropsten,
}

impl EthNetwork {
    pub fn from_chain_id(int: &u8) -> Result<Self> {
        match int {
            1 => Ok(EthNetwork::Mainnet),
            3 => Ok(EthNetwork::Ropsten),
            4 => Ok(EthNetwork::Rinkeby),
            5 => Ok(EthNetwork::Goerli),
            42 => Ok(EthNetwork::Kovan),
            _ => Err(AppError::Custom(format!("✘ Unrecognised chain id: '{}'!", int)))
        }
    }

    pub fn to_byte(&self) -> Byte {
        self.to_chain_id() as u8
    }

    pub fn to_chain_id(&self) -> u8 {
        match self {
            EthNetwork::Mainnet => 1,
            EthNetwork::Ropsten => 3,
            EthNetwork::Rinkeby => 4,
            EthNetwork::Goerli => 5,
            EthNetwork::Kovan => 42,
        }
    }

    pub fn from_str(network_str: &str) -> Result<Self> {
        let lowercase_network_str: &str = &network_str.to_lowercase();
        match lowercase_network_str {
            "mainnet" => EthNetwork::from_chain_id(&1),
            "ropsten" => EthNetwork::from_chain_id(&3),
            "rinkeby" => EthNetwork::from_chain_id(&4),
            "goerli"  => EthNetwork::from_chain_id(&5),
            "kovan"   => EthNetwork::from_chain_id(&42),
            _ => Err(AppError::Custom(format!("✘ Unrecognized ethereum network: '{}'!", network_str))),
        }
    }
}

impl fmt::Display for EthNetwork {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EthNetwork::Mainnet => write!(f, "Mainnet"),
            EthNetwork::Kovan => write!(f, "Kovan Testnet"),
            EthNetwork::Goerli => write!(f, "Goerli Testnet"),
            EthNetwork::Ropsten => write!(f, "Ropsten Testnet"),
            EthNetwork::Rinkeby => write!(f, "Rinkeby Testnet"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_mainnet_str_to_ethereum_chain_id_correctly() {
        let network_str = "Mainnet";
        let expected_result = EthNetwork::Mainnet;
        let result = EthNetwork::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_kovan_str_to_ethereum_chain_id_correctly() {
        let network_str = "kOvAN";
        let expected_result = EthNetwork::Kovan;
        let result = EthNetwork::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_ropsten_str_to_ethereum_chain_id_correctly() {
        let network_str = "ROPSTEN";
        let expected_result = EthNetwork::Ropsten;
        let result = EthNetwork::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_goerli_str_to_ethereum_chain_id_correctly() {
        let network_str = "goerli";
        let expected_result = EthNetwork::Goerli;
        let result = EthNetwork::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_rinkeby_str_to_ethereum_chain_id_correctly() {
        let network_str = "rinkeby";
        let expected_result = EthNetwork::Rinkeby;
        let result = EthNetwork::from_str(network_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_convert_unknown_network_correctly() {
        let network_str = "some other network";
        let expected_err = format!("✘ Unrecognized ethereum network: '{}'!", network_str);
        match EthNetwork::from_str(network_str) {
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(err) => panic!("Wrong error received: {}", err),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn should_convert_mainnet_to_correct_chain_id() {
        let eth_network = EthNetwork::Mainnet;
        let expected_result = 1;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_rinekby_to_correct_chain_id() {
        let eth_network = EthNetwork::Rinkeby;
        let expected_result = 4;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_ropsten_to_correct_chain_id() {
        let eth_network = EthNetwork::Ropsten;
        let expected_result = 3;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_goerli_to_correct_chain_id() {
        let eth_network = EthNetwork::Goerli;
        let expected_result = 5;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_kovan_to_correct_chain_id() {
        let eth_network = EthNetwork::Kovan;
        let expected_result = 42;
        let result = eth_network.to_chain_id();
        assert_eq!(result, expected_result);
    }
}
